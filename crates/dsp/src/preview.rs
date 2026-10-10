//! Offline render of a single hit, for the panel's waveform display.

use crate::params::{ALL_PARAMS, Param, VoicePatch, map};
use crate::voice::Voice;

/// How long a hit of `patch` lasts: the longest audible chamber's delay,
/// attack, hold and decay (to -60 dB), plus a little room.
pub fn hit_seconds(patch: &VoicePatch) -> f32 {
    let k = |p: Param| patch[p.index()];
    let body = map::attack_seconds(k(Param::Attack))
        + map::hold_seconds(k(Param::Hold))
        + map::decay_seconds(k(Param::Decay));
    let chambers = [
        (Param::Level1, None, None),
        (Param::Level2, Some(Param::Length2), Some(Param::Delay2)),
        (Param::Level3, Some(Param::Length3), Some(Param::Delay3)),
    ];
    let longest = chambers
        .iter()
        .filter(|(level, _, _)| k(*level) > 0.0)
        .map(|&(_, length, delay)| {
            let len = length.map_or(1.0, |l| map::length_ratio(k(l)));
            let late = delay.map_or(0.0, |d| map::chamber_delay_seconds(k(d)));
            late + body * len
        })
        .fold(0.0f32, f32::max);
    longest.max(body) * 1.05
}

/// Min/max envelope of one hit of `patch`, in `columns` equal slices of
/// `seconds`. Rendered at `fs` without oversampling or reverb: quick and
/// close enough to draw. Deterministic for a given patch.
pub fn hit_outline(
    patch: &VoicePatch,
    velocity: f32,
    fs: f32,
    seconds: f32,
    columns: usize,
) -> Vec<(f32, f32)> {
    let mut v = Voice::new(fs, 0x0bad_5eed);
    for p in ALL_PARAMS {
        v.set_param(p, patch[p.index()]);
    }
    v.snap_params();
    v.trigger(velocity);
    let total = (fs * seconds) as usize;
    let per = (total / columns.max(1)).max(1);
    (0..columns)
        .map(|_| {
            (0..per).fold((0.0f32, 0.0f32), |(lo, hi), _| {
                let s = v.tick();
                (lo.min(s), hi.max(s))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::default_patch;

    #[test]
    fn hit_length_follows_the_envelope_and_the_chambers() {
        let mut p = default_patch();
        let short = hit_seconds(&p);
        p[Param::Attack.index()] = 1.0;
        assert!(hit_seconds(&p) > short + 9.0);
        let mut q = default_patch();
        q[Param::Level2.index()] = 1.0;
        q[Param::Delay2.index()] = 1.0;
        assert!(hit_seconds(&q) > short + 2.9);
        // A silent chamber doesn't count.
        q[Param::Level2.index()] = 0.0;
        assert_eq!(hit_seconds(&q), short);
    }

    #[test]
    fn outline_has_the_requested_shape_and_decays() {
        let o = hit_outline(&default_patch(), 0.8, 24_000.0, 1.0, 100);
        assert_eq!(o.len(), 100);
        let amp = |(lo, hi): (f32, f32)| hi - lo;
        assert!(amp(o[1]) > amp(o[99]) * 5.0);
        assert_eq!(o, hit_outline(&default_patch(), 0.8, 24_000.0, 1.0, 100));
    }
}
