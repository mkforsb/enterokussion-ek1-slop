# Enterokussion EK-1

A two-channel **alien intestinal sound generator**: growls, grumbles,
gurgles, squelches, squeaks, rumbles and, yes, farts. Lots of farts. It is a
gut-flora descendant of [Xenokussion XK-1](https://github.com/mkforsb/xenokussion-xk1-slop)
by way of [Subkussion SB-2](https://github.com/mkforsb/subkussion-sb2-slop),
and keeps their frame: two channels, a trigger pad per channel, a plate
reverb per channel, MUTATE, RND and locks. The percussion is gone. Instead,
every channel runs one of **eight digestive processes** in **three chambers**,
each with an envelope that can take anything from a millisecond to over a
minute, squeezed by a **peristaltic wave** that rolls from chamber to chamber.

It's written in Rust with a [Dioxus](https://dioxuslabs.com) UI and builds for
two targets from one codebase:

| Target | Audio | How |
| --- | --- | --- |
| Web (wasm) | WebAudio `AudioWorklet` | `make serve-web` |
| Native Linux | PulseAudio (also PipeWire's pulse server) | `make run-desktop` |

## Building

Requirements:

- Rust (stable) with `rustup target add wasm32-unknown-unknown`
- Dioxus CLI 0.7: `cargo binstall dioxus-cli` or grab `dx` from the Dioxus releases
- Desktop only (Debian/Ubuntu package names): `libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libpulse-dev pkg-config`

```sh
make serve-web      # http://127.0.0.1:8080, click once to enable audio
make run-desktop    # native window
make web desktop    # release bundles in target/dx/enterokussion/release/
make test lint
make renders        # offline-render every preset to ./renders/*.wav
make bench          # engine speed as a real-time factor
make calibrate      # loudness survey of every engine
make survey         # what RND gives: onset, length, brightness, loudness
```

For the desktop build, `ENTEROKUSSION_LATENCY_MS` (default 20) sets the
PulseAudio target latency.

## What EK-1 changes

Compared with SB-2, the frame it was grown from:

- **Eight new engines**, each an alien digestive process rather than an
  oscillator: bubbling fluid, a sphincter valve, a growling throat,
  stick-slip friction, a peristaltic tube, symbiotic organisms, a swarm of
  gut flora and a rumbling gut (see [The engines](#the-engines)).
- **Engines are driven, not just enveloped.** Each chamber's envelope,
  times the peristaltic squeeze, times the velocity, is the chamber's
  *pressure*, and the process engines respond to it physically. A valve only
  buzzes above a threshold and sags in pitch as the pressure falls, friction
  only slips while it's pushed, bubbles only form while there's gas. A long
  swell, a sagging tail or a squeeze changes *what* happens, not just how
  loud.
- **Envelopes for slow things:**

  | | SB-2 | EK-1 |
  | --- | --- | --- |
  | ATTACK | 0.5 – 100 ms | **1 ms – 10 s**, with a CURVE from a slow swell to a fast snap |
  | HOLD | 0 – 0.8 s | **0 – 10 s** |
  | DECAY | 30 ms – 6 s | **30 ms – 20 s** |
  | Per chamber | decay ×0.06 … ×2 | **LENGTH** ×0.125 … ×4 of the whole envelope, and an onset **DELAY** of up to 3 s |
  | SWEEP time | 2 ms – 1 s | **2 ms – 10 s** |

- **PERISTALSIS** replaces the sub oscillator: waves of contraction that
  squeeze the pressure and bend the pitch, with irregular timing, rolling
  from chamber to chamber.
- **FIZZ** replaces the click and noise transient: an acid layer that goes
  from a smooth hiss through effervescence to a sparse crackle.
- **PITCH** spans 16 Hz – 4 kHz (was 16 Hz – 1 kHz), for squeaks and critters.
- **RND** is reshaped for gut sounds (see [Playing it](#playing-it)).

## Playing it

- **Pads**: click a channel's STRIKE pad. Where you click sets the velocity:
  near the top is a hard hit, near the bottom a soft one. Velocity sets the
  level *and the pressure*: a soft hit on FLATUS barely flaps, a hard one
  blasts.
- **Keyboard**: `Z X C V` trigger channel 1 at increasing velocity,
  `N M , .` do the same for channel 2, and `Space` triggers both.
- **Knobs/faders**: drag vertically. Hold `Shift` for fine control, use the
  scroll wheel to nudge, and double-click to reset.
- **Locks**: **Ctrl-click** (or ⌘-click) any knob, fader, switch or engine
  button to lock it against RND and MUTATE. A locked control shows a padlock
  and turns rose; the padlock counter in the channel header unlocks them all.
  Locks belong to the channel, not the patch, and are saved with the panel.
  Locking never changes what the free controls roll.
- **MUTATE** (the knob under each pad) changes the channel's sound on every
  hit: each trigger draws a fresh random patch and moves each free control
  that fraction of the way towards it. At **RND** every hit is a new
  creature.
- **RND** draws each engine's pitch from its own register (SQUEAK and FLORA
  high, RUMBLE and GROWL low, FLATUS from tuba to trumpet) and keeps each
  engine's macros where it sounds most alive. Envelopes are usually short to
  medium, but sometimes swell in slowly; peristalsis squeezes about two rolls
  in three. A lowpass never swallows the sound: its cutoff stays above the
  pitch even after a negative envelope sweep. `make survey` on 300 rolls:

  | | p10 | median | p90 |
  | --- | --- | --- | --- |
  | Time to peak | 0 s | 0.07 s | 1.1 – 1.3 s |
  | Length (to −40 dB) | 0.8 s | 2.3 – 2.5 s | 5.4 s |
  | Spectral centroid | 115 Hz | 475 Hz | 1.6 – 1.9 kHz |

  Tests render 300 random patches through the real DSP and check every one
  is audible and bounded, and that RND rolls include slow swells, long
  sounds and short blurts.
- **Presets** (31), **RND** and **→ CHn** (copy a channel to the other) are in
  each channel's header. The presets are level-matched to −18 dBFS
  short-term loudness (±0.4 dB). The whole panel is saved automatically: to
  `localStorage` on the web, and to `~/.config/enterokussion-ek1/state.txt`
  on the desktop.
- The **HIT** display is an offline render of one hit of the current patch,
  as long as its longest audible chamber (up to 30 s). Long sounds are
  rendered at a lower rate so turning a knob never stalls the panel.

## The engines

The three big knobs under the faders are the engine's macros, shared by all
three chambers. They are relabelled for each engine, and so is the chaos
modulator's DEST switch.

| Engine | Process | X | Y | Z |
| --- | --- | --- | --- | --- |
| **GURGLE** | Gas bubbles detaching in fluid: a Poisson cloud of Minnaert resonances, each a damped sine whose damping follows van den Doel's fit and whose pitch glides as it rises | DENSITY: 0.5 … 400 bubbles/s, following the pressure | SCATTER: bubble sizes, ±2 octaves | CHIRP: bipolar glide, −1 (sinking) … +2 octaves (rising plinks) |
| **FLATUS** | A pressure-driven sphincter: a Van der Pol oscillator whose μ is the gas pressure, radiating the flow's thump, the snap of each opening and turbulent hiss and crackle into a cheek cavity | TIGHT: soft round puff … tight relaxation buzz | WET: dry trumpet … diseased sputter (stalls, stumbles, turbulence) | CHEEK: cavity resonance, 60 Hz – 2.4 kHz |
| **GROWL** | A creaky glottal pulse through three formants | VOWEL: OO, OH, AH, EH, EE | ROUGH: jitter, shimmer, breath and period doubling (a subharmonic rattle) | SIZE: formants ×0.35 (enormous) … ×1.6 (critter) |
| **SQUEAK** | Stick-slip friction (a bowed-string friction loop) rubbing three modes of a membrane | PRESS: loop gain, pure whistle … raucous creak | SLIDE: where the slip sits on the friction curve | MEMBR: mode ratios, tube (1:2:3) → membrane (1:1.59:2.14) → gristle (1:2.76:5.40) |
| **TRACT** | A Kelly–Lochbaum tube, tuned as a quarter-wave pipe to PITCH, with a Gaussian ring of muscle travelling along it | SQUEEZE: how far the ring closes, up to 97% | WAVE: speed and direction of the peristaltic wave, ±6 Hz | GAS: buzz … rushing gas |
| **SYMBIO** | Two organisms phase-modulating each other (Xenokussion's FLUX), the coupling following the pressure | RATIO: 0.5 … 7.13 | COUPLE: calm → locked → chaos | FEEDBK: self-feedback, sine → saw → noise |
| **FLORA** | A swarm of chirping gut flora: windowed, gliding, warbling sines released as a Poisson stream | DENSITY: 1 … 300 critters/s, following the pressure | CHIRP: glide and warble | CHATTER: a talk/listen process that groups critters into syllables and turns |
| **RUMBLE** | Borborygmus: three slightly stretched partials on independent random walks, roughened by gas | WANDER: pitch drift, ±12 semitones | ROUGH: noisy 10–60 Hz amplitude modulation and gas noise | GRIT: asymmetric drive, up to +24 dB |

Some details worth knowing:

- **FLATUS was tuned against real farts.** Recordings of a few (the
  "diseased gnome" meme among them: *"That's not a fart! That's
  disease!"*) were measured with the `lab` example. A wet one turned out to
  be not a pitched buzz at all but a big sub-70 Hz air thump under
  irregular, impulsive crackle, flapping in clumps about 12 Hz apart
  (periodicity ≈ 0.3–0.7, brightness ≈ 0.6–2.4 kHz, crest factor 4–8). A
  buzzier one was turbulent hiss gated by a valve flapping at ~50 Hz. So
  FLATUS radiates three things: the flow (thump), its derivative (the snap
  of each opening) and flow-gated turbulence whose band rises with the
  pressure, as turbulence does. WET also randomly stalls the valve (μ goes
  negative for a few cycles and the oscillation has to build up again) and
  makes the flap rate stumble. At `FLATUS 22 Hz, TIGHT 0.9, WET 1` its
  envelope statistics land close to the gnome's (fluctuation depth 1.35
  vs 1.25). The *Diseased Gnome* preset is built around that setting. The
  reference audio is not part of this repository.
- **The valve stays in tune.** The Van der Pol period stretches as μ grows
  (×2.08 at μ = 6); a table of periods computed with RK4 retunes it, and
  the valve is integrated in substeps, so FLATUS is within ~3% of PITCH from
  40 Hz to 900 Hz at any TIGHT.
- **SQUEAK's friction curve is scale-free**: its steepest negative slope is
  about −0.25 however it's scaled, so whether and how wildly it squeaks
  depends on the loop gain against it (PRESS) and where the slip speed sits
  on it (SLIDE). With no pressure nothing slides, so a decay sputters out.
  Some settings period-double (a squeak with a subharmonic growl under it).
- **TRACT's ring enters and leaves the tube** rather than wrapping around,
  so a wave rolls in and out. Where it pinches, the tube splits in two and
  the formants jump, and gas forced through the narrowing hisses.
- **GROWL's formants ring like cavities**: the resonators are used with their
  natural gain (≈ Q), so narrow high formants stand out of the source's
  falling spectrum, and EE really is brighter than OO.
- **Bubble clouds never clog**: when all 16 bubbles are busy, the quietest
  fades out in 4 ms to make room (no clicks). Bubbles are rotating phasors,
  not sines, which keeps a dense boil cheap.
- **Bounded everywhere.** A test runs every engine at every corner of its
  macros at 30 Hz, 250 Hz and 2 kHz and requires finite, audible, bounded
  output; another requires the driven engines to fall silent when the
  pressure goes.

## The chambers, the envelope and peristalsis

```
 PITCH ─┬─ SWEEP ─┬─ PERISTALSIS ─┬─ CHAOS ─┬─ JITTER ─► pitch ─┬── +TUNE2 ─┬── +TUNE3
        ▼                                                  CHAMBER 1  CHAMBER 2  CHAMBER 3   (the engine, ×3)
 TRIGGER ─► envelopes: DELAY ▸ ATTACK (CURVE) ▸ HOLD ▸ DECAY, × LENGTH per chamber
               × peristaltic squeeze × velocity ═► pressure into each chamber, and its level ─► Σ + FIZZ
                                                                                                │
   ┌────────────────────────────────────────────────────────────────────────────────────────────┘
   ▼
 [SAT pre] ─► LADDER ─► EQ ─► [SAT post] ─► OUTPUT ─► CHORUS (mono → stereo) ─┬───────────► PAN ─► mix
                                                                               └─► PLATE ─┘
```

- **CHAMBER 1–3**: three instances of the engine. Chamber 1 sets the pitch
  (PITCH, FINE) and the envelope; chambers 2 and 3 have a free **TUNE** (±24
  semitones, continuous), **LEVEL**, **LENGTH** (×0.125 … ×4 of the whole
  envelope: attack, hold and decay) and **DELAY** (up to 3 s). Delays turn
  one trigger into a sequence (*Fart in Three Movements*), lengths make one
  chamber outlast the others. Stacked chambers are scaled back towards the
  loudness of one.
- **ENVELOPE**: ATTACK (1 ms – 10 s), CURVE (bipolar: `t^4` swell … linear
  … `t^0.25` snap), HOLD (0 – 10 s) and the DECAY fader (30 ms – 20 s). A
  retrigger starts the attack from wherever the envelope is, so it never
  clicks, and during a DELAY the previous tail keeps decaying. Rates update
  every control block, so turning ATTACK mid-swell changes the swell.
- **SWEEP**: TIME (2 ms – 10 s) and a bipolar DEPTH (±6 octaves).
- **PERISTALSIS**: contractions `((1 − cos 2πφ)/2)²`, a rounded squeeze with
  slack between.
  - RATE: 0.1 – 25 Hz, from slow waves to a fluttering "brrr".
  - SQUEEZE: how far the pressure drops between contractions (with some
    make-up gain).
  - PITCH: bipolar pitch change at the peak of a squeeze, ±12 semitones.
  - IRREG: each contraction gets its own length and strength.
  - TRAVEL: the lag from one chamber to the next, up to half a cycle, so a
    squeeze rolls through the chambers.

  The ring next to the title contracts at the rate.
- **FIZZ**: LEVEL, TONE (band centre, 200 Hz – 12 kHz) and CRACKLE (hiss →
  hundreds of pops a second → sparse crackle; the pops get louder as they
  get fewer). It follows chamber 1's envelope and squeeze.
- **CHAOS**: a slow Lorenz attractor as an LFO, with RATE, DEPTH, DEST
  (pitch, X, Y, Z or cutoff) and JITTER (per-hit random offsets).
- The rest of the strip is SB-2's:
  - **LADDER FILTER**: a ZDF 4-pole ladder with LP12/LP24/BP/HP modes and a
    bipolar envelope amount.
  - **SATURATOR**: TAPE, TUBE, HARD or FOLD curves, pre or post.
  - **EQ**: low cut, low shelf, mid bell and high shelf, with a response
    display.
  - **CHORUS**: three-tap, with a crossover that keeps the lows mono.
  - **REVERB**: a Dattorro plate, the belly cavity.

## Performance

The voices run at 2× the output rate and are decimated by a 47-tap
windowed-sinc filter. With both channels hitting, three chambers each,
peristalsis, fizz and the whole strip busy, the slowest engine (FLORA)
renders about 18× faster than real time on one desktop core and the fastest
(SQUEAK) about 30× (see `make bench`).

## Layout

```
crates/dsp        engine, the 8 engines, chamber envelopes, peristalsis, fizz, chaos, ladder,
                  saturator, EQ, chorus, plate reverb, parameters, presets and the hit preview,
                  RND/MUTATE/locks; no dependencies, 93 unit tests
  src/engines/      gurgle, flatus, growl, squeak, tract, symbio, flora, rumble
  examples/         lab (one engine, or a WAV recording: level/pitch/periodicity/brightness/
                    crest/flutter over time), render (presets → WAV + loudness), bench,
                    calibrate (loudness survey), survey (what RND gives),
                    analyze (one preset over time), probe (which knob made it quiet?)
crates/worklet    C ABI over the engine, compiled to a standalone wasm module with no imports
crates/app        Dioxus UI with `web` and `desktop` features; panel state and persistence
  build.rs          (web) builds crates/worklet for wasm32 and embeds it
  src/audio/web.rs     AudioContext + AudioWorkletNode; the module is compiled on the main thread
  src/audio/worklet.js the AudioWorkletProcessor that calls the wasm engine
  src/audio/pulse.rs   audio thread + pa_simple playback stream
```

The `lab` example is the tool the engines were tuned with:

```sh
# One chamber of FLATUS at 21 Hz, X/Y/Z = 0.9/1/0.3, a pressure swell over 2.5 s, saved to a WAV
cargo run -p enterokussion-dsp --release --example lab -- FLATUS 21 0.9 1 0.3 swell 2.5 out.wav
# The same analysis of a recording (mono 16-bit WAV), 0.45 s in, 2.5 s long, 50 ms windows
cargo run -p enterokussion-dsp --release --example lab -- ref.wav 0.45 2.5 50
```

The UI never touches the audio engine directly. It sends small commands (set
a parameter, trigger, master volume): over the worklet's `MessagePort` on the
web, and over an `mpsc` channel on the desktop. Both backends render in
blocks of 128 frames. Locks live only in the UI; the engine never sees them.

## Inspiration and sources

- K. van den Doel, "Physically-based models for liquid sounds", ACM
  Transactions on Applied Perception 2(4), 2005 (GURGLE's bubbles); M.
  Minnaert, "On musical air-bubbles and the sounds of running water", Phil.
  Mag. 16, 1933
- B. van der Pol, "On relaxation-oscillations", Phil. Mag. 2(11), 1926
  (FLATUS's valve)
- M. E. McIntyre, R. T. Schumacher and J. Woodhouse, "On the oscillations of
  musical instruments", JASA 74(5), 1983, and P. Cook and G. Scavone's
  Synthesis ToolKit (`Bowed`, its friction table) (SQUEAK)
- J. L. Kelly Jr. and C. C. Lochbaum, "Speech synthesis", Proc. 4th Int.
  Congr. Acoustics, 1962 (TRACT)
- A. E. Rosenberg, "Effect of glottal pulse shape on the quality of natural
  vowels", JASA 49(2B), 1971; G. E. Peterson and H. L. Barney, "Control
  methods used in a study of the vowels", JASA 24(2), 1952 (GROWL)
- [Xenokussion XK-1](https://github.com/mkforsb/xenokussion-xk1-slop)'s FLUX
  (SYMBIO) and the rest of its frame;
  [Subkussion SB-2](https://github.com/mkforsb/subkussion-sb2-slop)'s channel
  strip and locks
- A. Simper, V. Zavalishin, *The Art of VA Filter Design* (TPT state-variable
  filter, ZDF ladder); R. Bristow-Johnson, *Audio EQ Cookbook*
- J. Dattorro, "Effect Design, Part 1: Reverberator and Other Filters", JAES 45(9), 1997 (plate reverb)
- E. N. Lorenz, "Deterministic Nonperiodic Flow", J. Atmos. Sci. 20, 1963 (the chaos modulator)
- A gnome, for reference
