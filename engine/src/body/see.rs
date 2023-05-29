use cozy_chess::{Board, Move, Piece};

use super::position::is_ep;

const VALUES: [i32; 6] = [100, 300, 330, 500, 900, 0];

// fn see(board: &Board, mv: Move, threshold: i32) -> bool {
//     let stm = board.side_to_move();

//     let score = move_gain(board, mv) - threshold;

//     // Threshold breaks the gain
//     if score < 0 {
//         return false;
//     }

//     false
// }

fn move_gain(board: &Board, mv: Move) -> i32 {
    // Castling
    if board.piece_on(mv.from) == Some(Piece::King) && mv.from.file() < mv.to.file() {
        return 0;
    }

    if is_ep(board, mv) {
        return VALUES[Piece::Pawn as usize];
    }

    // Capture ?
    #[rustfmt::skip]
    let score = if let Some(captured_piece) = board.piece_on(mv.to) { 
        VALUES[captured_piece as usize]
    } else {
        0
    };

    if let Some(promo) = mv.promotion {
        return score + VALUES[promo as usize] - VALUES[Piece::Pawn as usize];
    }

    score
}
