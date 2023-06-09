use cozy_chess::{Board, Move, Piece};

use super::position::is_ep;

const VALUES: [i32; 6] = [100, 300, 330, 500, 900, 0];

fn see(board: &Board, mv: Move, threshold: i32) -> bool {
    // Value of the move minus the threshold
    let mut score = move_gain(board, mv) - threshold;
    
    let next_victim = if let Some(promo) = mv.promotion {
        promo
    } else {
        board.piece_on(mv.from).unwrap()
    };

    // Assume best case, no more captures
    // ----------------------------------
    // Fails to beat threshold
    if score < 0 {
        return false;
    }

    // Assume the worst case, the opponent captures the piece back
    score -= VALUES[next_victim as usize];

    // Worst case beats the threshold
    if score >= 0 {
        return true;
    }

    let bishops = board.pieces(Piece::Bishop) | board.pieces(Piece::Queen);
    let rooks = board.pieces(Piece::Rook) | board.pieces(Piece::Queen);
    let stm = board.side_to_move();
 

    false
}

fn move_gain(board: &Board, mv: Move) -> i32 {
    // Castling
    if board.color_on(mv.to).is_some() {
        return 0
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
