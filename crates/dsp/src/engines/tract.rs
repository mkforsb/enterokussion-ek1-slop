//! TRACT: a long digestive tube with a peristaltic constriction travelling
//! through it.
//!
//! The tube is a Kelly–Lochbaum waveguide, the model of the human vocal
//! tract: eight cylindrical sections, each a pair of delay lines (one per
//! direction), joined by scattering junctions whose reflection coefficients
//! come from the ratio of neighbouring cross-sections:
//!
//! ```text
//!  k = (Aᵢ − Aᵢ₊₁) / (Aᵢ + Aᵢ₊₁)     p⁺ᵢ₊₁ = (1 + k)·p⁺ᵢ − k·p⁻ᵢ₊₁     p⁻ᵢ = k·p⁺ᵢ + (1 − k)·p⁻ᵢ₊₁
//! ```
//!
//! The far end of the tube is closed (gas pushes in there), the near end
//! open and radiating, so an even tube resonates like a quarter-wave pipe:
//! PITCH, 3×, 5× … The section delays are set so the first resonance lands
//! on PITCH.
//!
//! A ring of muscle squeezes the tube: a Gaussian narrowing of the area
//! function that travels along it. Where it pinches, the tube splits into two
//! shorter ones and every resonance moves, so the tone wah-wahs, gulps and
//! squelches as the wave passes, and gas forced through the narrowing hisses.
//!
//! * SQUEEZE: how far the ring closes (up to 97%).
//! * WAVE: speed and direction of the peristaltic wave, ±6 Hz. At the centre
//!   the constriction parks in the middle of the tube.
//! * GAS: the excitation, from a buzz at PITCH (a voiced, vowel-like gut) to
//!   a rush of gas noise (wind through the bowels).

use super::{Ctx, Osc, OscCtx};
use crate::params::map;
use crate::util::{Noise, fast_tanh, flush, one_pole_coeff, sin_turns};

const SECTIONS: usize = 8;
/// Ring buffer length per delay line (power of two).
const LINE: usize = 512;
const MAX_DELAY: f32 = (LINE - 4) as f32;
/// Reflection at the closed (source) end.
const CLOSED_END: f32 = 0.94;
/// Reflection at the open end (inverting, lowpassed).
const OPEN_END: f32 = -0.9;
/// Loss per section pass.
const LOSS: f32 = 0.996;
/// Width of the constriction, as a fraction of the tube.
const RING_WIDTH: f32 = 0.16;

#[derive(Clone, Debug)]
pub struct Tract {
    rng: Noise,
    /// Forward lines then backward lines, `LINE` samples each.
    buf: Vec<f32>,
    pos: usize,
    delay: f32,
    k: [f32; SECTIONS - 1],
    /// Section with the narrowest area (where turbulence is injected).
    pinch: usize,
    pinch_amount: f32,
    wave_phase: f32,
    phase: f32,
    inc: f32,
    buzz: f32,
    gas: f32,
    noise_lp: [f32; 2],
    k_noise: f32,
    end_lp: f32,
    k_end: f32,
    norm: f32,
}

impl Default for Tract {
    fn default() -> Self {
        Self {
            rng: Noise::default(),
            buf: vec![0.0; 2 * SECTIONS * LINE],
            pos: 0,
            delay: 10.0,
            k: [0.0; SECTIONS - 1],
            pinch: 0,
            pinch_amount: 0.0,
            wave_phase: 0.5,
            phase: 0.0,
            inc: 0.0,
            buzz: 0.0,
            gas: 0.0,
            noise_lp: [0.0; 2],
            k_noise: 0.5,
            end_lp: 0.0,
            k_end: 0.5,
            norm: 1.0,
        }
    }
}

impl Tract {
    /// The wave that entered line `line` `d` (≥ 1) samples ago, linearly
    /// interpolated. Call before this sample's [`Tract::write`].
    #[inline]
    fn read(&self, line: usize, d: f32) -> f32 {
        let base = line * LINE;
        let i = d as usize;
        let f = d - i as f32;
        let a = self.buf[base + ((self.pos + 1).wrapping_sub(i) & (LINE - 1))];
        let b = self.buf[base + (self.pos.wrapping_sub(i) & (LINE - 1))];
        a + (b - a) * f
    }

    #[inline]
    fn write(&mut self, line: usize, x: f32) {
        self.buf[line * LINE + self.pos] = flush(x * LOSS);
    }
}

impl Osc for Tract {
    fn reset(&mut self) {
        self.buf.iter_mut().for_each(|x| *x = 0.0);
        self.noise_lp = [0.0; 2];
        self.end_lp = 0.0;
    }

    fn trigger(&mut self, c: &Ctx, o: &OscCtx, rng: &mut Noise) {
        self.rng = rng.fork();
        self.wave_phase = 0.5;
        self.control(c, o);
    }

    fn control(&mut self, c: &Ctx, o: &OscCtx) {
        let fs = c.fs;
        let block_s = 16.0 / fs;
        // Quarter-wave tube: the round trip is half a period.
        self.delay = (fs / (4.0 * SECTIONS as f32 * o.hz.max(1.0))).clamp(1.0, MAX_DELAY);
        self.inc = (o.hz / fs).min(0.25);
        // The travelling ring. It enters before the tube and leaves after it,
        // so the wave rolls in and out rather than jumping.
        self.wave_phase += map::tract_wave_hz(c.y) * block_s;
        self.wave_phase -= self.wave_phase.floor();
        let centre = -0.3 + 1.6 * self.wave_phase;
        let squeeze = 0.97 * c.x * (0.7 + 0.3 * c.velocity);
        let mut area = [1.0f32; SECTIONS];
        let mut narrowest = 1.0;
        for (i, a) in area.iter_mut().enumerate() {
            let d = ((i as f32 + 0.5) / SECTIONS as f32 - centre) / RING_WIDTH;
            *a = (1.0 - squeeze * (-d * d).exp()).max(0.03);
            if *a < narrowest {
                narrowest = *a;
                self.pinch = i;
            }
        }
        for i in 0..SECTIONS - 1 {
            self.k[i] = (area[i] - area[i + 1]) / (area[i] + area[i + 1]);
        }
        self.pinch_amount = 1.0 - narrowest;
        let p = o.env.clamp(0.0, 1.0).sqrt();
        let gas = c.z;
        self.buzz = (1.0 - gas) * p;
        self.gas = gas * p;
        self.k_noise = one_pole_coeff(1.0 / (core::f32::consts::TAU * 1_800.0), fs);
        self.k_end = one_pole_coeff(1.0 / (core::f32::consts::TAU * 2_500.0), fs);
        self.norm = 0.5;
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        let d = self.delay;
        // Waves arriving at the ends of each section.
        let mut fo = [0.0f32; SECTIONS];
        let mut bo = [0.0f32; SECTIONS];
        for i in 0..SECTIONS {
            fo[i] = self.read(i, d);
            bo[i] = self.read(SECTIONS + i, d);
        }
        // Excitation at the closed end: a smooth pulse train and/or gas.
        self.phase += self.inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        let w = 0.5 - 0.5 * sin_turns(self.phase + 0.25);
        let w2 = w * w;
        let pulse = w2 * w2 * w2 - 0.2;
        // Gas: noise darkened by two one-poles (a rush, not a hiss).
        self.noise_lp[0] += (self.rng.sample() - self.noise_lp[0]) * self.k_noise;
        self.noise_lp[1] += (self.noise_lp[0] - self.noise_lp[1]) * self.k_noise;
        let noise = self.noise_lp[1] * 5.0;
        let source = 2.0 * self.buzz * pulse + self.gas * noise;

        let mut fin = [0.0f32; SECTIONS];
        let mut bin = [0.0f32; SECTIONS];
        fin[0] = CLOSED_END * bo[0] + source;
        for i in 0..SECTIONS - 1 {
            let k = self.k[i];
            fin[i + 1] = (1.0 + k) * fo[i] - k * bo[i + 1];
            bin[i] = k * fo[i] + (1.0 - k) * bo[i + 1];
        }
        // Gas squeezed through the narrowing hisses.
        fin[self.pinch] +=
            1.5 * self.pinch_amount * self.pinch_amount * (self.gas + 0.3 * self.buzz) * noise;
        // Open end: an inverting, lowpassed reflection; what's not reflected
        // radiates.
        let last = fo[SECTIONS - 1];
        self.end_lp += (last - self.end_lp) * self.k_end;
        bin[SECTIONS - 1] = OPEN_END * self.end_lp;
        // Mostly what radiates (the highpassed part), with some of the
        // pressure inside for body.
        let out = (last - self.end_lp) + 0.35 * last;

        self.pos = (self.pos + 1) & (LINE - 1);
        for i in 0..SECTIONS {
            self.write(i, fin[i]);
            self.write(SECTIONS + i, bin[i]);
        }
        if !out.is_finite() || out.abs() > 1e3 {
            self.reset();
            return 0.0;
        }
        // A pinched tube can resonate hard; round the peaks off.
        2.0 * fast_tanh(0.5 * out * self.norm)
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::*;
    use crate::params::ENGINE_TRACT;

    /// An open tube blown with gas: its first resonance sits on PITCH.
    #[test]
    fn open_tube_resonates_at_pitch() {
        let fs = 48_000.0;
        let n = 1 << 16;
        for hz in [80.0f32, 200.0, 500.0] {
            let out = run(
                ENGINE_TRACT,
                fs,
                hz,
                (0.0, 0.5, 1.0),
                1.0,
                0.2 + n as f32 / fs,
            );
            let mut re: Vec<f64> = out[out.len() - n..].iter().map(|&v| v as f64).collect();
            let mut im = vec![0.0; n];
            crate::util::fft(&mut re, &mut im, false);
            let bin_hz = fs / n as f32;
            // Strongest bin below 2× the pitch (smoothed over ±4 bins).
            let power = |b: usize| {
                (b - 4..=b + 4)
                    .map(|i| re[i] * re[i] + im[i] * im[i])
                    .sum::<f64>()
            };
            let best = (8..(2.0 * hz / bin_hz) as usize)
                .max_by(|&a, &b| power(a).total_cmp(&power(b)))
                .unwrap();
            let f = best as f32 * bin_hz;
            assert!((f / hz - 1.0).abs() < 0.06, "{hz}: {f}");
        }
    }

    /// A moving constriction makes the timbre wander: brightness varies far
    /// more over time than with the tube left alone.
    #[test]
    fn peristaltic_wave_moves_the_formants() {
        let fs = 48_000.0;
        let wander = |x: f32, y: f32| {
            let out = run(ENGINE_TRACT, fs, 90.0, (x, y, 0.3), 1.0, 2.0);
            let b: Vec<f32> = out
                .as_chunks::<2_400>()
                .0
                .iter()
                .map(|w| brightness(w, fs))
                .collect();
            let mean = b.iter().sum::<f32>() / b.len() as f32;
            (b.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / b.len() as f32).sqrt() / mean
        };
        assert!(
            wander(0.9, 0.85) > 3.0 * wander(0.0, 0.5),
            "{} {}",
            wander(0.9, 0.85),
            wander(0.0, 0.5)
        );
    }
}
