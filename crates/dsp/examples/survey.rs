//! What does RND give? Rolls `N` patches exactly as RND (or MUTATE at RND)
//! does, plays each at full velocity through the whole engine (chorus and
//! reverb included) and reports how long they last, how slowly they arrive,
//! how loud and how bright they are, overall and per engine.
//!
//! ```sh
//! cargo run -p enterokussion-dsp --release --example survey -- 300 [seed]
//! ```

use enterokussion_dsp::params::{ENGINE_NAMES, default_patch};
use enterokussion_dsp::random::{NO_LOCKS, random_patch};
use enterokussion_dsp::util::{Noise, fft};
use enterokussion_dsp::{Engine, Param};

const SR: f32 = 24_000.0;
const SECONDS: f32 = 16.0;

struct Hit {
    engine: usize,
    /// Seconds until the short-term level first comes within 3 dB of its peak.
    onset: f32,
    /// Seconds until it falls 40 dB below its peak for good.
    length: f32,
    centroid: f64,
    /// Peak short-term (50 ms) level, dBFS.
    loudness: f32,
}

fn analyze(x: &[f32], engine: usize) -> Hit {
    let n = x.len().next_power_of_two();
    let mut re: Vec<f64> = x.iter().map(|&v| v as f64).collect();
    re.resize(n, 0.0);
    let mut im = vec![0.0; n];
    fft(&mut re, &mut im, false);
    let (mut total, mut moment) = (0.0, 0.0);
    for bin in 1..n / 2 {
        let f = bin as f64 * SR as f64 / n as f64;
        let p = re[bin] * re[bin] + im[bin] * im[bin];
        total += p;
        moment += p * f;
    }
    let hop = (0.01 * SR) as usize;
    let st: Vec<f32> = x
        .chunks(hop)
        .map(|c| c.iter().map(|v| v * v).sum::<f32>() / c.len() as f32)
        .collect::<Vec<_>>()
        .windows(5)
        .map(|w| w.iter().sum::<f32>() / 5.0)
        .collect();
    let peak = st.iter().cloned().fold(0.0f32, f32::max);
    let onset = st.iter().position(|&e| e > peak * 0.5).unwrap_or(0);
    let last = st.iter().rposition(|&e| e > peak * 1e-4).unwrap_or(0);
    Hit {
        engine,
        onset: onset as f32 * hop as f32 / SR,
        length: (last + 5) as f32 * hop as f32 / SR,
        centroid: moment / total.max(1e-30),
        loudness: 10.0 * peak.max(1e-12).log10(),
    }
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q) as usize]
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let seed: u32 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mut rng = Noise::new(seed);
    let mut hits = Vec::new();
    for _ in 0..n {
        let patch = random_patch(&default_patch(), &NO_LOCKS, &mut rng);
        let mut e = Engine::new(SR);
        e.load_patch(0, &patch);
        e.snap_params();
        e.trigger(0, 1.0);
        let (l, r) = e.render_vec((SR * SECONDS) as usize);
        let mono: Vec<f32> = l.iter().zip(&r).map(|(a, b)| 0.5 * (a + b)).collect();
        hits.push(analyze(&mono, patch[Param::Engine.index()] as usize));
    }
    let col = |f: &dyn Fn(&Hit) -> f64| hits.iter().map(f).collect::<Vec<f64>>();
    let show = |name: &str, mut v: Vec<f64>, unit: &str| {
        println!(
            "{name:<22} p10 {:>8.2}  p25 {:>8.2}  median {:>8.2}  p75 {:>8.2}  p90 {:>8.2} {unit}",
            pct(&mut v, 0.1),
            pct(&mut v, 0.25),
            pct(&mut v, 0.5),
            pct(&mut v, 0.75),
            pct(&mut v, 0.9)
        );
    };
    println!("{n} random hits (seed {seed}), full velocity, whole engine");
    show("time to peak", col(&|h| h.onset as f64), "s");
    show("length to -40 dB", col(&|h| h.length as f64), "s");
    show("spectral centroid", col(&|h| h.centroid), "Hz");
    show("peak loudness", col(&|h| h.loudness as f64), "dBFS");
    println!(
        "\nper engine: n / median time to peak / median length / median centroid / median loudness"
    );
    for (e, name) in ENGINE_NAMES.iter().enumerate() {
        let sel: Vec<&Hit> = hits.iter().filter(|h| h.engine == e).collect();
        if sel.is_empty() {
            continue;
        }
        let med = |f: &dyn Fn(&&Hit) -> f64| pct(&mut sel.iter().map(f).collect::<Vec<_>>(), 0.5);
        println!(
            "  {name:<7} n={:<3} {:>6.2} s {:>6.2} s {:>7.0} Hz {:>6.1} dBFS",
            sel.len(),
            med(&|h| h.onset as f64),
            med(&|h| h.length as f64),
            med(&|h| h.centroid),
            med(&|h| h.loudness as f64),
        );
    }
}
