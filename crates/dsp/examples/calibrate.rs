//! Loudness survey: peak and RMS (first second) of one full-velocity hit
//! per engine across pitch, pressure-ish settings and macros, dry, at unity
//! level, with a held envelope.
//!
//! ```sh
//! cargo run -p enterokussion-dsp --release --example calibrate
//! ```

use enterokussion_dsp::Param;
use enterokussion_dsp::params::ENGINE_NAMES;
use enterokussion_dsp::voice::Voice;

fn main() {
    let fs = 48_000.0;
    println!(
        "{:<8} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "engine", "pk min", "pk med", "pk max", "rms min", "rms med", "rms max"
    );
    for (engine, name) in ENGINE_NAMES.iter().enumerate() {
        let mut peaks = Vec::new();
        let mut rmss = Vec::new();
        for pitch in [0.15f32, 0.3, 0.45, 0.6, 0.75] {
            for m in [0.0f32, 0.5, 1.0] {
                for m2 in [0.2f32, 0.8] {
                    let mut v = Voice::new(fs, 1);
                    v.set_param(Param::Engine, engine as f32);
                    v.set_param(Param::Pitch, pitch);
                    v.set_param(Param::MacroX, m);
                    v.set_param(Param::MacroY, m2);
                    v.set_param(Param::MacroZ, 1.0 - m);
                    v.set_param(Param::Attack, 0.1);
                    v.set_param(Param::Hold, 0.35);
                    v.set_param(Param::Level, 1.0);
                    v.set_param(Param::Sense, 0.5);
                    v.snap_params();
                    v.trigger(1.0);
                    let out: Vec<f32> = (0..fs as usize).map(|_| v.tick()).collect();
                    peaks.push(out.iter().fold(0.0f32, |a, &b| a.max(b.abs())));
                    rmss.push((out.iter().map(|x| x * x).sum::<f32>() / out.len() as f32).sqrt());
                }
            }
        }
        peaks.sort_by(f32::total_cmp);
        rmss.sort_by(f32::total_cmp);
        let med = |v: &[f32]| v[v.len() / 2];
        println!(
            "{:<8} {:>7.3} {:>7.3} {:>7.3} {:>7.3} {:>7.3} {:>7.3}",
            name,
            peaks[0],
            med(&peaks),
            peaks[peaks.len() - 1],
            rmss[0],
            med(&rmss),
            rmss[rmss.len() - 1]
        );
    }
}
