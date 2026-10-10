//! The panel: two channel strips, each with an engine picker, a hit preview,
//! the three chambers, peristalsis and the envelope/sweep/fizz/chaos/filter/
//! saturator/EQ/chorus/output/reverb sections.
//!
//! Every control that RND and MUTATE can change can be locked against them
//! with Ctrl-click (⌘-click works too).

use dioxus::prelude::*;
use enterokussion_dsp::fx::Eq;
use enterokussion_dsp::params::{ENGINE_BLURBS, ENGINE_NAMES, ParamKind, macro_label, map};
use enterokussion_dsp::presets::PRESETS;
use enterokussion_dsp::{Param, VOICES, VoicePatch};

use crate::audio::AudioStatus;
use crate::state::{Drag, Synth, Target};

const STYLE: &str = include_str!("../assets/style.css");

/// Pixels of vertical drag for a knob's full travel.
const KNOB_SPAN_PX: f64 = 200.0;
/// Fader geometry; keep in sync with `.fader-track` / `.fader-cap` in the CSS.
const FADER_TRACK_PX: f64 = 150.0;
const FADER_CAP_PX: f64 = 24.0;
const FADER_LEDS: usize = 10;
/// Toggle switch geometry; keep in sync with `.switch-lever`.
const SWITCH_LEVER_PX: f64 = 14.0;
const SWITCH_LEVER_INSET_PX: f64 = 2.0;
/// Vertical space per switch position.
const SWITCH_STEP_PX: f64 = 15.0;
/// Trigger pad height; keep in sync with `.pad` in the CSS.
const PAD_PX: f64 = 96.0;
/// Hit preview: columns, the velocity it is drawn at, the longest stretch it
/// shows, and the render budget: long sounds are rendered at a lower rate so
/// a knob turn never stalls the panel.
const SCOPE_COLUMNS: usize = 180;
const SCOPE_VELOCITY: f32 = 0.8;
const SCOPE_MAX_S: f32 = 30.0;
const SCOPE_SAMPLES: f32 = 240_000.0;
const SCOPE_FS_RANGE: (f32, f32) = (8_000.0, 24_000.0);
/// EQ display: points and the rate the curve is evaluated at.
const EQ_POINTS: usize = 120;
const EQ_FS: f32 = 96_000.0;
const EQ_RANGE_DB: f32 = 20.0;

/// Keyboard map: (key, channel, velocity).
const KEYMAP: &[(Code, usize, f32)] = &[
    (Code::KeyZ, 0, 0.3),
    (Code::KeyX, 0, 0.55),
    (Code::KeyC, 0, 0.8),
    (Code::KeyV, 0, 1.0),
    (Code::KeyN, 1, 0.3),
    (Code::KeyM, 1, 0.55),
    (Code::Comma, 1, 0.8),
    (Code::Period, 1, 1.0),
];

/// Ctrl-click (or ⌘-click on a Mac, where Ctrl-click opens a context menu).
fn lock_gesture(m: Modifiers) -> bool {
    m.ctrl() || m.meta()
}

#[component]
pub fn App() -> Element {
    let synth = use_context_provider(Synth::new);
    let drag = synth.drag;

    let s_move = synth.clone();
    let s_up = synth.clone();
    let s_down = synth.clone();
    let s_key = synth.clone();
    let end_drag = move || {
        let mut drag = s_up.drag;
        if drag.peek().is_some() {
            drag.set(None);
            s_up.save();
        }
    };
    let end_drag_leave = end_drag.clone();
    let end_drag_up = end_drag;

    rsx! {
        style { {STYLE} }
        div {
            class: if drag.read().is_some() { "root dragging" } else { "root" },
            tabindex: "0",
            autofocus: true,
            // Ctrl-click is the lock gesture; keep the browser's menu out of it.
            oncontextmenu: move |e: MouseEvent| {
                if e.modifiers().ctrl() {
                    e.prevent_default();
                }
            },
            onpointerdown: move |_| s_down.audio.user_gesture(),
            onpointermove: move |e: PointerEvent| {
                let Some(d) = *s_move.drag.peek() else { return };
                e.prevent_default();
                let y = e.client_coordinates().y;
                let fine = e.modifiers().shift();
                let mut drag = s_move.drag;
                if fine != d.fine {
                    // Re-anchor so toggling SHIFT mid-drag doesn't jump.
                    drag.set(Some(Drag { start_y: y, start_value: s_move.value(d.target), fine, ..d }));
                    return;
                }
                let scale = if fine { 0.1 } else { 1.0 };
                let v = d.start_value as f64 + (d.start_y - y) / d.span_px * scale;
                s_move.set(d.target, v as f32);
            },
            onpointerup: move |_| end_drag_up(),
            onpointerleave: move |_| end_drag_leave(),
            onkeydown: move |e: KeyboardEvent| {
                let m = e.modifiers();
                if e.is_auto_repeating() || m.ctrl() || m.alt() || m.meta() {
                    return;
                }
                let code = e.code();
                if code == Code::Space {
                    e.prevent_default();
                    for v in 0..VOICES {
                        s_key.trigger(v, 1.0);
                    }
                } else if let Some(&(_, voice, vel)) = KEYMAP.iter().find(|(c, _, _)| *c == code) {
                    // Also stops a focused <select> from type-ahead jumping presets.
                    e.prevent_default();
                    s_key.trigger(voice, vel);
                } else {
                    s_key.audio.user_gesture();
                }
            },
            Header {}
            main { class: "channels",
                for v in 0..VOICES {
                    Channel { key: "{v}", voice: v }
                }
            }
            footer { class: "footer",
                span { "Keys: " }
                kbd { "Z X C V" }
                span { " channel 1 (soft → hard) · " }
                kbd { "N M , ." }
                span { " channel 2 · " }
                kbd { "Space" }
                span { " both · Drag knobs vertically (" }
                kbd { "Shift" }
                span { " = fine), scroll to nudge, double-click to reset. " }
                kbd { "Ctrl" }
                span { "-click any control to lock it: RND and MUTATE leave locked controls alone. Pads: hit higher for harder (more pressure)." }
            }
        }
    }
}

#[component]
fn Header() -> Element {
    let synth = use_context::<Synth>();
    let status = synth.audio.status.read().clone();
    let (class, text) = match status {
        AudioStatus::NeedsGesture => (
            "status warn",
            "Click anywhere or press a key to start audio".to_string(),
        ),
        AudioStatus::Starting => ("status warn", "Starting audio…".to_string()),
        AudioStatus::Running {
            sample_rate,
            detail,
        } => (
            "status ok",
            format!("{detail} · {:.1} kHz", sample_rate as f32 / 1000.0),
        ),
        AudioStatus::Failed(e) => ("status err", format!("Audio unavailable: {e}")),
    };
    rsx! {
        header { class: "header",
            div { class: "brand",
                Logo {}
                span { class: "logo", "ENTEROKUSSION" }
                span { class: "model", "EK-1" }
                span { class: "tagline", "Alien Intestinal Sound Generator" }
            }
            div { class: "{class}",
                span { class: "status-dot" }
                span { "{text}" }
            }
            div { class: "master",
                Knob { target: Target::Master, size: "sm" }
            }
        }
    }
}

/// The brand mark: a coiled length of gut (a looping trochoid), its coils
/// bunching up as if mid-squeeze.
#[component]
fn Logo() -> Element {
    let path = use_hook(|| {
        let (w, h) = (40.0f64, 26.0f64);
        let mut d = String::new();
        let n = 260;
        let loops = 4.5;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let a = t * loops * core::f64::consts::TAU;
            // A prolate cycloid: x = r·a − d·sin a, y = d·cos a, with the
            // loop size swelling and shrinking along the tube.
            let swell = 0.75 + 0.25 * (t * core::f64::consts::PI * 3.0).sin();
            let x = 3.0 + (t * (w - 9.0)) - 3.2 * swell * a.sin();
            let y = h / 2.0 + (h / 2.0 - 3.0) * swell * a.cos();
            let cmd = if i == 0 { 'M' } else { 'L' };
            d += &format!("{cmd}{x:.2} {y:.2}");
        }
        d
    });
    rsx! {
        svg { class: "brand-mark", view_box: "0 0 40 26",
            path { d: "{path}" }
        }
    }
}

/// An SVG path of the Lorenz attractor (x–z projection) scaled to `w`×`h`,
/// for the chaos modulator's glyph.
fn attractor_path(w: f64, h: f64, steps: usize) -> String {
    let (mut x, mut y, mut z) = (1.0f64, 1.0f64, 20.0f64);
    let dt = 0.008;
    let mut pts = Vec::with_capacity(steps);
    for i in 0..steps + 300 {
        let (dx, dy, dz) = (10.0 * (y - x), x * (28.0 - z) - y, x * y - 8.0 / 3.0 * z);
        x += dx * dt;
        y += dy * dt;
        z += dz * dt;
        if i >= 300 {
            pts.push((x, z));
        }
    }
    // The attractor spans about x ∈ [-20, 20], z ∈ [5, 48].
    let sx = |x: f64| (x + 21.0) / 42.0 * w;
    let sz = |z: f64| h - (z - 3.0) / 47.0 * h;
    let mut d = String::with_capacity(steps * 12);
    for (i, (x, z)) in pts.iter().enumerate() {
        let cmd = if i == 0 { 'M' } else { 'L' };
        d += &format!("{cmd}{:.1} {:.1}", sx(*x), sz(*z));
    }
    d
}

#[component]
fn Channel(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let other = (voice + 1) % VOICES;
    let s_preset = synth.clone();
    let s_rand = synth.clone();
    let s_copy = synth.clone();
    let s_unlock = synth.clone();
    let locks = synth.lock_count(voice);
    let p = |param: Param| Target::Param(voice, param);
    rsx! {
        section { class: "channel ch{voice + 1}",
            div { class: "channel-head",
                h2 { "CHANNEL {voice + 1}" }
                select {
                    class: "preset",
                    title: "Load a factory preset",
                    onchange: move |e| {
                        if let Some(p) = e.value().parse::<usize>().ok().and_then(|i| PRESETS.get(i)) {
                            s_preset.load_patch(voice, &p.patch());
                        }
                    },
                    option { value: "", disabled: true, selected: true, "Preset…" }
                    for (i, p) in PRESETS.iter().enumerate() {
                        option { key: "{i}", value: "{i}", "{p.name}" }
                    }
                }
                button {
                    class: "btn",
                    title: "Randomize this channel (locked controls stay put)",
                    onclick: move |_| s_rand.randomize(voice),
                    "RND"
                }
                button {
                    class: "btn",
                    title: "Copy this channel to channel {other + 1}",
                    onclick: move |_| s_copy.copy_voice(voice, other),
                    "→ CH{other + 1}"
                }
                button {
                    class: if locks > 0 { "btn locks on" } else { "btn locks" },
                    disabled: locks == 0,
                    title: if locks > 0 { "{locks} control(s) locked against RND and MUTATE. Click to unlock all." } else { "Ctrl-click a control to lock it against RND and MUTATE" },
                    onclick: move |_| s_unlock.clear_locks(voice),
                    LockIcon {}
                    span { "{locks}" }
                }
                Lcd { voice }
            }
            div { class: "channel-body",
                div { class: "sec sec-engine",
                    div { class: "sec-title", "ENGINE" }
                    EnginePicker { voice }
                }
                div { class: "sec sec-scope",
                    HitScope { voice }
                }
                div { class: "sec sec-voice",
                    div { class: "row faders",
                        div { class: "tune-group",
                            Knob { target: p(Param::Fine), size: "sm" }
                            Fader { voice, param: Param::Pitch }
                        }
                        Fader { voice, param: Param::Decay }
                        div { class: "macros",
                            Knob { target: p(Param::MacroX), size: "xl" }
                            Knob { target: p(Param::MacroY), size: "xl" }
                            Knob { target: p(Param::MacroZ), size: "xl" }
                        }
                    }
                }
                div { class: "sec sec-out",
                    div { class: "row",
                        Knob { target: p(Param::Level), size: "sm" }
                        Knob { target: p(Param::Sense), size: "sm" }
                        Knob { target: p(Param::Pan), size: "sm" }
                    }
                    Pad { voice }
                    div { class: "row mutate-row",
                        Knob { target: Target::Mutate(voice), size: "sm" }
                    }
                }
                div { class: "sec sec-osc",
                    div { class: "osc-group",
                        div { class: "osc-title", "CHAMBER 1" }
                        div { class: "row",
                            Knob { target: p(Param::Level1) }
                        }
                    }
                    div { class: "osc-group",
                        div { class: "osc-title", "CHAMBER 2" }
                        div { class: "row",
                            Knob { target: p(Param::Tune2) }
                            Knob { target: p(Param::Level2) }
                            Knob { target: p(Param::Length2) }
                            Knob { target: p(Param::Delay2) }
                        }
                    }
                    div { class: "osc-group",
                        div { class: "osc-title", "CHAMBER 3" }
                        div { class: "row",
                            Knob { target: p(Param::Tune3) }
                            Knob { target: p(Param::Level3) }
                            Knob { target: p(Param::Length3) }
                            Knob { target: p(Param::Delay3) }
                        }
                    }
                }
                div { class: "sec sec-peri",
                    div { class: "sec-title",
                        "PERISTALSIS"
                        PeriGlyph { voice }
                    }
                    div { class: "row",
                        Knob { target: p(Param::PeriRate) }
                        Knob { target: p(Param::PeriSqueeze) }
                        Knob { target: p(Param::PeriPitch) }
                        Knob { target: p(Param::PeriIrreg) }
                        Knob { target: p(Param::PeriTravel) }
                    }
                }
                div { class: "sec sec-amp",
                    div { class: "sec-title", "ENVELOPE · SWEEP" }
                    div { class: "row",
                        Knob { target: p(Param::Attack) }
                        Knob { target: p(Param::Curve) }
                        Knob { target: p(Param::Hold) }
                        div { class: "sweep-group",
                            Knob { target: p(Param::SweepSpeed) }
                            Knob { target: p(Param::SweepDepth) }
                        }
                    }
                }
                div { class: "sec sec-fizz",
                    div { class: "sec-title", "FIZZ" }
                    div { class: "row",
                        Knob { target: p(Param::FizzLevel) }
                        Knob { target: p(Param::FizzTone) }
                        Knob { target: p(Param::FizzCrackle) }
                    }
                }
                div { class: "sec sec-chaos",
                    div { class: "sec-title",
                        "CHAOS"
                        ChaosGlyph { voice }
                    }
                    div { class: "row",
                        Knob { target: p(Param::ChaosRate) }
                        Knob { target: p(Param::ChaosDepth) }
                        Switch { voice, param: Param::ChaosDest }
                        Knob { target: p(Param::Jitter) }
                    }
                }
                div { class: "sec sec-filter",
                    div { class: "sec-title", "LADDER FILTER" }
                    div { class: "row",
                        Knob { target: p(Param::Cutoff) }
                        Knob { target: p(Param::Reso) }
                        Knob { target: p(Param::FilterEnv) }
                        Switch { voice, param: Param::FilterType }
                    }
                }
                div { class: "sec sec-sat",
                    div { class: "sec-title", "SATURATOR" }
                    div { class: "row",
                        Knob { target: p(Param::Drive) }
                        Knob { target: p(Param::SatMix) }
                        Switch { voice, param: Param::SatType }
                        Switch { voice, param: Param::SatPos }
                    }
                }
                div { class: "sec sec-chorus",
                    div { class: "sec-title", "CHORUS" }
                    div { class: "row",
                        Knob { target: p(Param::ChorusMix) }
                        Knob { target: p(Param::ChorusRate) }
                        Knob { target: p(Param::ChorusDepth) }
                        Knob { target: p(Param::ChorusXover) }
                    }
                }
                div { class: "sec sec-eq",
                    div { class: "sec-title", "EQ" }
                    div { class: "eq-body",
                        div { class: "row eq-knobs",
                            Knob { target: p(Param::LowCut) }
                            div { class: "eq-band",
                                Knob { target: p(Param::LowFreq), size: "sm" }
                                Knob { target: p(Param::LowGain) }
                            }
                            div { class: "eq-band",
                                Knob { target: p(Param::MidFreq), size: "sm" }
                                Knob { target: p(Param::MidGain) }
                                Knob { target: p(Param::MidQ), size: "sm" }
                            }
                            div { class: "eq-band",
                                Knob { target: p(Param::HighFreq), size: "sm" }
                                Knob { target: p(Param::HighGain) }
                            }
                        }
                        EqCurve { voice }
                    }
                }
                div { class: "sec sec-reverb",
                    div { class: "sec-title", "REVERB" }
                    div { class: "row",
                        Knob { target: p(Param::ReverbMix) }
                        Knob { target: p(Param::ReverbDecay) }
                        Knob { target: p(Param::ReverbTone) }
                        Knob { target: p(Param::ReverbPredelay) }
                    }
                }
            }
        }
    }
}

fn engine_of(patch: &VoicePatch) -> u32 {
    Param::Engine.sanitize(patch[Param::Engine.index()]) as u32
}

#[component]
fn LockIcon() -> Element {
    rsx! {
        svg { class: "lock-icon", view_box: "0 0 10 12",
            path { d: "M2.5 5.5V3.6a2.5 2.5 0 0 1 5 0v1.9" }
            rect { x: "1", y: "5.5", width: "8", height: "6", rx: "1.2" }
        }
    }
}

/// Small read-out showing the engine and the last touched control.
#[component]
fn Lcd(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let state = synth.state.read();
    let patch = &state.voices[voice];
    let engine = engine_of(patch);
    let line2 = match synth.touched.read()[voice] {
        Some(p) => format!(
            "{}{} {}",
            if state.locks[voice][p.index()] {
                "[LOCKED] "
            } else {
                ""
            },
            p.name_for(engine).to_uppercase(),
            p.display_for(engine, patch[p.index()])
        ),
        None => format!(
            "PITCH {} · ATTACK {} · DECAY {}",
            Param::Pitch.display(patch[Param::Pitch.index()]),
            Param::Attack.display(patch[Param::Attack.index()]),
            Param::Decay.display(patch[Param::Decay.index()])
        ),
    };
    rsx! {
        div { class: "lcd",
            div { class: "lcd-line", "ENGINE {ENGINE_NAMES[engine as usize]}" }
            div { class: "lcd-line", "{line2}" }
        }
    }
}

/// Eight engine buttons with glyphs.
#[component]
fn EnginePicker(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let current = engine_of(&synth.state.read().voices[voice]) as usize;
    let locked = synth.is_locked(Target::Param(voice, Param::Engine));
    rsx! {
        div {
            class: if locked { "engines locked" } else { "engines" },
            role: "radiogroup",
            "aria-label": "Engine",
            for (i, name) in ENGINE_NAMES.iter().enumerate() {
                EngineButton { key: "{i}", voice, index: i, name, selected: i == current, locked: locked && i == current }
            }
        }
        div { class: "engine-blurb", "{ENGINE_BLURBS[current]}" }
    }
}

#[component]
fn EngineButton(
    voice: usize,
    index: usize,
    name: &'static str,
    selected: bool,
    locked: bool,
) -> Element {
    let synth = use_context::<Synth>();
    rsx! {
        button {
            class: "engine",
            class: if selected { "sel" },
            class: if locked { "locked" },
            role: "radio",
            "aria-checked": "{selected}",
            title: "{ENGINE_BLURBS[index]} (Ctrl-click: lock the engine)",
            onclick: move |e: MouseEvent| {
                if lock_gesture(e.modifiers()) {
                    synth.toggle_lock(Target::Param(voice, Param::Engine));
                    return;
                }
                synth.set_param(voice, Param::Engine, index as f32);
                synth.save();
            },
            svg { class: "engine-glyph", view_box: "0 0 24 24",
                path { d: "{engine_glyph(index)}" }
            }
            span { "{name}" }
            if locked {
                LockIcon {}
            }
        }
    }
}

/// Line-art glyph per engine, in a 24×24 box.
fn engine_glyph(engine: usize) -> &'static str {
    match engine {
        // GURGLE: bubbles of three sizes, rising.
        0 => {
            "M4 18a3.5 3.5 0 1 0 7 0a3.5 3.5 0 1 0-7 0M13 11a2.6 2.6 0 1 0 5.2 0a2.6 2.6 0 1 0-5.2 0\
             M8 5.5a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0-3.2 0M18 19a1.2 1.2 0 1 0 2.4 0a1.2 1.2 0 1 0-2.4 0"
        }
        // FLATUS: a valve flapping out puffs that sputter away.
        1 => "M2 12c1-6 2-6 3 0s2 6 3 0 1.5-5 2.5 0 1.5 4 2.5 0 1-3 2 0M16 12h1M19 11h.8M21 13h.6",
        // GROWL: rough pulses under a formant hump.
        2 => {
            "M2 19L4 9L5 19L7 8L8 19L10.5 10L11.5 19L13 7L14 19L17 10L18 19L20 11L21 19M3 6Q8 1 13 4T22 5"
        }
        // SQUEAK: a tight, bright squeal.
        3 => {
            "M2 12c.75-4 1.5-4 2.25 0s1.5 4 2.25 0 1.5-4 2.25 0 1.5 4 2.25 0 1.5-4 2.25 0 1.5 4 2.25 0 1.5-4 2.25 0 1.5 4 2.25 0"
        }
        // TRACT: a tube pinched by a travelling ring.
        4 => {
            "M2 7C9 7 10 10.5 13 10.5S17 7 22 7M2 17C9 17 10 13.5 13 13.5S17 17 22 17M5 12h3M17 12h2"
        }
        // SYMBIO: two organisms, entangled.
        5 => {
            "M3 12a4.5 4.5 0 1 0 9 0a4.5 4.5 0 1 0-9 0M10 12a4.5 4.5 0 1 0 9 0a4.5 4.5 0 1 0-9 0M20 7l2-2M20 17l2 2"
        }
        // FLORA: little critters with wriggling tails.
        6 => {
            "M5 7a2 2 0 1 0 4 0a2 2 0 1 0-4 0M9 7c2 0 2 2 4 2M14 15a2 2 0 1 0 4 0a2 2 0 1 0-4 0\
             M18 15c2 0 2-2 4-2M3 17a1.4 1.4 0 1 0 2.8 0a1.4 1.4 0 1 0-2.8 0M5.8 17c1.5 0 1.5 1.5 3 1.5"
        }
        // RUMBLE: a low, uneven roll.
        _ => "M2 12c2-5 3 4 5 0s2-6 4 0 2 5 3.5 0 2.5-4 4 0 2 3 3.5 0M3 18c3-2 5 1 8 0s5-1 8 0",
    }
}

/// Offline-rendered outline of one hit of the current patch.
#[component]
fn HitScope(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let hits = synth.hits.read()[voice];
    let state = synth.state;
    let outline = use_memo(move || {
        let patch = state.read().voices[voice];
        let seconds = enterokussion_dsp::preview::hit_seconds(&patch).clamp(0.25, SCOPE_MAX_S);
        let fs = (SCOPE_SAMPLES / seconds).clamp(SCOPE_FS_RANGE.0, SCOPE_FS_RANGE.1);
        let cols = enterokussion_dsp::preview::hit_outline(
            &patch,
            SCOPE_VELOCITY,
            fs,
            seconds,
            SCOPE_COLUMNS,
        );
        (cols, seconds)
    });
    let (cols, seconds) = &*outline.read();
    let peak = cols
        .iter()
        .fold(0.0f32, |a, &(lo, hi)| a.max(lo.abs()).max(hi.abs()));
    let scale = 1.0 / peak.max(1e-3);
    let (w, h) = (SCOPE_COLUMNS as f32, 60.0f32);
    let mut d = String::with_capacity(SCOPE_COLUMNS * 24);
    for (i, &(_, hi)) in cols.iter().enumerate() {
        let cmd = if i == 0 { 'M' } else { 'L' };
        d += &format!("{cmd}{i} {:.1}", h / 2.0 - hi * scale * h * 0.46);
    }
    for (i, &(lo, _)) in cols.iter().enumerate().rev() {
        d += &format!("L{i} {:.1}", h / 2.0 - lo * scale * h * 0.46);
    }
    d += "Z";
    let peak_db = if peak > 1e-5 {
        format!("{:+.0} dB", 20.0 * peak.log10())
    } else {
        "silent".into()
    };
    let time = if *seconds >= 1.0 {
        format!("{seconds:.1} s")
    } else {
        format!("{:.0} ms", seconds * 1000.0)
    };
    rsx! {
        div { class: "scope",
            div { class: "sec-title scope-title",
                "HIT"
                span { class: "scope-meta", "{time} · peak {peak_db}" }
            }
            svg {
                class: "scope-svg",
                view_box: "0 0 {w} {h}",
                preserve_aspect_ratio: "none",
                line { class: "scope-axis", x1: "0", y1: "{h / 2.0}", x2: "{w}", y2: "{h / 2.0}" }
                // Re-keyed on every hit so the flash animation replays.
                for k in std::iter::once(hits) {
                    path { key: "{k}", class: if k > 0 { "scope-wave played" } else { "scope-wave" }, d: "{d}" }
                }
            }
        }
    }
}

/// The EQ's magnitude response, 20 Hz … 20 kHz, ±20 dB, computed by the same
/// filter code the voice runs.
#[component]
fn EqCurve(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let state = synth.state;
    let path = use_memo(move || {
        let s = state.read();
        let patch = &s.voices[voice];
        let k = |p: Param| patch[p.index()];
        let mut eq = Eq::default();
        eq.update(
            [
                k(Param::LowCut),
                k(Param::LowFreq),
                k(Param::LowGain),
                k(Param::MidFreq),
                k(Param::MidGain),
                k(Param::MidQ),
                k(Param::HighFreq),
                k(Param::HighGain),
            ],
            EQ_FS,
        );
        let mut d = String::with_capacity(EQ_POINTS * 14);
        for i in 0..EQ_POINTS {
            let t = i as f32 / (EQ_POINTS - 1) as f32;
            let f = 20.0 * 1000f32.powf(t);
            let db = 20.0 * eq.magnitude(f, EQ_FS).max(1e-6).log10();
            let y = 30.0 - (db / EQ_RANGE_DB).clamp(-1.2, 1.2) * 26.0;
            let cmd = if i == 0 { 'M' } else { 'L' };
            d += &format!("{cmd}{:.1} {y:.1}", t * 200.0);
        }
        d
    });
    // Decade lines at 100 Hz, 1 kHz and 10 kHz.
    let decades = [100.0f32, 1_000.0, 10_000.0].map(|f| (f / 20.0).log10() / 3.0 * 200.0);
    rsx! {
        svg {
            class: "eq-svg",
            view_box: "0 0 200 60",
            preserve_aspect_ratio: "none",
            for (i, x) in decades.iter().enumerate() {
                line { key: "{i}", class: "eq-grid", x1: "{x}", y1: "0", x2: "{x}", y2: "60" }
            }
            line { class: "eq-zero", x1: "0", y1: "30", x2: "200", y2: "30" }
            path { class: "eq-line", d: "{path}" }
        }
    }
}

/// A ring next to the PERISTALSIS title that contracts at the peristaltic
/// rate while SQUEEZE or PITCH is up.
#[component]
fn PeriGlyph(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let state = synth.state.read();
    let patch = &state.voices[voice];
    let active = patch[Param::PeriSqueeze.index()] > 0.0
        || map::peri_semitones(patch[Param::PeriPitch.index()]).abs() > 0.05;
    let period = 1.0 / map::peri_hz(patch[Param::PeriRate.index()]);
    rsx! {
        svg { class: if active { "peri-glyph on" } else { "peri-glyph" }, view_box: "0 0 30 16",
            path { class: "peri-tube", d: "M1 4H29M1 12H29" }
            ellipse {
                class: "peri-ring",
                cx: "15",
                cy: "8",
                rx: "3",
                ry: "6",
                style: "animation-duration: {period:.3}s",
            }
        }
    }
}

/// Animated attractor next to the CHAOS title; moves at the chaos rate when
/// the modulator has depth.
#[component]
fn ChaosGlyph(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let path = use_hook(|| attractor_path(30.0, 16.0, 700));
    let state = synth.state.read();
    let patch = &state.voices[voice];
    let active = patch[Param::ChaosDepth.index()] > 0.0;
    // One trip around the drawn path covers roughly ten orbits.
    let period = 10.0 / map::chaos_hz(patch[Param::ChaosRate.index()]);
    rsx! {
        svg { class: if active { "chaos-glyph on" } else { "chaos-glyph" }, view_box: "0 0 30 16",
            path { class: "chaos-trail", d: "{path}" }
            path {
                class: "chaos-comet",
                d: "{path}",
                "pathLength": "100",
                style: "animation-duration: {period:.2}s",
            }
        }
    }
}

fn polar(cx: f64, cy: f64, r: f64, deg: f64) -> (f64, f64) {
    let a = deg.to_radians();
    (cx + r * a.sin(), cy - r * a.cos())
}

fn arc_path(cx: f64, cy: f64, r: f64, from_deg: f64, to_deg: f64) -> String {
    let (x0, y0) = polar(cx, cy, r, from_deg);
    let (x1, y1) = polar(cx, cy, r, to_deg);
    let large = if (to_deg - from_deg).abs() > 180.0 {
        1
    } else {
        0
    };
    format!("M {x0:.2} {y0:.2} A {r} {r} 0 {large} 1 {x1:.2} {y1:.2}")
}

/// Pointer down on a knob or fader: Ctrl toggles its lock, anything else
/// starts a drag.
fn begin_drag(synth: &Synth, target: Target, e: &PointerEvent, span_px: f64) {
    e.prevent_default();
    synth.audio.user_gesture();
    if lock_gesture(e.modifiers()) {
        synth.toggle_lock(target);
        return;
    }
    if let Target::Param(v, p) = target {
        let mut touched = synth.touched;
        touched.write()[v] = Some(p);
    }
    let mut drag = synth.drag;
    drag.set(Some(Drag {
        target,
        start_y: e.client_coordinates().y,
        start_value: synth.value(target),
        span_px,
        fine: e.modifiers().shift(),
    }));
}

fn nudge(synth: &Synth, target: Target, e: &WheelEvent) {
    e.prevent_default();
    let dy = e.delta().strip_units().y;
    if dy == 0.0 {
        return;
    }
    let step = if e.modifiers().shift() { 0.002 } else { 0.02 };
    let v = synth.value(target) - (dy.signum() as f32) * step;
    synth.set(target, v);
    synth.save();
}

/// (label, descriptive name) for a control, resolving macro knobs to the
/// channel's current engine.
fn target_info(synth: &Synth, target: Target) -> (String, String) {
    match target {
        Target::Param(v, p) => {
            let engine = engine_of(&synth.state.read().voices[v]);
            (
                p.label_for(engine).to_string(),
                p.name_for(engine).to_string(),
            )
        }
        Target::Master => ("MASTER".into(), "Master volume".into()),
        Target::Mutate(_) => ("MUTATE".into(), "Mutate on trigger".into()),
    }
}

fn display(synth: &Synth, target: Target, value: f32) -> String {
    match target {
        Target::Param(v, p) => p.display_for(engine_of(&synth.state.read().voices[v]), value),
        Target::Master => Param::Level.display(value),
        Target::Mutate(_) => crate::state::mutate_display(value),
    }
}

/// Tooltip suffix explaining the lock gesture where it applies.
fn lock_hint(target: Target, locked: bool) -> &'static str {
    match (target.lockable(), locked) {
        (None, _) => "",
        (Some(_), true) => " · LOCKED (Ctrl-click to unlock)",
        (Some(_), false) => " · Ctrl-click to lock",
    }
}

#[component]
fn Knob(target: Target, #[props(default = "lg")] size: &'static str) -> Element {
    let synth = use_context::<Synth>();
    let value = synth.value(target);
    let (label, name) = target_info(&synth, target);
    let locked = synth.is_locked(target);
    let active = matches!(*synth.drag.read(), Some(d) if d.target == target);
    let angle = -135.0 + 270.0 * value as f64;
    let (px, py) = polar(30.0, 30.0, 15.0, angle);
    let track = arc_path(30.0, 30.0, 26.0, -135.0, 135.0);
    // Bipolar controls light the arc from the centre.
    let bipolar = match target {
        Target::Param(v, p) => p.is_bipolar_for(engine_of(&synth.state.read().voices[v])),
        _ => false,
    };
    let (a0, a1) = if bipolar {
        (angle.min(0.0), angle.max(0.0))
    } else {
        (-135.0, angle)
    };
    let lit = if (a1 - a0).abs() > 0.5 {
        arc_path(30.0, 30.0, 26.0, a0, a1)
    } else {
        String::new()
    };
    let text = display(&synth, target, value);
    let is_macro = matches!(target, Target::Param(_, p) if p.macro_slot().is_some());
    let hint = lock_hint(target, locked);
    let s_down = synth.clone();
    let s_dbl = synth.clone();
    let s_wheel = synth.clone();
    rsx! {
        div {
            class: "knob knob-{size}",
            class: if active { "active" },
            class: if is_macro { "macro" },
            class: if locked { "locked" },
            div { class: "ctl-label",
                "{label}"
                if locked {
                    LockIcon {}
                }
            }
            svg {
                class: "knob-svg",
                view_box: "0 0 60 60",
                role: "slider",
                "aria-label": "{name}",
                "aria-valuetext": "{text}",
                onpointerdown: move |e| begin_drag(&s_down, target, &e, KNOB_SPAN_PX),
                ondoubleclick: move |e: MouseEvent| {
                    if !lock_gesture(e.modifiers()) {
                        s_dbl.reset(target);
                    }
                },
                onwheel: move |e| nudge(&s_wheel, target, &e),
                title { "{name}: {text}{hint}" }
                path { class: "knob-track", d: "{track}" }
                if !lit.is_empty() {
                    path { class: "knob-lit", d: "{lit}" }
                }
                circle { class: "knob-skirt", cx: "30", cy: "30", r: "21" }
                circle { class: "knob-body", cx: "30", cy: "30", r: "17" }
                line { class: "knob-pointer", x1: "30", y1: "30", x2: "{px:.2}", y2: "{py:.2}" }
            }
            div { class: "ctl-value", "{text}" }
        }
    }
}

#[component]
fn Fader(voice: usize, param: Param) -> Element {
    let synth = use_context::<Synth>();
    let target = Target::Param(voice, param);
    let value = synth.value(target);
    let info = param.info();
    let text = param.display(value);
    let locked = synth.is_locked(target);
    let hint = lock_hint(target, locked);
    let active = matches!(*synth.drag.read(), Some(d) if d.target == target);
    let travel = FADER_TRACK_PX - FADER_CAP_PX;
    let cap_top = (1.0 - value as f64) * travel;
    let s_down = synth.clone();
    let s_dbl = synth.clone();
    let s_wheel = synth.clone();
    rsx! {
        div {
            class: "fader",
            class: if active { "active" },
            class: if locked { "locked" },
            div { class: "ctl-label",
                "{info.label}"
                if locked {
                    LockIcon {}
                }
            }
            div {
                class: "fader-body",
                role: "slider",
                title: "{info.name}: {text}{hint}",
                "aria-label": "{info.name}",
                "aria-valuetext": "{text}",
                onpointerdown: move |e| begin_drag(&s_down, target, &e, travel),
                ondoubleclick: move |e: MouseEvent| {
                    if !lock_gesture(e.modifiers()) {
                        s_dbl.reset(target);
                    }
                },
                onwheel: move |e| nudge(&s_wheel, target, &e),
                div { class: "fader-leds",
                    for i in (0..FADER_LEDS).rev() {
                        div {
                            key: "{i}",
                            class: if value * FADER_LEDS as f32 >= i as f32 + 0.5 { "led on" } else { "led" },
                        }
                    }
                }
                div { class: "fader-track",
                    div { class: "fader-slot" }
                    div { class: "fader-cap", style: "top: {cap_top:.1}px" }
                }
            }
            div { class: "ctl-value", "{text}" }
        }
    }
}

/// Multi-position slide switch for choice parameters; click a position to
/// select it, or the slot to step, Ctrl-click anywhere to lock it. The chaos
/// destination names the current engine's macros.
#[component]
fn Switch(voice: usize, param: Param) -> Element {
    let synth = use_context::<Synth>();
    let ParamKind::Choice(names) = param.info().kind else {
        return rsx! {};
    };
    let engine = engine_of(&synth.state.read().voices[voice]);
    let names: Vec<String> = names
        .iter()
        .enumerate()
        .map(|(i, n)| match (param, i) {
            (Param::ChaosDest, 1..=3) => macro_label(engine, i - 1).to_string(),
            _ => n.to_string(),
        })
        .collect();
    let target = Target::Param(voice, param);
    let current = synth.value(target) as usize;
    let locked = synth.is_locked(target);
    let hint = lock_hint(target, locked);
    let n = names.len();
    let slot_px = SWITCH_STEP_PX * n as f64 + 4.0;
    let lever = if n > 1 {
        current as f64 / (n - 1) as f64
    } else {
        0.0
    };
    let lever_px =
        SWITCH_LEVER_INSET_PX + lever * (slot_px - SWITCH_LEVER_PX - 2.0 * SWITCH_LEVER_INSET_PX);
    let s_cycle = synth.clone();
    rsx! {
        div { class: if locked { "switch locked" } else { "switch" },
            div { class: "ctl-label",
                "{param.info().label}"
                if locked {
                    LockIcon {}
                }
            }
            div { class: "switch-body",
                div {
                    class: "switch-slot",
                    style: "height: {slot_px}px",
                    title: "{param.info().name}{hint}",
                    onclick: move |e: MouseEvent| {
                        if lock_gesture(e.modifiers()) {
                            s_cycle.toggle_lock(target);
                            return;
                        }
                        s_cycle.set_param(voice, param, ((current + 1) % n) as f32);
                        s_cycle.save();
                    },
                    div { class: "switch-lever", style: "top: {lever_px:.1}px" }
                }
                div { class: "switch-labels", style: "height: {slot_px}px",
                    for (i, name) in names.into_iter().enumerate() {
                        SwitchLabel { key: "{i}", voice, param, index: i, name, selected: i == current }
                    }
                }
            }
        }
    }
}

#[component]
fn SwitchLabel(voice: usize, param: Param, index: usize, name: String, selected: bool) -> Element {
    let synth = use_context::<Synth>();
    rsx! {
        div {
            class: if selected { "switch-label sel" } else { "switch-label" },
            onclick: move |e: MouseEvent| {
                if lock_gesture(e.modifiers()) {
                    synth.toggle_lock(Target::Param(voice, param));
                    return;
                }
                synth.set_param(voice, param, index as f32);
                synth.save();
            },
            "{name}"
        }
    }
}

#[component]
fn Pad(voice: usize) -> Element {
    let synth = use_context::<Synth>();
    let hits = synth.hits.read()[voice];
    rsx! {
        div {
            class: "pad",
            role: "button",
            title: "Trigger (hit higher for a harder hit)",
            onpointerdown: move |e: PointerEvent| {
                e.prevent_default();
                let y = e.element_coordinates().y;
                let velocity = (1.0 - y / PAD_PX).clamp(0.05, 1.0) as f32;
                synth.trigger(voice, velocity);
            },
            for h in std::iter::once(hits) {
                span { key: "{h}", class: if h > 0 { "led trig-led flash" } else { "led trig-led" } }
            }
            span { class: "pad-label", "STRIKE" }
            span { class: "pad-hint", "hard" }
            span { class: "pad-hint soft", "soft" }
        }
    }
}
