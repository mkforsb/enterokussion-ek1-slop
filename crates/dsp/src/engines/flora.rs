//! FLORA: a chattering swarm of gut flora.
//!
//! Billions of tiny residents, each with something to say. Every critter is
//! a short chirp: a sine in a raised-cosine window, gliding exponentially up
//! or down, with a warble (vibrato) of its own. They are released as a
//! Poisson stream around PITCH.
//!
//! * DENSITY: critters per second at full pressure. The rate follows the
//!   pressure, so the colony quietens as the envelope decays and squabbles
//!   on every peristaltic squeeze.
//! * CHIRP: how far each one glides (mostly up, as questions do) and how
//!   hard it warbles, from short peeps to sweeping squeals.
//! * CHATTER: turns the steady stream into conversation. A two-state
//!   process switches the colony between talking and listening a few times
//!   a second. While talking, critters come fast and share the talker's
//!   pitch, so they group into syllables; while listening, almost nobody
//!   speaks. It also makes the chirps shorter and more clipped.

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{Noise, sin_turns};

const MAX_CRITTERS: usize = 10;
/// Pitch scatter of the colony, ± octaves.
const SPREAD_OCT: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default)]
struct Critter {
    on: bool,
    phase: f32,
    inc: f32,
    /// Per-sample glide factor of `inc`.
    glide: f32,
    win: f32,
    win_inc: f32,
    vib_phase: f32,
    vib_inc: f32,
    /// Vibrato depth, in turns of phase modulation.
    vib_depth: f32,
    amp: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Flora {
    rng: Noise,
    critters: [Critter; MAX_CRITTERS],
    fs: f32,
    hz: f32,
    p_release: f32,
    chirp_oct: f32,
    chatter: f32,
    talking: bool,
    p_switch: f32,
    /// The current talker's pitch offset in octaves.
    voice_oct: f32,
    velocity: f32,
    norm: f32,
}

impl Flora {
    fn release(&mut self) {
        let Some(slot) = self.critters.iter().position(|c| !c.on) else {
            return;
        };
        let rng = &mut self.rng;
        let fs = self.fs;
        let own = if self.chatter > 0.0 && self.talking {
            self.voice_oct + 0.15 * rng.sample()
        } else {
            SPREAD_OCT * rng.sample()
        };
        let hz = (self.hz * own.exp2()).clamp(20.0, 0.3 * fs);
        let seconds = (0.015 + 0.11 * rng.uniform()) * (1.0 - 0.6 * self.chatter);
        // Glide over the critter's life: mostly up.
        let glide_oct = self.chirp_oct * (rng.uniform() * 1.3 - 0.3);
        let samples = seconds * fs;
        self.critters[slot] = Critter {
            on: true,
            phase: 0.0,
            inc: hz / fs,
            glide: (glide_oct / samples).exp2(),
            win: 0.0,
            win_inc: 1.0 / samples,
            vib_phase: rng.uniform(),
            vib_inc: (8.0 + 30.0 * rng.uniform()) / fs,
            vib_depth: self.chirp_oct * 0.3 * rng.uniform(),
            amp: (0.4 + 0.6 * rng.uniform()) * (0.5 + 0.5 * self.velocity),
        };
    }
}

impl Osc for Flora {
    fn reset(&mut self) {
        self.critters = [Critter::default(); MAX_CRITTERS];
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        self.talking = true;
        self.voice_oct = SPREAD_OCT * self.rng.sample();
        self.control(c, o);
        self.release();
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        self.fs = c.fs;
        self.hz = o.hz;
        self.velocity = c.velocity;
        self.chirp_oct = map::flora_chirp_octaves(c.y);
        self.chatter = c.z;
        let state = if self.talking {
            1.0 + 4.0 * self.chatter
        } else {
            1.0 - 0.995 * self.chatter
        };
        let density = map::flora_density(c.x);
        self.p_release = density * state * o.env.clamp(0.0, 1.0).sqrt() / c.fs;
        // Turns of a few hundred milliseconds.
        self.p_switch = (1.5 + 2.5 * self.chatter) / c.fs;
        // Mean life ≈ 70 ms; overlapping critters add in power.
        let overlap = density * 0.07 * (1.0 - 0.6 * self.chatter) * (1.0 + 2.0 * self.chatter);
        self.norm = 1.25 / (1.0 + overlap).sqrt();
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        if self.chatter > 0.0 && self.rng.uniform() < self.p_switch {
            self.talking = !self.talking;
            if self.talking {
                self.voice_oct = SPREAD_OCT * self.rng.sample();
            }
        }
        if self.rng.uniform() < self.p_release {
            self.release();
        }
        let mut y = 0.0;
        for c in self.critters.iter_mut() {
            if !c.on {
                continue;
            }
            // Hann window over the critter's life.
            let window = 0.5 - 0.5 * sin_turns(c.win + 0.25);
            let vib = c.vib_depth * sin_turns(c.vib_phase);
            y += c.amp * window * sin_turns(c.phase + vib);
            c.phase += c.inc;
            if c.phase >= 1.0 {
                c.phase -= 1.0;
            }
            c.inc = (c.inc * c.glide).min(0.45);
            c.vib_phase += c.vib_inc;
            if c.vib_phase >= 1.0 {
                c.vib_phase -= 1.0;
            }
            c.win += c.win_inc;
            if c.win >= 1.0 {
                c.on = false;
            }
        }
        y * self.norm
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_FLORA;

    /// CHATTER groups the critters into turns with silences between: the
    /// steady stream has hardly any quiet stretches, the conversation does.
    #[test]
    fn chatter_takes_turns() {
        let fs = 48_000.0;
        let quiet = |z: f32| {
            let out = run(ENGINE_FLORA, fs, 800.0, (0.75, 0.5, z), 1.0, 8.0);
            let env = envelope(&out, 2_400);
            let mean = env.iter().sum::<f32>() / env.len() as f32;
            env.iter().filter(|&&e| e < 0.05 * mean).count() as f32 / env.len() as f32
        };
        assert!(quiet(0.0) < 0.03, "{}", quiet(0.0));
        assert!(quiet(1.0) > 0.15, "{}", quiet(1.0));
    }
}
