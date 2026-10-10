//! Render every factory preset to a WAV file (a soft hit, then a hard one,
//! each as long as its envelope) and print its loudness: the sample peak and
//! the loudest 400 ms RMS (short-term loudness, used to level-match them).
//!
//! ```sh
//! cargo run -p enterokussion-dsp --release --example render -- out_dir
//! ```

use std::io::Write;

use enterokussion_dsp::params::map;
use enterokussion_dsp::{Engine, Param, presets::PRESETS};

const SR: u32 = 48_000;

fn write_wav(path: &std::path::Path, left: &[f32], right: &[f32]) -> std::io::Result<()> {
    let frames = left.len() as u32;
    let data_len = frames * 4;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&SR.to_le_bytes())?;
    f.write_all(&(SR * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for (l, r) in left.iter().zip(right) {
        for s in [l, r] {
            f.write_all(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes())?;
        }
    }
    f.flush()
}

fn main() -> std::io::Result<()> {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "renders".into()));
    std::fs::create_dir_all(&dir)?;
    for preset in PRESETS {
        let patch = preset.patch();
        let k = |p: Param| patch[p.index()];
        let delays = map::chamber_delay_seconds(k(Param::Delay2))
            .max(map::chamber_delay_seconds(k(Param::Delay3)));
        let seconds = (map::attack_seconds(k(Param::Attack))
            + map::hold_seconds(k(Param::Hold))
            + map::decay_seconds(k(Param::Decay)).min(5.0)
            + delays
            + 0.5)
            .min(24.0);
        let mut engine = Engine::new(SR as f32);
        engine.load_patch(0, &patch);
        engine.snap_params();
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut hard_from = 0;
        for velocity in [0.35, 1.0] {
            hard_from = left.len();
            engine.trigger(0, velocity);
            let (l, r) = engine.render_vec((SR as f32 * seconds) as usize);
            left.extend(l);
            right.extend(r);
        }
        let peak = left.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        // Short-term loudness of the hard hit: the loudest 400 ms.
        let hard: Vec<f32> = left[hard_from..]
            .iter()
            .zip(&right[hard_from..])
            .map(|(a, b)| 0.5 * (a + b))
            .collect();
        let win = (0.4 * SR as f32) as usize;
        let rms = hard
            .chunks(win / 4)
            .map(|c| c.iter().map(|s| s * s).sum::<f32>())
            .collect::<Vec<_>>()
            .windows(4)
            .map(|w| (w.iter().sum::<f32>() / win as f32).sqrt())
            .fold(0.0f32, f32::max);
        let name: String = preset
            .name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect();
        let path = dir.join(format!("{name}.wav"));
        write_wav(&path, &left, &right)?;
        println!(
            "{:<24} peak {:>6.3}  short-term {:>6.1} dBFS  output {:.3}  -> {}",
            preset.name,
            peak,
            20.0 * rms.max(1e-9).log10(),
            k(Param::Level),
            path.display()
        );
    }
    Ok(())
}
