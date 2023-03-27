use crate::constants::MAX_PLY;

const FEATURES: usize = 768;
const HIDDEN: usize = 256;

// clipped relu bounds
const CR_MIN: i32 = 0;
const CR_MAX: i32 = 255;

// quantization
const QAB: i32 = 255 * 64;
const SCALE: i32 = 400;

struct Parameters {
    feature_weights: [i16; FEATURES * HIDDEN],
    feature_bias: [i16; HIDDEN],
    output_weights: [i16; HIDDEN * 2], // perspective aware
    output_bias: i16,
}

// the model is read from binary files at compile time
static MODEL: Parameters = Parameters {
    feature_weights: unsafe { std::mem::transmute(*include_bytes!("net/feature_weights.bin")) },
    feature_bias: unsafe { std::mem::transmute(*include_bytes!("net/feature_bias.bin")) },
    output_weights: unsafe { std::mem::transmute(*include_bytes!("net/output_weights.bin")) },
    output_bias: unsafe { std::mem::transmute(*include_bytes!("net/output_bias.bin")) },
};

struct Accumulator {
    white: [i16; HIDDEN],
    black: [i16; HIDDEN],
}

impl Default for Accumulator {
    fn default() -> Self {
        Self {
            white: MODEL.feature_bias,
            black: MODEL.feature_bias,
        }
    }
}
