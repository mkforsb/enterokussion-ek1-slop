//! RND and MUTATE: random patches shaped for playable alien digestion, and
//! the parameter locks that keep controls out of them.
//!
//! This lives in the DSP crate (it has no UI dependencies) so the analysis
//! examples can roll and render random sounds exactly as the panel does.

use crate::params::{ENGINE_COUNT, FILTER_BP, FILTER_HP, ParamKind, map};
use crate::util::Noise;
use crate::{ALL_PARAMS, PARAM_COUNT, Param, VoicePatch};

/// Which parameters of one channel are locked against RND and MUTATE.
pub type Locks = [bool; PARAM_COUNT];

pub const NO_LOCKS: Locks = [false; PARAM_COUNT];

/// Can this control be locked? Everything RND and MUTATE would change.
pub fn lockable(p: Param) -> bool {
    !p.is_performance()
}

/// A random patch, as produced by the RND button (and MUTATE at RND on every
/// hit). Performance controls (level, sense, pan) and `locks`ed controls keep
/// their values from `current`.
///
/// Every control draws exactly one number, in parameter order, whether or
/// not it is locked, so locking one control never changes what the others
/// roll. The draws are shaped for *gut sounds*: each engine's pitch comes
/// from its own natural register ([`pitch_range`]), envelopes are usually a
/// fraction of a second to a few seconds but sometimes swell in slowly,
/// peristalsis squeezes about two rolls in three, and each engine's macros
/// stay where it sounds most alive ([`macro_range`]).
pub fn random_patch(current: &VoicePatch, locks: &Locks, rng: &mut Noise) -> VoicePatch {
    let mut patch = *current;
    let mut draws = [0.0f32; PARAM_COUNT];
    for p in ALL_PARAMS {
        draws[p.index()] = rng.sample() * 0.5 + 0.5;
    }
    let bipolar = |x: f32, depth: f32| 0.5 + depth * (2.0 * x - 1.0).powi(3);
    // Often off, otherwise anywhere up to `max`.
    let sometimes = |x: f32, off: f32, max: f32| ((x - off) / (1.0 - off)).max(0.0) * max;
    let lerp = |lo: f32, hi: f32, x: f32| lo + (hi - lo) * x;
    let free = |p: Param| !p.is_performance() && !locks[p.index()];
    for p in ALL_PARAMS {
        if !free(p) {
            continue;
        }
        let x = draws[p.index()];
        patch[p.index()] = p.sanitize(match (p, p.info().kind) {
            (Param::FilterType, _) => match x {
                x if x < 0.35 => 0.0,
                x if x < 0.72 => 1.0,
                x if x < 0.92 => 2.0,
                _ => 3.0,
            },
            (_, ParamKind::Choice(names)) => (x * names.len() as f32).floor(),
            (Param::Fine, _) => lerp(0.3, 0.7, x),
            // 0.25 s … 8 s.
            (Param::Decay, _) => lerp(0.33, 0.86, x),
            (Param::Level1, _) => lerp(0.7, 1.0, x),
            (Param::Level2 | Param::Level3, _) => sometimes(x, 0.4, 0.9),
            (Param::Tune2 | Param::Tune3, _) => map::tune_knob(lerp(-12.0, 12.0, x)),
            // ×0.35 … ×2.
            (Param::Length2 | Param::Length3, _) => lerp(0.3, 0.8, x),
            // Up to about a second late, half the time.
            (Param::Delay2 | Param::Delay3, _) => sometimes(x, 0.5, 0.6),
            // 0.23 … 11 Hz.
            (Param::PeriRate, _) => lerp(0.15, 0.85, x),
            (Param::PeriSqueeze, _) => sometimes(x, 0.35, 0.9),
            (Param::PeriPitch, _) => bipolar(x, 0.35),
            (Param::PeriIrreg, _) => 0.8 * x,
            // Median ≈ 50 ms, one roll in four over 0.4 s, one in ten over
            // 1.8 s.
            (Param::Attack, _) => 0.1 + 0.82 * x.powf(1.3),
            (Param::Curve, _) => lerp(0.15, 0.85, x),
            // Up to 3 s.
            (Param::Hold, _) => 0.55 * x * x,
            (Param::FizzLevel, _) => sometimes(x, 0.6, 0.6),
            (Param::FizzTone, _) => lerp(0.2, 0.85, x),
            (Param::SweepSpeed, _) => lerp(0.2, 0.75, x),
            (Param::SweepDepth, _) => bipolar(x, 0.3),
            (Param::ChaosDepth, _) => 0.6 * x * x,
            (Param::Jitter, _) => 0.4 * x * x,
            (Param::Cutoff, _) => lerp(0.5, 1.0, x),
            (Param::Reso, _) => 0.7 * x * x,
            (Param::FilterEnv, _) => bipolar(x, 0.3),
            (Param::Drive, _) => 0.6 * x * x,
            (Param::SatMix, _) => lerp(0.4, 1.0, x),
            (Param::LowCut, _) => sometimes(x, 0.6, 0.3),
            (Param::LowGain, _) => bipolar(x, 0.25),
            (Param::MidGain, _) => bipolar(x, 0.4),
            (Param::MidQ, _) => lerp(0.2, 0.8, x),
            (Param::HighGain, _) => bipolar(x, 0.3),
            (Param::ChorusMix, _) => sometimes(x, 0.55, 0.8),
            (Param::ChorusXover, _) => lerp(0.0, 0.6, x),
            // The belly is a cavity: a fair bit of room, sometimes a lot.
            (Param::ReverbMix, _) => 0.6 * x,
            (Param::ReverbDecay, _) => lerp(0.2, 0.8, x),
            (Param::ReverbTone, _) => lerp(0.2, 0.8, x),
            (Param::ReverbPredelay, _) => 0.6 * x,
            _ => x,
        });
    }
    let engine = (patch[Param::Engine.index()] as u32).min(ENGINE_COUNT as u32 - 1);
    // Each engine's pitch register, log-uniform.
    if free(Param::Pitch) {
        let (lo, hi) = pitch_range(engine);
        let hz = lo * (hi / lo).powf(draws[Param::Pitch.index()]);
        patch[Param::Pitch.index()] = map::pitch_knob(hz).clamp(0.0, 1.0);
    }
    // Each engine's macros, within their lively range.
    for (slot, p) in [Param::MacroX, Param::MacroY, Param::MacroZ]
        .into_iter()
        .enumerate()
    {
        if free(p) {
            let (lo, hi) = macro_range(engine, slot);
            patch[p.index()] = lerp(lo, hi, draws[p.index()]);
        }
    }
    // A highpass or bandpass with a lowpass's cutoff would leave little:
    // pull the cutoff into the body of the sound.
    let ftype = patch[Param::FilterType.index()] as u32;
    if (ftype == FILTER_HP || ftype == FILTER_BP) && free(Param::Cutoff) {
        let x = draws[Param::Cutoff.index()];
        let base = map::pitch_hz(patch[Param::Pitch.index()], 0.5);
        // HP: up to just below the pitch. BP: around one to four times it.
        let (lo, hi) = if ftype == FILTER_HP {
            (base / 8.0, base * 0.9)
        } else {
            (base, base * 4.0)
        };
        let hz = (lo * (hi / lo).powf(x)).clamp(20.0, 15_000.0);
        patch[Param::Cutoff.index()] = (hz / 20.0).log(1000.0);
    }
    // A lowpass mustn't swallow the sound: keep its cutoff, even after a
    // negative envelope sweep, above the pitch (squeaks and critters live
    // high up).
    if ftype != FILTER_HP && ftype != FILTER_BP && free(Param::Cutoff) {
        let base = map::pitch_hz(patch[Param::Pitch.index()], 0.5);
        let dip = (-map::filter_env_octaves(patch[Param::FilterEnv.index()])).max(0.0);
        let lowest = (1.5 * base * dip.exp2()).min(20_000.0);
        let knob = (lowest / 20.0).log(1000.0);
        let c = &mut patch[Param::Cutoff.index()];
        *c = c.max(knob).min(1.0);
    }
    // Keep chambers 2 and 3 between 20 Hz and 5 kHz: fold them by octaves.
    let base = map::pitch_hz(patch[Param::Pitch.index()], patch[Param::Fine.index()]);
    for tune in [Param::Tune2, Param::Tune3] {
        if free(tune) {
            let mut st = map::tune_semitones(patch[tune.index()]);
            while base * (st / 12.0).exp2() < 20.0 && st < 12.0 {
                st += 12.0;
            }
            while base * (st / 12.0).exp2() > 5_000.0 && st > -12.0 {
                st -= 12.0;
            }
            patch[tune.index()] = tune.sanitize(map::tune_knob(st));
        }
    }
    patch
}

/// The pitch register RND draws from for each engine, in Hz.
pub fn pitch_range(engine: u32) -> (f32, f32) {
    const RANGES: [(f32, f32); ENGINE_COUNT] = [
        // GURGLE: big gloops to small plinks.
        (90.0, 900.0),
        // FLATUS: tuba to trumpet.
        (35.0, 260.0),
        // GROWL: a beast's throat.
        (30.0, 180.0),
        // SQUEAK: rubber on rubber.
        (180.0, 2_200.0),
        // TRACT: a long tube to a short one.
        (45.0, 400.0),
        // SYMBIO
        (40.0, 600.0),
        // FLORA: small critters, high voices.
        (250.0, 2_500.0),
        // RUMBLE: the deep end.
        (25.0, 110.0),
    ];
    RANGES[(engine as usize).min(ENGINE_COUNT - 1)]
}

/// The part of each engine's macro ranges (X, Y, Z) that RND draws from.
pub fn macro_range(engine: u32, slot: usize) -> (f32, f32) {
    const RANGES: [[(f32, f32); 3]; ENGINE_COUNT] = [
        // GURGLE: from drips to a boil, any scatter, any chirp.
        [(0.1, 0.85), (0.0, 0.8), (0.2, 1.0)],
        // FLATUS: everything goes.
        [(0.0, 1.0), (0.0, 1.0), (0.0, 0.9)],
        // GROWL: any vowel, mostly rough, mostly big.
        [(0.0, 1.0), (0.2, 1.0), (0.0, 0.8)],
        // SQUEAK: any pressure, enough slide to get it going.
        [(0.0, 1.0), (0.25, 1.0), (0.0, 1.0)],
        // TRACT: a real squeeze, a wave in either direction.
        [(0.3, 1.0), (0.1, 0.9), (0.0, 1.0)],
        // SYMBIO: any partner, coupling up to chaos.
        [(0.0, 1.0), (0.1, 0.9), (0.0, 0.8)],
        // FLORA: a colony that talks.
        [(0.2, 0.85), (0.1, 1.0), (0.0, 1.0)],
        // RUMBLE: everything goes.
        [(0.0, 1.0), (0.0, 1.0), (0.0, 1.0)],
    ];
    RANGES[(engine as usize).min(ENGINE_COUNT - 1)][slot.min(2)]
}

/// Move every free control `amount` (0..=1) of the way towards a fresh random
/// patch: `clamp(current + (random - current) * amount)`. At 1 this is exactly
/// [`random_patch`]; choice controls round to the nearest position. Locked
/// controls never move.
pub fn mutate_patch(
    current: &VoicePatch,
    amount: f32,
    locks: &Locks,
    rng: &mut Noise,
) -> VoicePatch {
    let target = random_patch(current, locks, rng);
    if amount >= 1.0 {
        return target;
    }
    let amount = amount.max(0.0);
    let mut patch = *current;
    for p in ALL_PARAMS {
        let (c, t) = (current[p.index()], target[p.index()]);
        patch[p.index()] = p.sanitize(c + (t - c) * amount);
    }
    patch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::default_patch;
    use crate::voice::Voice;

    fn is_sanitized(patch: &VoicePatch) -> bool {
        ALL_PARAMS
            .iter()
            .all(|p| p.sanitize(patch[p.index()]) == patch[p.index()])
    }

    #[test]
    fn mutate_off_changes_nothing() {
        let current = default_patch();
        assert_eq!(
            mutate_patch(&current, 0.0, &NO_LOCKS, &mut Noise::new(7)),
            current
        );
    }

    #[test]
    fn mutate_at_rnd_equals_the_rnd_button() {
        let current = default_patch();
        let a = mutate_patch(&current, 1.0, &NO_LOCKS, &mut Noise::new(7));
        let b = random_patch(&current, &NO_LOCKS, &mut Noise::new(7));
        assert_eq!(a, b);
        assert_ne!(a, current);
    }

    #[test]
    fn partial_mutation_moves_part_way_towards_the_random_patch() {
        let current = default_patch();
        for seed in 1..50 {
            let target = random_patch(&current, &NO_LOCKS, &mut Noise::new(seed));
            let half = mutate_patch(&current, 0.5, &NO_LOCKS, &mut Noise::new(seed));
            assert!(is_sanitized(&half));
            for p in ALL_PARAMS {
                let (c, t, h) = (current[p.index()], target[p.index()], half[p.index()]);
                match p.info().kind {
                    ParamKind::Continuous => {
                        assert!((h - (c + (t - c) * 0.5)).abs() < 1e-6, "{p:?}")
                    }
                    ParamKind::Choice(_) => assert_eq!(h, p.sanitize(c + (t - c) * 0.5), "{p:?}"),
                }
            }
        }
    }

    #[test]
    fn mutation_keeps_level_sense_and_pan() {
        let mut current = default_patch();
        current[Param::Level.index()] = 0.33;
        current[Param::Sense.index()] = 0.66;
        current[Param::Pan.index()] = 0.1;
        let mut rng = Noise::new(3);
        for amount in [0.25, 0.5, 1.0] {
            let m = mutate_patch(&current, amount, &NO_LOCKS, &mut rng);
            for p in [Param::Level, Param::Sense, Param::Pan] {
                assert_eq!(m[p.index()], current[p.index()]);
            }
        }
    }

    #[test]
    fn locked_controls_never_move() {
        let mut current = default_patch();
        current[Param::Engine.index()] = 3.0;
        current[Param::Pitch.index()] = 0.123;
        current[Param::FilterType.index()] = FILTER_HP as f32;
        current[Param::Cutoff.index()] = 0.9;
        let mut locks = NO_LOCKS;
        let locked = [
            Param::Engine,
            Param::Pitch,
            Param::FilterType,
            Param::Cutoff,
        ];
        for p in locked {
            locks[p.index()] = true;
        }
        let mut rng = Noise::new(17);
        let mut patch = current;
        for i in 0..500 {
            patch = mutate_patch(&patch, [0.3, 1.0][i % 2], &locks, &mut rng);
            for p in locked {
                assert_eq!(patch[p.index()], current[p.index()], "{p:?}");
            }
        }
        // …while the free controls do move.
        assert_ne!(patch[Param::Decay.index()], current[Param::Decay.index()]);
    }

    #[test]
    fn locking_one_control_leaves_the_other_rolls_unchanged() {
        let current = default_patch();
        let mut locks = NO_LOCKS;
        locks[Param::Decay.index()] = true;
        locks[Param::MacroY.index()] = true;
        let a = random_patch(&current, &NO_LOCKS, &mut Noise::new(42));
        let b = random_patch(&current, &locks, &mut Noise::new(42));
        for p in ALL_PARAMS {
            if !locks[p.index()] {
                assert_eq!(a[p.index()], b[p.index()], "{p:?}");
            }
        }
    }

    /// A locked engine still gets its own pitch register.
    #[test]
    fn a_locked_engine_keeps_its_register() {
        use crate::params::ENGINE_SQUEAK;
        let mut current = default_patch();
        current[Param::Engine.index()] = ENGINE_SQUEAK as f32;
        let mut locks = NO_LOCKS;
        locks[Param::Engine.index()] = true;
        let mut rng = Noise::new(9);
        let (lo, hi) = pitch_range(ENGINE_SQUEAK);
        for _ in 0..100 {
            let p = random_patch(&current, &locks, &mut rng);
            let hz = map::pitch_hz(p[Param::Pitch.index()], 0.5);
            assert!(hz > lo * 0.99 && hz < hi * 1.01, "{hz}");
        }
    }

    #[test]
    fn repeated_mutation_stays_in_range() {
        let mut patch = default_patch();
        let mut rng = Noise::new(11);
        for i in 0..1000 {
            patch = mutate_patch(&patch, [0.1, 0.5, 0.9][i % 3], &NO_LOCKS, &mut rng);
            assert!(is_sanitized(&patch));
        }
    }

    fn voice_for(patch: &VoicePatch, fs: f32, seed: u32) -> Voice {
        let mut v = Voice::new(fs, seed);
        for p in ALL_PARAMS {
            v.set_param(p, patch[p.index()]);
        }
        v.snap_params();
        v
    }

    /// RND (and MUTATE at RND) must always give something audible and sane,
    /// whichever engine it lands on, even when it swells in slowly.
    #[test]
    fn random_patches_are_playable() {
        let mut rng = Noise::new(2024);
        let mut engines = [0usize; ENGINE_COUNT];
        // At a realistic rate: the friction and valve loops depend on it.
        let fs = 48_000.0;
        for i in 0..300 {
            let patch = random_patch(&default_patch(), &NO_LOCKS, &mut rng);
            engines[patch[Param::Engine.index()] as usize] += 1;
            let mut v = voice_for(&patch, fs, i);
            v.trigger(0.9);
            let out: Vec<f32> = (0..(fs * 4.0) as usize).map(|_| v.tick()).collect();
            let peak = out.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
            assert!(out.iter().all(|x| x.is_finite()), "patch {i}");
            assert!(
                peak > 0.02 && peak < 4.0,
                "patch {i}: peak {peak} {patch:?}"
            );
        }
        assert!(engines.iter().all(|&n| n > 20), "engine spread {engines:?}");
    }

    /// EK-1's point: RND gives sounds of every length and onset, from short
    /// blurts to long swells.
    #[test]
    fn random_envelopes_have_range() {
        let mut rng = Noise::new(77);
        let n = 1_000;
        let (mut slow, mut long, mut short) = (0, 0, 0);
        for _ in 0..n {
            let p = random_patch(&default_patch(), &NO_LOCKS, &mut rng);
            let attack = map::attack_seconds(p[Param::Attack.index()]);
            let total = attack
                + map::hold_seconds(p[Param::Hold.index()])
                + map::decay_seconds(p[Param::Decay.index()]);
            slow += (attack > 0.4) as usize;
            long += (total > 3.0) as usize;
            short += (total < 1.0) as usize;
        }
        assert!(slow * 100 > n * 15, "slow attacks {slow}/{n}");
        assert!(long * 100 > n * 25, "long sounds {long}/{n}");
        assert!(short * 100 > n * 10, "short sounds {short}/{n}");
    }
}
