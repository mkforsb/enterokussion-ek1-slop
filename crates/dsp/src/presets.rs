//! Factory voice presets.
//!
//! Generated from physical units (Hz, seconds, semitones) by
//! `tools/genpresets.py` and level-matched with the `render` example.

use crate::params::{Param, VoicePatch, default_patch};

pub struct Preset {
    pub name: &'static str,
    pub values: &'static [(Param, f32)],
}

impl Preset {
    /// Full patch: defaults overlaid with this preset's values.
    pub fn patch(&self) -> VoicePatch {
        let mut p = default_patch();
        for &(param, v) in self.values {
            p[param.index()] = param.sanitize(v);
        }
        p
    }
}

use Param::*;

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Init",
        values: &[],
    },
    // --- FLATUS ------------------------------------------------------------
    // Modelled on a certain gnome: a 20 Hz flap, tight and soaking wet, clumped by an irregular squeeze.
    Preset {
        name: "Diseased Gnome",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.049),
            (MacroX, 0.900),
            (MacroY, 1.000),
            (MacroZ, 0.300),
            (Attack, 0.195),
            (Hold, 0.300),
            (Decay, 0.484),
            (PeriRate, 0.604),
            (PeriSqueeze, 0.550),
            (PeriIrreg, 0.750),
            (PeriTravel, 0.300),
            (LowGain, 0.600),
            (LowFreq, 0.250),
            (ReverbMix, 0.180),
            (ReverbDecay, 0.266),
            (Level, 0.573),
        ],
    },
    // The rare trumpet kind: dry, tight, in tune, sagging at the end.
    Preset {
        name: "Brass Section",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.360),
            (MacroX, 0.850),
            (MacroY, 0.080),
            (MacroZ, 0.564),
            (Attack, 0.294),
            (Hold, 0.235),
            (Decay, 0.433),
            (SweepDepth, 0.582),
            (SweepSpeed, 0.567),
            (ReverbMix, 0.200),
            (Level, 0.822),
        ],
    },
    // A rubber bladder emptying: snaps open, flaps, sags and sputters out.
    Preset {
        name: "Whoopee Cushion",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.290),
            (MacroX, 0.600),
            (MacroY, 0.450),
            (MacroZ, 0.398),
            (Attack, 0.226),
            (Curve, 0.750),
            (Hold, 0.187),
            (Decay, 0.461),
            (PeriRate, 0.815),
            (PeriSqueeze, 0.250),
            (PeriIrreg, 0.500),
            (ReverbMix, 0.150),
            (Level, 0.837),
        ],
    },
    // Barely a valve at all: a long, soft swell of hiss with the odd flap.
    Preset {
        name: "Silent but Deadly",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.136),
            (MacroX, 0.150),
            (MacroY, 0.920),
            (MacroZ, 0.230),
            (Attack, 0.711),
            (Curve, 0.300),
            (Hold, 0.374),
            (Decay, 0.612),
            (Cutoff, 0.615),
            (FizzLevel, 0.250),
            (FizzTone, 0.367),
            (FizzCrackle, 0.100),
            (ReverbMix, 0.300),
            (ReverbDecay, 0.436),
            (Level, 0.785),
        ],
    },
    // Three seconds of wavering, peristaltically squeezed bravado.
    Preset {
        name: "Long Distance Call",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.385),
            (MacroX, 0.700),
            (MacroY, 0.300),
            (MacroZ, 0.666),
            (Attack, 0.325),
            (Hold, 0.548),
            (Decay, 0.630),
            (PeriRate, 0.502),
            (PeriSqueeze, 0.600),
            (PeriPitch, 0.789),
            (PeriIrreg, 0.400),
            (ReverbMix, 0.220),
            (Level, 0.746),
        ],
    },
    // Low, round and overdriven through a tube saturator.
    Preset {
        name: "Tuba Exhaust",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.190),
            (MacroX, 0.750),
            (MacroY, 0.200),
            (MacroZ, 0.164),
            (Attack, 0.250),
            (Hold, 0.300),
            (Decay, 0.505),
            (Drive, 0.350),
            (SatType, 1.0),
            (SatPos, 1.0),
            (ReverbMix, 0.120),
            (Level, 0.746),
        ],
    },
    // The chambers fire one after another, each lower and longer: a travelling performance.
    Preset {
        name: "Fart in Three Movements",
        values: &[
            (Engine, 1.0),
            (Pitch, 0.404),
            (MacroX, 0.700),
            (MacroY, 0.550),
            (MacroZ, 0.500),
            (Attack, 0.250),
            (Hold, 0.187),
            (Decay, 0.416),
            (Level2, 0.900),
            (Tune2, 0.396),
            (Delay2, 0.447),
            (Length2, 0.676),
            (Level3, 0.900),
            (Tune3, 0.250),
            (Delay3, 0.671),
            (Length3, 0.800),
            (ReverbMix, 0.200),
            (Level, 1.000),
        ],
    },
    // --- GURGLE ------------------------------------------------------------
    // Bubbles rising through a dark belly on slow contractions.
    Preset {
        name: "Midnight Gurgle",
        values: &[
            (Engine, 0.0),
            (Pitch, 0.473),
            (MacroX, 0.656),
            (MacroY, 0.600),
            (MacroZ, 0.800),
            (Attack, 0.619),
            (Hold, 0.283),
            (Decay, 0.612),
            (PeriRate, 0.377),
            (PeriSqueeze, 0.700),
            (PeriIrreg, 0.500),
            (Cutoff, 0.680),
            (ReverbMix, 0.350),
            (ReverbDecay, 0.376),
            (Level, 0.688),
        ],
    },
    // Sparse, rising plinks in a long, dark cavity.
    Preset {
        name: "Bile Duct Drips",
        values: &[
            (Engine, 0.0),
            (Pitch, 0.660),
            (MacroX, 0.395),
            (MacroY, 0.400),
            (MacroZ, 0.880),
            (Attack, 0.175),
            (Hold, 0.548),
            (Decay, 0.752),
            (ReverbMix, 0.550),
            (ReverbDecay, 0.702),
            (ReverbTone, 0.250),
            (ReverbPredelay, 0.400),
            (Level, 0.902),
        ],
    },
    // A slow swell into a seething, crackling boil.
    Preset {
        name: "Acid Boil",
        values: &[
            (Engine, 0.0),
            (Pitch, 0.551),
            (MacroX, 0.930),
            (MacroY, 0.800),
            (MacroZ, 0.700),
            (Attack, 0.794),
            (Curve, 0.250),
            (Hold, 0.447),
            (Decay, 0.646),
            (FizzLevel, 0.450),
            (FizzTone, 0.699),
            (FizzCrackle, 0.650),
            (ReverbMix, 0.250),
            (Level, 0.519),
        ],
    },
    // Big, falling bubbles: the drain is winning.
    Preset {
        name: "Sinking Gloops",
        values: &[
            (Engine, 0.0),
            (Pitch, 0.378),
            (MacroX, 0.475),
            (MacroY, 0.500),
            (MacroZ, 0.200),
            (Attack, 0.250),
            (Hold, 0.316),
            (Decay, 0.567),
            (ReverbMix, 0.300),
            (Level, 0.672),
        ],
    },
    // --- GROWL -------------------------------------------------------------
    // Something vast and empty, swelling up from below.
    Preset {
        name: "Hungry Beast",
        values: &[
            (Engine, 2.0),
            (Pitch, 0.182),
            (MacroX, 0.250),
            (MacroY, 0.750),
            (MacroZ, 0.200),
            (Attack, 0.663),
            (Curve, 0.300),
            (Hold, 0.316),
            (Decay, 0.580),
            (SweepDepth, 0.405),
            (SweepSpeed, 0.670),
            (Drive, 0.300),
            (SatType, 1.0),
            (SatPos, 1.0),
            (ReverbMix, 0.250),
            (Level, 1.000),
        ],
    },
    // A mid-afternoon complaint in four or five squeezes.
    Preset {
        name: "Stomach at 3 PM",
        values: &[
            (Engine, 2.0),
            (Pitch, 0.307),
            (MacroX, 0.600),
            (MacroY, 0.500),
            (MacroZ, 0.450),
            (Attack, 0.575),
            (Hold, 0.387),
            (Decay, 0.539),
            (PeriRate, 0.644),
            (PeriSqueeze, 0.800),
            (PeriPitch, 0.750),
            (PeriIrreg, 0.600),
            (ReverbMix, 0.200),
            (Level, 0.912),
        ],
    },
    // A small, rough, alien throat whose vowels drift on their own.
    Preset {
        name: "Xeno Larynx",
        values: &[
            (Engine, 2.0),
            (Pitch, 0.391),
            (MacroX, 0.900),
            (MacroY, 0.950),
            (MacroZ, 0.900),
            (Attack, 0.425),
            (Hold, 0.346),
            (Decay, 0.523),
            (ChaosDepth, 0.700),
            (ChaosDest, 1.0),
            (ChaosRate, 0.550),
            (Jitter, 0.300),
            (ReverbMix, 0.250),
            (Level, 0.808),
        ],
    },
    // --- SQUEAK ------------------------------------------------------------
    // A wet finger on a stretched membrane, wobbling with each squeeze.
    Preset {
        name: "Balloon Membrane",
        values: &[
            (Engine, 3.0),
            (Pitch, 0.681),
            (MacroX, 0.200),
            (MacroY, 0.500),
            (MacroZ, 0.500),
            (Attack, 0.425),
            (Hold, 0.245),
            (Decay, 0.398),
            (PeriRate, 0.616),
            (PeriSqueeze, 0.300),
            (PeriPitch, 0.823),
            (ReverbMix, 0.200),
            (Level, 0.692),
        ],
    },
    // A chorus of tiny gristly squeaks, each chamber a little later.
    Preset {
        name: "Rubber Villi",
        values: &[
            (Engine, 3.0),
            (Pitch, 0.771),
            (MacroX, 0.400),
            (MacroY, 0.600),
            (MacroZ, 1.000),
            (Attack, 0.369),
            (Hold, 0.173),
            (Decay, 0.378),
            (Level2, 0.850),
            (Tune2, 0.646),
            (Delay2, 0.224),
            (Length2, 0.497),
            (Level3, 0.800),
            (Tune3, 0.752),
            (Delay3, 0.327),
            (Length3, 0.400),
            (ReverbMix, 0.300),
            (Level, 0.827),
        ],
    },
    // Organs grinding past each other under heavy pressure.
    Preset {
        name: "Creaking Hull",
        values: &[
            (Engine, 3.0),
            (Pitch, 0.404),
            (MacroX, 1.000),
            (MacroY, 0.150),
            (MacroZ, 0.300),
            (Attack, 0.726),
            (Curve, 0.300),
            (Hold, 0.447),
            (Decay, 0.602),
            (PeriRate, 0.291),
            (PeriSqueeze, 0.500),
            (Cutoff, 0.748),
            (ReverbMix, 0.350),
            (ReverbDecay, 0.575),
            (Level, 0.525),
        ],
    },
    // --- TRACT -------------------------------------------------------------
    // Gas rolling through a long tube as the muscle ring travels along it.
    Preset {
        name: "Peristaltic Wind",
        values: &[
            (Engine, 4.0),
            (Pitch, 0.266),
            (MacroX, 0.850),
            (MacroY, 0.720),
            (MacroZ, 0.900),
            (Attack, 0.794),
            (Curve, 0.300),
            (Hold, 0.548),
            (Decay, 0.708),
            (ReverbMix, 0.400),
            (ReverbDecay, 0.575),
            (Level, 0.733),
        ],
    },
    // Three buzzing tubes in a fifth and an octave, slowly wah-wahing.
    Preset {
        name: "Colon Choir",
        values: &[
            (Engine, 4.0),
            (Pitch, 0.348),
            (MacroX, 0.700),
            (MacroY, 0.620),
            (MacroZ, 0.080),
            (Attack, 0.825),
            (Curve, 0.350),
            (Hold, 0.632),
            (Decay, 0.708),
            (Level2, 0.800),
            (Tune2, 0.646),
            (Length2, 0.628),
            (Level3, 0.800),
            (Tune3, 0.250),
            (Length3, 0.653),
            (ChorusMix, 0.400),
            (ReverbMix, 0.400),
            (Level, 0.741),
        ],
    },
    // One quick swallow: a fast ring squeezing past.
    Preset {
        name: "Gulp",
        values: &[
            (Engine, 4.0),
            (Pitch, 0.436),
            (MacroX, 0.950),
            (MacroY, 0.920),
            (MacroZ, 0.400),
            (Attack, 0.175),
            (Hold, 0.122),
            (Decay, 0.378),
            (ReverbMix, 0.150),
            (Level, 0.989),
        ],
    },
    // The longest attack there is: ten seconds of gas building up.
    Preset {
        name: "Ten Second Swell",
        values: &[
            (Engine, 4.0),
            (Pitch, 0.223),
            (MacroX, 0.800),
            (MacroY, 0.600),
            (MacroZ, 0.700),
            (Attack, 1.000),
            (Curve, 0.200),
            (Hold, 0.224),
            (Decay, 0.708),
            (FilterEnv, 0.760),
            (Cutoff, 0.466),
            (ReverbMix, 0.400),
            (Level, 0.881),
        ],
    },
    // --- SYMBIO ------------------------------------------------------------
    // Two organisms and their echo half a second later, tangled up.
    Preset {
        name: "Symbiont Duet",
        values: &[
            (Engine, 5.0),
            (Pitch, 0.378),
            (MacroX, 0.350),
            (MacroY, 0.550),
            (MacroZ, 0.200),
            (Attack, 0.726),
            (Hold, 0.447),
            (Decay, 0.646),
            (Level2, 0.800),
            (Tune2, 0.604),
            (Delay2, 0.408),
            (ReverbMix, 0.300),
            (Level, 0.818),
        ],
    },
    // A slow swell that boils over into chaos at its peak.
    Preset {
        name: "Parasite Bloom",
        values: &[
            (Engine, 5.0),
            (Pitch, 0.503),
            (MacroX, 0.620),
            (MacroY, 0.950),
            (MacroZ, 0.600),
            (Attack, 0.869),
            (Curve, 0.100),
            (Hold, 0.447),
            (Decay, 0.752),
            (ReverbMix, 0.350),
            (ReverbDecay, 0.624),
            (Level, 0.837),
        ],
    },
    // A locked pair pulsing on a fast peristaltic rhythm.
    Preset {
        name: "Mitochondria",
        values: &[
            (Engine, 5.0),
            (Pitch, 0.238),
            (MacroX, 0.260),
            (MacroY, 0.400),
            (MacroZ, 0.300),
            (Attack, 0.325),
            (Hold, 0.316),
            (Decay, 0.505),
            (PeriRate, 0.769),
            (PeriSqueeze, 0.900),
            (PeriTravel, 0.500),
            (Level2, 0.700),
            (Tune2, 0.750),
            (ReverbMix, 0.150),
            (Level, 0.624),
        ],
    },
    // --- FLORA -------------------------------------------------------------
    // Everybody talking in turns, nobody listening.
    Preset {
        name: "Gut Flora Conference",
        values: &[
            (Engine, 6.0),
            (Pitch, 0.727),
            (MacroX, 0.647),
            (MacroY, 0.500),
            (MacroZ, 0.900),
            (Attack, 0.619),
            (Hold, 0.632),
            (Decay, 0.646),
            (ReverbMix, 0.300),
            (Level, 0.586),
        ],
    },
    // A dense, squealing swarm.
    Preset {
        name: "Probiotic Riot",
        values: &[
            (Engine, 6.0),
            (Pitch, 0.819),
            (MacroX, 0.878),
            (MacroY, 0.900),
            (MacroZ, 0.300),
            (Attack, 0.500),
            (Hold, 0.387),
            (Decay, 0.539),
            (ReverbMix, 0.250),
            (Level, 0.676),
        ],
    },
    // One small voice now and then, far away.
    Preset {
        name: "Lonely Microbe",
        values: &[
            (Engine, 6.0),
            (Pitch, 0.681),
            (MacroX, 0.193),
            (MacroY, 0.600),
            (MacroZ, 0.600),
            (Attack, 0.250),
            (Hold, 0.707),
            (Decay, 0.708),
            (ReverbMix, 0.600),
            (ReverbDecay, 0.763),
            (ReverbPredelay, 0.600),
            (Level, 0.708),
        ],
    },
    // --- RUMBLE ------------------------------------------------------------
    // The textbook rumble: rolling, beating, roughened by gas.
    Preset {
        name: "Borborygmus",
        values: &[
            (Engine, 7.0),
            (Pitch, 0.205),
            (MacroX, 0.500),
            (MacroY, 0.600),
            (MacroZ, 0.350),
            (Attack, 0.695),
            (Hold, 0.387),
            (Decay, 0.602),
            (PeriRate, 0.325),
            (PeriSqueeze, 0.600),
            (PeriIrreg, 0.500),
            (ReverbMix, 0.200),
            (Level, 0.638),
        ],
    },
    // A four-second swell, six seconds of churning and a long fade.
    Preset {
        name: "Deep Digestion",
        values: &[
            (Engine, 7.0),
            (Pitch, 0.125),
            (MacroX, 0.300),
            (MacroY, 0.300),
            (MacroZ, 0.600),
            (Attack, 0.901),
            (Curve, 0.250),
            (Hold, 0.775),
            (Decay, 0.859),
            (PeriRate, 0.199),
            (PeriSqueeze, 0.400),
            (ReverbMix, 0.250),
            (Level, 0.684),
        ],
    },
    // Something enormous rolls over, groaning upwards.
    Preset {
        name: "Leviathan Belly",
        values: &[
            (Engine, 7.0),
            (Pitch, 0.094),
            (MacroX, 0.700),
            (MacroY, 0.900),
            (MacroZ, 0.800),
            (Attack, 0.619),
            (Hold, 0.387),
            (Decay, 0.680),
            (SweepDepth, 0.349),
            (SweepSpeed, 0.777),
            (ReverbMix, 0.300),
            (ReverbDecay, 0.624),
            (Level, 0.832),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ENGINE_COUNT;
    use crate::voice::Voice;

    #[test]
    fn preset_names_unique() {
        for (i, a) in PRESETS.iter().enumerate() {
            for b in &PRESETS[i + 1..] {
                assert_ne!(a.name, b.name);
            }
        }
    }

    #[test]
    fn presets_are_in_range() {
        for p in PRESETS {
            for &(param, v) in p.values {
                assert_eq!(param.sanitize(v), v, "{}: {:?}", p.name, param);
            }
        }
    }

    #[test]
    fn every_engine_has_presets() {
        for e in 0..ENGINE_COUNT {
            let n = PRESETS
                .iter()
                .filter(|p| p.patch()[Param::Engine.index()] as usize == e)
                .count();
            assert!(n >= 2, "engine {e}: {n} presets");
        }
    }

    #[test]
    fn presets_sound_and_stay_sane() {
        let fs = 24_000.0;
        for p in PRESETS {
            let mut v = Voice::new(fs, 1);
            let patch = p.patch();
            for param in crate::params::ALL_PARAMS {
                v.set_param(param, patch[param.index()]);
            }
            v.snap_params();
            v.trigger(1.0);
            // Long enough to reach the peak of even the slowest swell.
            let seconds =
                (crate::params::map::attack_seconds(patch[Param::Attack.index()]) + 2.0).min(13.0);
            let out: Vec<f32> = (0..(fs * seconds) as usize).map(|_| v.tick()).collect();
            let peak = out.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
            assert!(out.iter().all(|x| x.is_finite()), "{}", p.name);
            assert!(peak > 0.1 && peak < 2.0, "{}: peak {peak}", p.name);
        }
    }
}
