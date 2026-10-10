//! Engine lab: render one chamber of one engine at fixed settings and print
//! a time series of level, pitch, periodicity and brightness. Optionally
//! writes the render to a WAV file.
//!
//! ```sh
//! cargo run -p enterokussion-dsp --release --example lab -- FLATUS 90 0.5 0.3 0.4 [pressure] [seconds] [out.wav]
//! ```
//!
//! `pressure` may be a number or `swell` (0 → 1 over the first half, then
//! back down) to hear how the process responds to the envelope.
//!
//! With a WAV file instead (mono 16-bit PCM, e.g. from
//! `ffmpeg -i in.mp3 -ac 1 -ar 48000 -c:a pcm_s16le ref.wav`) it prints the
//! same analysis for a recording, to compare against:
//!
//! ```sh
//! cargo run -p enterokussion-dsp --release --example lab -- ref.wav [start_s] [seconds] [window_ms]
//! ```

use enterokussion_dsp::engines::{Ctx, Engines, OSCS, OscCtx};
use enterokussion_dsp::params::ENGINE_NAMES;
use enterokussion_dsp::util::Noise;

/// Autocorrelation pitch: the shortest lag within 8% of the best one.
fn pitch(x: &[f32], fs: f32) -> (f32, f32) {
    let n = x.len() / 2;
    let r = |lag: usize| (0..n).map(|i| x[i] * x[i + lag]).sum::<f32>();
    let r0 = r(0).max(1e-12);
    let (l0, l1) = ((fs / 4_000.0) as usize, ((fs / 18.0) as usize).min(n - 1));
    let vals: Vec<f32> = (l0..=l1).map(r).collect();
    let best = vals.iter().cloned().fold(f32::MIN, f32::max);
    let i = (1..vals.len() - 1)
        .find(|&i| vals[i] >= 0.92 * best && vals[i] >= vals[i - 1] && vals[i] >= vals[i + 1])
        .unwrap_or(0);
    (fs / (l0 + i) as f32, best / r0)
}

fn write_wav(path: &str, x: &[f32], sr: u32) -> std::io::Result<()> {
    use std::io::Write;
    let peak = x.iter().fold(1e-9f32, |a, &b| a.max(b.abs()));
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let len = x.len() as u32 * 2;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    for v in [1u16, 1] {
        f.write_all(&v.to_le_bytes())?;
    }
    f.write_all(&sr.to_le_bytes())?;
    f.write_all(&(sr * 2).to_le_bytes())?;
    for v in [2u16, 16] {
        f.write_all(&v.to_le_bytes())?;
    }
    f.write_all(b"data")?;
    f.write_all(&len.to_le_bytes())?;
    for s in x {
        f.write_all(&((s / peak * 0.9 * 32767.0) as i16).to_le_bytes())?;
    }
    f.flush()
}

fn read_wav(path: &str) -> (Vec<f32>, f32) {
    let b = std::fs::read(path).expect("read wav");
    let mut i = 12;
    let (mut sr, mut ch, mut data) = (48_000u32, 1u16, Vec::new());
    while i + 8 <= b.len() {
        let id = &b[i..i + 4];
        let len = u32::from_le_bytes(b[i + 4..i + 8].try_into().unwrap()) as usize;
        let body = &b[i + 8..(i + 8 + len).min(b.len())];
        if id == b"fmt " {
            ch = u16::from_le_bytes(body[2..4].try_into().unwrap());
            sr = u32::from_le_bytes(body[4..8].try_into().unwrap());
        } else if id == b"data" {
            data = body
                .chunks_exact(2 * ch as usize)
                .map(|f| i16::from_le_bytes([f[0], f[1]]) as f32 / 32768.0)
                .collect();
        }
        i += 8 + len + (len & 1);
    }
    (data, sr as f32)
}

/// Per-window level, pitch, periodicity, brightness, and the rate of the
/// loudness fluctuations (flaps, squeezes) over the whole stretch.
fn analyze(x: &[f32], fs: f32, win: usize) {
    let rms = |w: &[f32]| (w.iter().map(|v| v * v).sum::<f32>() / w.len() as f32).sqrt();
    println!(
        "{:>6} {:>7} {:>8} {:>6} {:>9} {:>6}",
        "t(ms)", "rms dB", "pitch", "per", "bright Hz", "crest"
    );
    for (i, w) in x.chunks_exact(win).enumerate() {
        let r = rms(w);
        if r < 1e-5 {
            println!("{:>6} {:>7}", i * win * 1000 / fs as usize, "-");
            continue;
        }
        let d = (w.windows(2).map(|p| (p[1] - p[0]).powi(2)).sum::<f32>() / w.len() as f32).sqrt();
        let bright = fs * d / (core::f32::consts::TAU * r);
        let (p, per) = pitch(&w[..win / 2], fs);
        let crest = w.iter().fold(0.0f32, |a, &b| a.max(b.abs())) / r;
        println!(
            "{:>6} {:>7.1} {:>8.1} {:>6.2} {:>9.0} {:>6.1}",
            (i * win) as f32 * 1000.0 / fs,
            20.0 * r.log10(),
            p,
            per,
            bright,
            crest
        );
    }
    // Loudness fluctuation rate: autocorrelate the 2 ms RMS envelope.
    let hop = (fs * 0.002) as usize;
    let env: Vec<f32> = x.chunks_exact(hop).map(rms).collect();
    let mean = env.iter().sum::<f32>() / env.len().max(1) as f32;
    let e: Vec<f32> = env.iter().map(|v| v - mean).collect();
    let n = e.len() / 2;
    if n > 260 {
        let r = |lag: usize| (0..n).map(|i| e[i] * e[i + lag]).sum::<f32>();
        let r0 = r(0).max(1e-12);
        // 4 … 100 Hz of envelope rate = lags of 250 … 5 hops.
        let (lag, best) = (5..250)
            .map(|l| (l, r(l)))
            .fold((0, f32::MIN), |a, b| if b.1 > a.1 { b } else { a });
        println!(
            "envelope fluctuation: depth {:.2} (std/mean), strongest period {:.1} ms ({:.1} Hz, corr {:.2})",
            (e.iter().map(|v| v * v).sum::<f32>() / e.len() as f32).sqrt() / mean.max(1e-9),
            lag as f32 * 2.0,
            500.0 / lag as f32,
            best / r0
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(path) = args.first().filter(|a| a.ends_with(".wav")) {
        let (x, fs) = read_wav(path);
        let num = |i: usize, d: f32| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
        let start = (num(1, 0.0) * fs) as usize;
        let len = (num(2, x.len() as f32 / fs) * fs) as usize;
        let win = (num(3, 100.0) * 0.001 * fs) as usize;
        let seg = &x[start.min(x.len())..(start + len).min(x.len())];
        println!("{path}: {:.2} s at {fs} Hz", seg.len() as f32 / fs);
        analyze(seg, fs, win);
        return;
    }
    let arg = |i: usize, d: &str| args.get(i).cloned().unwrap_or_else(|| d.to_string());
    let name = arg(0, "FLATUS").to_uppercase();
    let engine = ENGINE_NAMES
        .iter()
        .position(|n| *n == name)
        .expect("engine name") as u32;
    let hz: f32 = arg(1, "100").parse().unwrap();
    let (x, y, z): (f32, f32, f32) = (
        arg(2, "0.5").parse().unwrap(),
        arg(3, "0.5").parse().unwrap(),
        arg(4, "0.5").parse().unwrap(),
    );
    let pressure = arg(5, "1");
    let seconds: f32 = arg(6, "2").parse().unwrap();
    let fs = 96_000.0;
    let mut c = Ctx::new(fs);
    (c.x, c.y, c.z) = (x, y, z);
    let env = |t: f32| match pressure.as_str() {
        "swell" => {
            let u = t / seconds;
            if u < 0.5 { 2.0 * u } else { 2.0 - 2.0 * u }
        }
        p => p.parse().unwrap(),
    };
    let mut o = OscCtx {
        hz,
        env: env(0.0),
        decay_s: 2.0,
    };
    let mut e = Engines::new();
    e.trigger(engine, &c, &[o; OSCS], &mut Noise::new(7));
    let n = (seconds * fs) as usize;
    let out: Vec<f32> = (0..n)
        .map(|i| {
            if i % 16 == 0 {
                o.env = env(i as f32 / fs);
                e.control(engine, &c, &[o; OSCS]);
            }
            e.tick(engine, &[1.0, 0.0, 0.0])
        })
        .collect();
    let rms = |w: &[f32]| (w.iter().map(|v| v * v).sum::<f32>() / w.len() as f32).sqrt();
    let peak = out.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    println!(
        "{name} {hz} Hz x {x} y {y} z {z} pressure {pressure}: peak {peak:.3} rms {:.4}",
        rms(&out)
    );
    analyze(&out, fs, (fs * 0.1) as usize);
    if let Some(path) = args.get(7) {
        // Downsample 2:1 by averaging; it's for listening, not measuring.
        let half: Vec<f32> = out
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| 0.5 * (p[0] + p[1]))
            .collect();
        write_wav(path, &half, 48_000).unwrap();
        println!("wrote {path}");
    }
}
