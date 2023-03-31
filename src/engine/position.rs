use super::nnue::inference::{NNUEState, ACTIVATE, DEACTIVATE};
use cozy_chess::{Board, File, Move, Piece, Rank, Square};

pub fn board_default() -> (Board, Box<NNUEState>) {
    (Board::default(), NNUEState::from_board(&Board::default()))
}

pub fn play_move(board: &mut Board, nnue: &mut Box<NNUEState>, mv: Move) {
    nnue.push();

    let stm = board.side_to_move();
    let piece = board.piece_on(mv.from).unwrap();

    // Remove the from-square piece
    nnue.update_feature::<DEACTIVATE>(mv.from, piece, stm);

    // Remove the target-square piece
    // This also handles the move of the rook in castling
    if let Some((color, piece)) = board.color_on(mv.to).zip(board.piece_on(mv.to)) {
        nnue.update_feature::<DEACTIVATE>(mv.to, piece, color);
    }

    // Remove the en passant'd pawn
    if let Some(ep_file) = board.en_passant() {
        if piece == Piece::Pawn && mv.to == Square::new(ep_file, Rank::Sixth.relative_to(stm)) {
            nnue.update_feature::<DEACTIVATE>(
                Square::new(ep_file, Rank::Fifth.relative_to(stm)),
                Piece::Pawn,
                !stm,
            );
        }
    }

    // Castling
    if Some(stm) == board.color_on(mv.to) {
        let rank = Rank::First.relative_to(stm);
        // King side
        if mv.from.file() < mv.to.file() {
            // Move the rook
            nnue.update_feature::<ACTIVATE>(Square::new(File::F, rank), Piece::Rook, stm);

            // Move the king
            nnue.update_feature::<ACTIVATE>(Square::new(File::G, rank), Piece::King, stm);
        // Queen side
        } else {
            nnue.update_feature::<ACTIVATE>(Square::new(File::C, rank), Piece::King, stm);
            nnue.update_feature::<ACTIVATE>(Square::new(File::D, rank), Piece::Rook, stm);
        }
    } else {
        // The only thing left is to add the moved piece to it's target-square.
        // This also handles the promotion of a pawn
        let new_piece = mv.promotion.unwrap_or(piece);
        nnue.update_feature::<ACTIVATE>(mv.to, new_piece, stm)
    }

    board.play_unchecked(mv);
}

#[must_use]
pub fn is_capture(board: &Board, mv: Move) -> bool {
    board.colors(!board.side_to_move()).has(mv.to)
}

#[must_use]
pub fn is_quiet(board: &Board, mv: Move) -> bool {
    mv.promotion.is_none() && !is_capture(board, mv)
}
