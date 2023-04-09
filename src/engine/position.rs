use super::nnue::inference::{NNUEState, ACTIVATE, DEACTIVATE};
use cozy_chess::{Board, File, Move, Piece, Rank, Square};

pub fn play_move(board: &mut Board, nnue: &mut Box<NNUEState>, mv: Move) {
    nnue.push();

    let stm = board.side_to_move();
    let piece = board.piece_on(mv.from).unwrap();

    // Remove the from-square piece
    nnue.update_feature::<DEACTIVATE>(mv.from, piece, stm);

    // Remove the target-square piece
    // This also handles the move of the rook in castling
    if let Some((color, p)) = board.color_on(mv.to).zip(board.piece_on(mv.to)) {
        nnue.update_feature::<DEACTIVATE>(mv.to, p, color);
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
            nnue.update_feature::<ACTIVATE>(Square::new(File::D, rank), Piece::Rook, stm);
            nnue.update_feature::<ACTIVATE>(Square::new(File::C, rank), Piece::King, stm);
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
pub fn is_ep(board: &Board, mv: Move) -> bool {
    let stm = board.side_to_move();
    let piece = board.piece_on(mv.from).unwrap();

    if piece == Piece::Pawn {
        if let Some(ep_file) = board.en_passant() {
            if mv.to == Square::new(ep_file, Rank::Sixth.relative_to(stm)) {
                return true;
            }
        }
    }

    false
}

#[must_use]
pub fn is_capture(board: &Board, mv: Move) -> bool {
    board.colors(!board.side_to_move()).has(mv.to) || is_ep(board, mv)
}

#[must_use]
pub fn is_quiet(board: &Board, mv: Move) -> bool {
    mv.promotion.is_none() && !is_capture(board, mv)
}

#[cfg(test)]
mod tests {
    use crate::engine::position::{is_capture, is_quiet};

    #[test]
    fn quiet_moves() {
        use cozy_chess::{Board, Move, Piece, Square};

        let board_1 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", false)
                .unwrap();
        let board_2 = Board::from_fen("8/1k6/1q3R2/8/2p5/5K2/1p6/8 w - - 0 1", false).unwrap();
        let board_3 = Board::from_fen("2q5/1k5P/5n2/8/8/5K2/1p6/8 b - - 0 1", false).unwrap();

        // Quiet pawn move
        let mv = Move { from: Square::A2, to: Square::A3, promotion: None };

        // Queen promotion
        let mv_1 = Move { from: Square::H7, to: Square::H8, promotion: Some(Piece::Queen) };

        // Quiet rook move
        let mv_2 = Move { from: Square::F6, to: Square::F8, promotion: None };

        // Queen takes awn
        let mv_3 = Move { from: Square::E2, to: Square::B2, promotion: None };

        // Queen promotion
        let mv_4 = Move { from: Square::B2, to: Square::B1, promotion: Some(Piece::Queen) };

        // Knight takes pawn
        let mv_5 = Move { from: Square::F6, to: Square::H7, promotion: None };

        // Quiet check
        let mv_6 = Move { from: Square::C8, to: Square::C3, promotion: None };

        assert!(is_quiet(&board_1, mv));
        assert!(!is_quiet(&board_2, mv_1));
        assert!(is_quiet(&board_2, mv_2));
        assert!(!is_quiet(&board_2, mv_3));
        assert!(!is_quiet(&board_3, mv_4));
        assert!(!is_quiet(&board_3, mv_5));
        assert!(is_quiet(&board_3, mv_6));
    }

    #[test]
    fn capture_moves() {
        use cozy_chess::{Board, Move, Piece, Square};
        let board_1 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", false)
                .unwrap();

        let board_2 = Board::from_fen("8/1k6/1q3R2/8/2p5/5K2/1p6/8 w - - 0 1", false).unwrap();
        let board_3 = Board::from_fen("2q5/1k5P/5n2/8/8/5K2/1p6/8 b - - 0 1", false).unwrap();

        // Quiet pawn push
        let mv = Move { from: Square::A2, to: Square::A3, promotion: None };

        // Rook takes queen
        let mv_1 = Move { from: Square::F6, to: Square::B6, promotion: None };

        // Quiet king move
        let mv_2 = Move { from: Square::F3, to: Square::E2, promotion: None };

        // Queen promotion
        let mv_3 = Move { from: Square::B2, to: Square::B1, promotion: Some(Piece::Queen) };

        // Knight takes pawn
        let mv_4 = Move { from: Square::F6, to: Square::H7, promotion: None };

        assert!(!is_capture(&board_1, mv));
        assert!(is_capture(&board_2, mv_1));
        assert!(!is_capture(&board_3, mv_2));
        assert!(!is_capture(&board_3, mv_3));
        assert!(is_capture(&board_3, mv_4));
    }
}
