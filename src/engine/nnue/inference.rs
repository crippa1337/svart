// Svart uses a (768->256)x2->1 perspective NNUE, largely inspired by Viridithas and Carp.
// A huge thanks to Cosmo and Dede for their help with the implementation.
//
// I hope to further improve the network as well as make the code more original in the future.

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

// the accumulator represents the
// hidden layer from both perspective
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

impl Accumulator {
    // efficiently updates the weights of a feature, either on or off
    fn update_weights<const ON: bool>(&mut self, idx: (usize, usize)) {
        fn update<const ON: bool>(acc: &mut [i16; HIDDEN], idx: usize) {
            let feature_weights = acc
                .iter_mut()
                // zips a column of the weight matrix, this
                // being the weights for the feature we're toggling
                .zip(&MODEL.feature_weights[idx..idx + HIDDEN]);

            for (acc_val, &weight) in feature_weights {
                if ON {
                    *acc_val += weight;
                } else {
                    *acc_val -= weight;
                }
            }
        }

        // update both perspectives
        update::<ON>(&mut self.white, idx.0);
        update::<ON>(&mut self.black, idx.1);
    }
}
