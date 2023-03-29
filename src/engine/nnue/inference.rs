// Svart uses a (768->256)x2->1 perspective NNUE, largely inspired by Viridithas and Carp.
// A huge thanks to Cosmo and Dede for their help with the implementation.
//
// I hope to further improve the network as well as make the code more original in the future.
use crate::constants::MAX_PLY;
use cozy_chess::{Board, Color, Piece, Square};

const FEATURES: usize = 768;
const HIDDEN: usize = 256;

// clipped relu bounds
const CR_MIN: i32 = 0;
const CR_MAX: i32 = 255;

// quantization
const QAB: i32 = 255 * 64;
const SCALE: i32 = 400;

const ACTIVATE: bool = true;
const DEACTIVATE: bool = false;

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
// hidden layer from both perspectives
struct Accumulator {
    white: [i16; HIDDEN],
    black: [i16; HIDDEN],
}

pub struct NNUEState {
    accumulators: [Accumulator; MAX_PLY as usize],
    current_acc: usize,
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
    // efficiently update the change of a feature
    fn update_hidden<const ON: bool>(&mut self, idx: (usize, usize)) {
        fn update_perspective<const ON: bool>(acc: &mut [i16; HIDDEN], idx: usize) {
            // we iterate over the weights corresponding to the feature that has been changed
            // and then update the activations in the hidden layer accordingly
            let feature_weights = acc
                .iter_mut()
                // the column of the weight matrix corresponding to the index of the feature
                .zip(&MODEL.feature_weights[idx..idx + HIDDEN]);

            for (activation, &weight) in feature_weights {
                if ON {
                    *activation += weight;
                } else {
                    *activation -= weight;
                }
            }
        }

        update_perspective::<ON>(&mut self.white, idx.0);
        update_perspective::<ON>(&mut self.black, idx.1);
    }
}

impl NNUEState {
    // Referencing Viridithas' implementation:
    //
    //                 pov i16 hidden stack
    // The NNUEState is 2 * 2 * 256 * 250 + 8 bytes = 256,008 bytes large at the time of writing.
    // This would blow the stack if it were to be allocated on it, so we have to box it.
    // This is done by allocating the memory manually and then constructing the object in place.
    // Why not just box normally? Because rustc in debug mode will first allocate on the stack
    // before moving it to the heap when boxxing, which would blow the stack.
    pub fn from_board(board: &Board) -> Box<Self> {
        let mut boxed: Box<NNUEState> = unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr.cast())
        };

        // initialize the first state
        boxed.accumulators[0] = Accumulator::default();
        for sq in board.occupied() {
            let piece = board.piece_on(sq).unwrap();
            let color = board.color_on(sq).unwrap();
            let idx = weight_column_index(sq, piece, color);

            boxed.accumulators[0].update_hidden::<ACTIVATE>(idx);
        }

        boxed
    }
}

// Returns white's and black's feature weight index respectively
// i.e where the feature's weight column is in the weight matrix.
#[must_use]
fn weight_column_index(sq: Square, piece: Piece, color: Color) -> (usize, usize) {
    // The jump from one perspective to the other
    const COLOR_STRIDE: usize = 64 * 6;
    // The jump from one piece type to the next
    const PIECE_STRIDE: usize = 64;
    let p = match piece {
        Piece::Pawn => 0,
        Piece::Knight => 1,
        Piece::Bishop => 2,
        Piece::Rook => 3,
        Piece::Queen => 4,
        Piece::King => 5,
    };

    let c = color as usize;

    // STM's perspective is treated as being on top
    let white_idx = c * COLOR_STRIDE + p * PIECE_STRIDE + sq as usize;
    let black_idx = (1 ^ c) * COLOR_STRIDE + p * PIECE_STRIDE + sq.flip_rank() as usize;

    (white_idx * HIDDEN, black_idx * HIDDEN)
}
