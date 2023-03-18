use crate::constants::MAX_PLY;

const FEATURES: usize = 768;
const HIDDEN: usize = 256;

// clipped relu
const CR_MIN: i32 = 0;
const CR_MAX: i32 = 255;

// quantization
const QAB: i32 = 255 * 64;
const SCALE: i32 = 400;

struct Accumulator {
    white: [i16; HIDDEN],
    black: [i16; HIDDEN],
}

struct Parameters {
    feature_weights: [i16; FEATURES * HIDDEN],
    feature_bias: [i16; HIDDEN],
    output_weights: [i16; HIDDEN * 2],
    output_bias: i16,
}
