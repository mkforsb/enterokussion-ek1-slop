//! Enterokussion DSP: a dual-channel alien intestinal sound generator. Every
//! channel runs three freely tuned chambers of one of eight digestive
//! processes (bubbling fluid, a sphincter valve, a growling throat,
//! stick-slip friction, a peristaltic tube, coupled organisms, a swarm of
//! gut flora and a rumbling gut), each with a long, shapeable envelope and
//! squeezed by a peristaltic wave, plus a fizz layer, a saturator, a
//! Moog-style ladder filter, a four-band EQ, a stereo chorus and a plate
//! reverb.
//!
//! The crate has no dependencies and no platform code so it can be compiled to
//! a standalone wasm module for an AudioWorklet as well as used natively.

// The engines loop over several parallel per-oscillator arrays at once;
// index loops read better there than zipped iterators.
#![allow(clippy::needless_range_loop)]

pub mod engine;
pub mod engines;
pub mod envelope;
pub mod fizz;
pub mod fx;
pub mod modulation;
pub mod params;
pub mod presets;
pub mod preview;
pub mod random;
pub mod reverb;
pub mod util;
pub mod voice;

pub use engine::{Engine, VOICES};
pub use params::{ALL_PARAMS, PARAM_COUNT, Param, ParamKind, VoicePatch};
