//! The chamber envelope: delay, curved attack, hold, exponential decay.
//!
//! The ranges are wide on purpose. An attack can be a 1 ms snap or a 10 s
//! swell, a hold can sustain for 10 s and a decay can take 20 s, and every
//! chamber can stretch all of that (LENGTH) and start late (DELAY).
//!
//! * The attack runs from wherever the envelope is (so a retrigger never
//!   clicks) to the hit's velocity along `t^p`. A large `p` is a swell that
//!   creeps in and blooms at the end, a small one snaps up and eases in.
//! * During a DELAY, whatever is still ringing from the previous hit keeps
//!   decaying.
//! * Rates are set every control block, so turning ATTACK, HOLD or DECAY
//!   changes a sound that is already running.

use crate::util::t60_gain;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    #[default]
    Idle,
    Delay,
    Attack,
    Hold,
    Decay,
}

#[derive(Clone, Copy, Debug)]
pub struct Env {
    stage: Stage,
    level: f32,
    peak: f32,
    start: f32,
    /// Progress through the attack or the hold, `0..1`.
    t: f32,
    delay_left: u32,
    attack_inc: f32,
    hold_inc: f32,
    decay_mul: f32,
    exponent: f32,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            stage: Stage::Idle,
            level: 0.0,
            peak: 1.0,
            start: 0.0,
            t: 0.0,
            delay_left: 0,
            attack_inc: 1.0,
            hold_inc: 1.0,
            decay_mul: 0.999,
            exponent: 1.0,
        }
    }
}

impl Env {
    /// Start a hit towards `peak` after `delay` samples.
    pub fn trigger(&mut self, peak: f32, delay: u32) {
        self.peak = peak;
        self.delay_left = delay;
        if delay > 0 {
            self.stage = Stage::Delay;
        } else {
            self.begin_attack();
        }
    }

    fn begin_attack(&mut self) {
        self.stage = Stage::Attack;
        self.start = self.level;
        self.t = 0.0;
    }

    /// Times in seconds, `exponent` the attack curve, `fs` the tick rate.
    pub fn set_rates(&mut self, attack_s: f32, hold_s: f32, decay_s: f32, exponent: f32, fs: f32) {
        self.attack_inc = 1.0 / (attack_s * fs).max(1.0);
        self.hold_inc = 1.0 / (hold_s * fs).max(1.0);
        self.decay_mul = t60_gain(decay_s, fs);
        self.exponent = exponent;
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[inline]
    pub fn tick(&mut self) -> f32 {
        match self.stage {
            Stage::Idle => {}
            Stage::Delay => {
                self.level *= self.decay_mul;
                if self.delay_left == 0 {
                    self.begin_attack();
                } else {
                    self.delay_left -= 1;
                }
            }
            Stage::Attack => {
                self.t += self.attack_inc;
                if self.t >= 1.0 {
                    self.level = self.peak;
                    self.stage = Stage::Hold;
                    self.t = 0.0;
                } else {
                    let shape = if self.exponent == 1.0 {
                        self.t
                    } else {
                        self.t.powf(self.exponent)
                    };
                    self.level = self.start + (self.peak - self.start) * shape;
                }
            }
            Stage::Hold => {
                self.t += self.hold_inc;
                if self.t >= 1.0 {
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => {
                self.level *= self.decay_mul;
                if self.level < 1e-6 {
                    self.level = 0.0;
                    self.stage = Stage::Idle;
                }
            }
        }
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(env: &mut Env, n: usize) -> Vec<f32> {
        (0..n).map(|_| env.tick()).collect()
    }

    #[test]
    fn stages_take_their_time() {
        let fs = 1_000.0;
        let mut e = Env::default();
        e.set_rates(2.0, 1.0, 3.0, 1.0, fs);
        e.trigger(0.8, 500);
        let out = run(&mut e, 10_000);
        assert_eq!(out[499], 0.0);
        // Halfway up the attack, linear.
        assert!((out[500 + 1_000] - 0.4).abs() < 0.01, "{}", out[1_500]);
        // Hold.
        assert!((out[500 + 2_000 + 500] - 0.8).abs() < 1e-6);
        // 3 s T60 after the hold: -60 dB.
        let t60 = out[500 + 2_000 + 1_000 + 3_000];
        assert!((t60 / 0.8 - 0.001).abs() < 2e-4, "{t60}");
    }

    #[test]
    fn swell_creeps_and_snap_jumps() {
        let fs = 1_000.0;
        let quarter = |exponent: f32| {
            let mut e = Env::default();
            e.set_rates(1.0, 0.0, 1.0, exponent, fs);
            e.trigger(1.0, 0);
            run(&mut e, 250)[249]
        };
        assert!(quarter(4.0) < 0.01);
        assert!((quarter(1.0) - 0.25).abs() < 0.01);
        assert!(quarter(0.25) > 0.65);
    }

    #[test]
    fn retrigger_starts_from_the_current_level() {
        let fs = 1_000.0;
        let mut e = Env::default();
        e.set_rates(0.5, 0.0, 2.0, 1.0, fs);
        e.trigger(1.0, 0);
        let _ = run(&mut e, 1_000);
        let before = e.level();
        e.trigger(1.0, 0);
        let after = e.tick();
        assert!((after - before).abs() < 0.01, "{before} {after}");
    }

    #[test]
    fn a_delayed_retrigger_keeps_the_tail_decaying() {
        let fs = 1_000.0;
        let mut e = Env::default();
        e.set_rates(0.001, 0.0, 1.0, 1.0, fs);
        e.trigger(1.0, 0);
        let _ = run(&mut e, 10);
        e.trigger(1.0, 300);
        let out = run(&mut e, 300);
        assert!(out[299] < out[0] * 0.3 && out[299] > 0.0);
    }
}
