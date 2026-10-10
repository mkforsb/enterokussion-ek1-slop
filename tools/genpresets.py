"""Generate crates/dsp/src/presets.rs for Enterokussion EK-1.

Presets are written in physical units and converted to knob positions with
the same mappings as `params::map`. An optional levels.json (preset name ->
OUTPUT knob) from the loudness pass sets each preset's Level.

usage: python3 tools/genpresets.py crates/dsp/src/presets.rs [tools/levels.json]

The level pass: run `make renders`, read each preset's short-term loudness,
and set OUTPUT so it lands at -18 dBFS (see levels.json).
"""
import json
import math
import sys

ln = math.log


def hz(f):  # PITCH fader
    return math.log2(f / 16.0) / 8.0


def decay(s):
    return ln(s / 0.03) / ln(20.0 / 0.03)


def attack(s):
    return ln(s / 0.001) / ln(10000.0)


def hold(s):
    return math.sqrt(s / 10.0)


def length(r):
    return ln(r / 0.125) / ln(32.0)


def delay(s):
    return math.sqrt(s / 3.0)


def peri(f):
    return ln(f / 0.1) / ln(250.0)


def peri_st(st):
    b = math.copysign(math.sqrt(abs(st) / 12.0), st)
    return 0.5 + b / 2.0


def sweep_time(s):
    return ln(s / 0.002) / ln(5000.0)


def sweep_oct(o):
    # 6·x·sqrt|x|
    x = math.copysign((abs(o) / 6.0) ** (2.0 / 3.0), o)
    return 0.5 + x / 2.0


def cutoff(f):
    return ln(f / 20.0) / ln(1000.0)


def cheek(f):
    return ln(f / 60.0) / ln(40.0)


def tune(st):
    return 0.5 + st / 48.0


def gurgle_density(n):
    return ln(n / 0.5) / ln(800.0)


def flora_density(n):
    return ln(n) / ln(300.0)


def fizz_tone(f):
    return ln(f / 200.0) / ln(60.0)


def reverb_decay(s):
    return ln(s / 0.3) / ln(40.0)


def fenv(o):
    # 8·x·sqrt|x|
    x = math.copysign((abs(o) / 8.0) ** (2.0 / 3.0), o)
    return 0.5 + x / 2.0


ENGINES = ["GURGLE", "FLATUS", "GROWL", "SQUEAK", "TRACT", "SYMBIO", "FLORA", "RUMBLE"]

# (name, engine, comment, {Param: value})
PRESETS = [
    # --- FLATUS ----------------------------------------------------------
    ("Diseased Gnome", "FLATUS",
     "Modelled on a certain gnome: a 20 Hz flap, tight and soaking wet, clumped by an irregular squeeze",
     dict(Pitch=hz(21), MacroX=0.9, MacroY=1.0, MacroZ=0.3, Attack=attack(0.006), Hold=hold(0.9),
          Decay=decay(0.7), PeriRate=peri(2.8), PeriSqueeze=0.55, PeriIrreg=0.75, PeriTravel=0.3,
          LowGain=0.6, LowFreq=0.25, ReverbMix=0.18, ReverbDecay=reverb_decay(0.8))),
    ("Brass Section", "FLATUS",
     "The rare trumpet kind: dry, tight, in tune, sagging at the end",
     dict(Pitch=hz(118), MacroX=0.85, MacroY=0.08, MacroZ=cheek(480), Attack=attack(0.015),
          Hold=hold(0.55), Decay=decay(0.5), SweepDepth=sweep_oct(0.4), SweepSpeed=sweep_time(0.25),
          ReverbMix=0.2)),
    ("Whoopee Cushion", "FLATUS",
     "A rubber bladder emptying: snaps open, flaps, sags and sputters out",
     dict(Pitch=hz(80), MacroX=0.6, MacroY=0.45, MacroZ=cheek(260), Attack=attack(0.008),
          Curve=0.75, Hold=hold(0.35), Decay=decay(0.6), PeriRate=peri(9), PeriSqueeze=0.25,
          PeriIrreg=0.5, ReverbMix=0.15)),
    ("Silent but Deadly", "FLATUS",
     "Barely a valve at all: a long, soft swell of hiss with the odd flap",
     dict(Pitch=hz(34), MacroX=0.15, MacroY=0.92, MacroZ=cheek(140), Attack=attack(0.7), Curve=0.3,
          Hold=hold(1.4), Decay=decay(1.6), Cutoff=cutoff(1400), FizzLevel=0.25, FizzTone=fizz_tone(900),
          FizzCrackle=0.1, ReverbMix=0.3, ReverbDecay=reverb_decay(1.5))),
    ("Long Distance Call", "FLATUS",
     "Three seconds of wavering, peristaltically squeezed bravado",
     dict(Pitch=hz(135), MacroX=0.7, MacroY=0.3, MacroZ=cheek(700), Attack=attack(0.02), Hold=hold(3.0),
          Decay=decay(1.8), PeriRate=peri(1.6), PeriSqueeze=0.6, PeriPitch=peri_st(4), PeriIrreg=0.4,
          ReverbMix=0.22)),
    ("Tuba Exhaust", "FLATUS",
     "Low, round and overdriven through a tube saturator",
     dict(Pitch=hz(46), MacroX=0.75, MacroY=0.2, MacroZ=cheek(110), Attack=attack(0.01), Hold=hold(0.9),
          Decay=decay(0.8), Drive=0.35, SatType=1.0, SatPos=1.0, ReverbMix=0.12)),
    ("Fart in Three Movements", "FLATUS",
     "The chambers fire one after another, each lower and longer: a travelling performance",
     dict(Pitch=hz(150), MacroX=0.7, MacroY=0.55, MacroZ=cheek(380), Attack=attack(0.01), Hold=hold(0.35),
          Decay=decay(0.45), Level2=0.9, Tune2=tune(-5), Delay2=delay(0.6), Length2=length(1.3),
          Level3=0.9, Tune3=tune(-12), Delay3=delay(1.35), Length3=length(2.0), ReverbMix=0.2)),
    # --- GURGLE ----------------------------------------------------------
    ("Midnight Gurgle", "GURGLE",
     "Bubbles rising through a dark belly on slow contractions",
     dict(Pitch=hz(220), MacroX=gurgle_density(40), MacroY=0.6, MacroZ=0.8, Attack=attack(0.3),
          Hold=hold(0.8), Decay=decay(1.6), PeriRate=peri(0.8), PeriSqueeze=0.7, PeriIrreg=0.5,
          Cutoff=cutoff(2200), ReverbMix=0.35, ReverbDecay=reverb_decay(1.2))),
    ("Bile Duct Drips", "GURGLE",
     "Sparse, rising plinks in a long, dark cavity",
     dict(Pitch=hz(620), MacroX=gurgle_density(7), MacroY=0.4, MacroZ=0.88, Attack=attack(0.005),
          Hold=hold(3.0), Decay=decay(4.0), ReverbMix=0.55, ReverbDecay=reverb_decay(4.0),
          ReverbTone=0.25, ReverbPredelay=0.4)),
    ("Acid Boil", "GURGLE",
     "A slow swell into a seething, crackling boil",
     dict(Pitch=hz(340), MacroX=gurgle_density(250), MacroY=0.8, MacroZ=0.7, Attack=attack(1.5),
          Curve=0.25, Hold=hold(2.0), Decay=decay(2.0), FizzLevel=0.45, FizzTone=fizz_tone(3500),
          FizzCrackle=0.65, ReverbMix=0.25)),
    ("Sinking Gloops", "GURGLE",
     "Big, falling bubbles: the drain is winning",
     dict(Pitch=hz(130), MacroX=gurgle_density(12), MacroY=0.5, MacroZ=0.2, Attack=attack(0.01),
          Hold=hold(1.0), Decay=decay(1.2), ReverbMix=0.3)),
    # --- GROWL -----------------------------------------------------------
    ("Hungry Beast", "GROWL",
     "Something vast and empty, swelling up from below",
     dict(Pitch=hz(44), MacroX=0.25, MacroY=0.75, MacroZ=0.2, Attack=attack(0.45), Curve=0.3,
          Hold=hold(1.0), Decay=decay(1.3), SweepDepth=sweep_oct(-0.5), SweepSpeed=sweep_time(0.6),
          Drive=0.3, SatType=1.0, SatPos=1.0, ReverbMix=0.25)),
    ("Stomach at 3 PM", "GROWL",
     "A mid-afternoon complaint in four or five squeezes",
     dict(Pitch=hz(88), MacroX=0.6, MacroY=0.5, MacroZ=0.45, Attack=attack(0.2), Hold=hold(1.5),
          Decay=decay(1.0), PeriRate=peri(3.5), PeriSqueeze=0.8, PeriPitch=peri_st(3), PeriIrreg=0.6,
          ReverbMix=0.2)),
    ("Xeno Larynx", "GROWL",
     "A small, rough, alien throat whose vowels drift on their own",
     dict(Pitch=hz(140), MacroX=0.9, MacroY=0.95, MacroZ=0.9, Attack=attack(0.05), Hold=hold(1.2),
          Decay=decay(0.9), ChaosDepth=0.7, ChaosDest=1.0, ChaosRate=0.55, Jitter=0.3, ReverbMix=0.25)),
    # --- SQUEAK ----------------------------------------------------------
    ("Balloon Membrane", "SQUEAK",
     "A wet finger on a stretched membrane, wobbling with each squeeze",
     dict(Pitch=hz(700), MacroX=0.2, MacroY=0.5, MacroZ=0.5, Attack=attack(0.05), Hold=hold(0.6),
          Decay=decay(0.4), PeriRate=peri(3.0), PeriSqueeze=0.3, PeriPitch=peri_st(5), ReverbMix=0.2)),
    ("Rubber Villi", "SQUEAK",
     "A chorus of tiny gristly squeaks, each chamber a little later",
     dict(Pitch=hz(1150), MacroX=0.4, MacroY=0.6, MacroZ=1.0, Attack=attack(0.03), Hold=hold(0.3),
          Decay=decay(0.35), Level2=0.85, Tune2=tune(7.02), Delay2=delay(0.15), Length2=length(0.7),
          Level3=0.8, Tune3=tune(12.1), Delay3=delay(0.32), Length3=length(0.5), ReverbMix=0.3)),
    ("Creaking Hull", "SQUEAK",
     "Organs grinding past each other under heavy pressure",
     dict(Pitch=hz(150), MacroX=1.0, MacroY=0.15, MacroZ=0.3, Attack=attack(0.8), Curve=0.3,
          Hold=hold(2.0), Decay=decay(1.5), PeriRate=peri(0.5), PeriSqueeze=0.5, Cutoff=cutoff(3500),
          ReverbMix=0.35, ReverbDecay=reverb_decay(2.5))),
    # --- TRACT -----------------------------------------------------------
    ("Peristaltic Wind", "TRACT",
     "Gas rolling through a long tube as the muscle ring travels along it",
     dict(Pitch=hz(70), MacroX=0.85, MacroY=0.72, MacroZ=0.9, Attack=attack(1.5), Curve=0.3,
          Hold=hold(3.0), Decay=decay(3.0), ReverbMix=0.4, ReverbDecay=reverb_decay(2.5))),
    ("Colon Choir", "TRACT",
     "Three buzzing tubes in a fifth and an octave, slowly wah-wahing",
     dict(Pitch=hz(110), MacroX=0.7, MacroY=0.62, MacroZ=0.08, Attack=attack(2.0), Curve=0.35,
          Hold=hold(4.0), Decay=decay(3.0), Level2=0.8, Tune2=tune(7.02), Length2=length(1.1),
          Level3=0.8, Tune3=tune(-12), Length3=length(1.2), ChorusMix=0.4, ReverbMix=0.4)),
    ("Gulp", "TRACT",
     "One quick swallow: a fast ring squeezing past",
     dict(Pitch=hz(180), MacroX=0.95, MacroY=0.92, MacroZ=0.4, Attack=attack(0.005), Hold=hold(0.15),
          Decay=decay(0.35), ReverbMix=0.15)),
    ("Ten Second Swell", "TRACT",
     "The longest attack there is: ten seconds of gas building up",
     dict(Pitch=hz(55), MacroX=0.8, MacroY=0.6, MacroZ=0.7, Attack=1.0, Curve=0.2, Hold=hold(0.5),
          Decay=decay(3.0), FilterEnv=fenv(3), Cutoff=cutoff(500), ReverbMix=0.4)),
    # --- SYMBIO ----------------------------------------------------------
    ("Symbiont Duet", "SYMBIO",
     "Two organisms and their echo half a second later, tangled up",
     dict(Pitch=hz(130), MacroX=0.35, MacroY=0.55, MacroZ=0.2, Attack=attack(0.8), Hold=hold(2.0),
          Decay=decay(2.0), Level2=0.8, Tune2=tune(5), Delay2=delay(0.5), ReverbMix=0.3)),
    ("Parasite Bloom", "SYMBIO",
     "A slow swell that boils over into chaos at its peak",
     dict(Pitch=hz(260), MacroX=0.62, MacroY=0.95, MacroZ=0.6, Attack=attack(3.0), Curve=0.1,
          Hold=hold(2.0), Decay=decay(4.0), ReverbMix=0.35, ReverbDecay=reverb_decay(3.0))),
    ("Mitochondria", "SYMBIO",
     "A locked pair pulsing on a fast peristaltic rhythm",
     dict(Pitch=hz(60), MacroX=0.26, MacroY=0.4, MacroZ=0.3, Attack=attack(0.02), Hold=hold(1.0),
          Decay=decay(0.8), PeriRate=peri(7), PeriSqueeze=0.9, PeriTravel=0.5, Level2=0.7,
          Tune2=tune(12), ReverbMix=0.15)),
    # --- FLORA -----------------------------------------------------------
    ("Gut Flora Conference", "FLORA",
     "Everybody talking in turns, nobody listening",
     dict(Pitch=hz(900), MacroX=flora_density(40), MacroY=0.5, MacroZ=0.9, Attack=attack(0.3),
          Hold=hold(4.0), Decay=decay(2.0), ReverbMix=0.3)),
    ("Probiotic Riot", "FLORA",
     "A dense, squealing swarm",
     dict(Pitch=hz(1500), MacroX=flora_density(150), MacroY=0.9, MacroZ=0.3, Attack=attack(0.1),
          Hold=hold(1.5), Decay=decay(1.0), ReverbMix=0.25)),
    ("Lonely Microbe", "FLORA",
     "One small voice now and then, far away",
     dict(Pitch=hz(700), MacroX=flora_density(3), MacroY=0.6, MacroZ=0.6, Attack=attack(0.01),
          Hold=hold(5.0), Decay=decay(3.0), ReverbMix=0.6, ReverbDecay=reverb_decay(5.0),
          ReverbPredelay=0.6)),
    # --- RUMBLE ----------------------------------------------------------
    ("Borborygmus", "RUMBLE",
     "The textbook rumble: rolling, beating, roughened by gas",
     dict(Pitch=hz(50), MacroX=0.5, MacroY=0.6, MacroZ=0.35, Attack=attack(0.6), Hold=hold(1.5),
          Decay=decay(1.5), PeriRate=peri(0.6), PeriSqueeze=0.6, PeriIrreg=0.5, ReverbMix=0.2)),
    ("Deep Digestion", "RUMBLE",
     "A four-second swell, six seconds of churning and a long fade",
     dict(Pitch=hz(32), MacroX=0.3, MacroY=0.3, MacroZ=0.6, Attack=attack(4.0), Curve=0.25,
          Hold=hold(6.0), Decay=decay(8.0), PeriRate=peri(0.3), PeriSqueeze=0.4, ReverbMix=0.25)),
    ("Leviathan Belly", "RUMBLE",
     "Something enormous rolls over, groaning upwards",
     dict(Pitch=hz(27), MacroX=0.7, MacroY=0.9, MacroZ=0.8, Attack=attack(0.3), Hold=hold(1.5),
          Decay=decay(2.5), SweepDepth=sweep_oct(-1.0), SweepSpeed=sweep_time(1.5), ReverbMix=0.3,
          ReverbDecay=reverb_decay(3.0))),
]

CHOICES = {"ChaosDest", "FilterType", "SatType", "SatPos", "Engine"}


def main():
    out = sys.argv[1]
    levels = {}
    if len(sys.argv) > 2:
        levels = json.load(open(sys.argv[2]))
    names = [p[0] for p in PRESETS]
    assert len(names) == len(set(names))
    lines = []
    w = lines.append
    w("//! Factory voice presets.")
    w("//!")
    w("//! Generated from physical units (Hz, seconds, semitones) by")
    w("//! `tools/genpresets.py` and level-matched with the `render` example.")
    w("")
    w("use crate::params::{Param, VoicePatch, default_patch};")
    w("")
    w("pub struct Preset {")
    w("    pub name: &'static str,")
    w("    pub values: &'static [(Param, f32)],")
    w("}")
    w("")
    w("impl Preset {")
    w("    /// Full patch: defaults overlaid with this preset's values.")
    w("    pub fn patch(&self) -> VoicePatch {")
    w("        let mut p = default_patch();")
    w("        for &(param, v) in self.values {")
    w("            p[param.index()] = param.sanitize(v);")
    w("        }")
    w("        p")
    w("    }")
    w("}")
    w("")
    w("use Param::*;")
    w("")
    w("pub const PRESETS: &[Preset] = &[")
    w("    Preset {")
    w('        name: "Init",')
    w("        values: &[],")
    w("    },")
    last = None
    for name, engine, comment, vals in PRESETS:
        if engine != last:
            w(f"    // --- {engine} " + "-" * (66 - len(engine)))
            last = engine
        w(f"    // {comment}.")
        w("    Preset {")
        w(f'        name: "{name}",')
        w("        values: &[")
        items = [("Engine", float(ENGINES.index(engine)))] + list(vals.items())
        if name in levels:
            items.append(("Level", levels[name]))
        for k, v in items:
            if k in CHOICES:
                s = f"{v:.1f}"
            else:
                v = min(max(v, 0.0), 1.0)
                s = f"{v:.3f}"
            w(f"            ({k}, {s}),")
        w("        ],")
        w("    },")
    w("];")
    src = "\n".join(lines) + "\n"
    old = open(out).read()
    tests = old[old.index("#[cfg(test)]"):]
    open(out, "w").write(src + "\n" + tests)


main()
