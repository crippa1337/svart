use super::nnue::inference::{NNUEState, ACTIVATE, DEACTIVATE};
use cozy_chess::{Board, File, Move, Piece, Rank, Square};

pub struct Position {
    pub board: Board,
    pub nnue_state: Box<NNUEState>,
}

impl Position {
    pub fn default() -> Self {
        Self {
            board: Board::default(),
            nnue_state: NNUEState::from_board(&Board::default()),
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let board = Board::from_fen(fen, false).expect("Invalid FEN");

        Self {
            board: board.clone(),
            nnue_state: NNUEState::from_board(&board),
        }
    }

    pub fn play_move(&mut self, mv: Move) {
        self.nnue_state.push();

        let stm = self.board.side_to_move();
        let piece = self.board.piece_on(mv.from).unwrap();

        // Remove the from-square piece
        self.nnue_state
            .update_feature::<DEACTIVATE>(mv.from, piece, stm);

        // Remove the target-square piece
        // This also handles the move of the rook in castling
        if let Some((color, piece)) = self.board.color_on(mv.to).zip(self.board.piece_on(mv.to)) {
            self.nnue_state
                .update_feature::<DEACTIVATE>(mv.to, piece, color);
        }

        // Remove the en passant'd pawn
        if let Some(ep_file) = self.board.en_passant() {
            if piece == Piece::Pawn && mv.to == Square::new(ep_file, Rank::Sixth.relative_to(stm)) {
                self.nnue_state.update_feature::<DEACTIVATE>(
                    Square::new(ep_file, Rank::Fifth.relative_to(stm)),
                    Piece::Pawn,
                    !stm,
                );
            }
        }

        // Castling
        if Some(stm) == self.board.color_on(mv.to) {
            let rank = Rank::First.relative_to(stm);
            // King side
            if mv.from.file() < mv.to.file() {
                // Move the rook
                self.nnue_state.update_feature::<ACTIVATE>(
                    Square::new(File::F, rank),
                    Piece::Rook,
                    stm,
                );

                // Move the king
                self.nnue_state.update_feature::<ACTIVATE>(
                    Square::new(File::G, rank),
                    Piece::King,
                    stm,
                );
            // Queen side
            } else {
                self.nnue_state.update_feature::<ACTIVATE>(
                    Square::new(File::C, rank),
                    Piece::Rook,
                    stm,
                );
                self.nnue_state.update_feature::<ACTIVATE>(
                    Square::new(File::D, rank),
                    Piece::Rook,
                    stm,
                );
            }
        } else {
            // The only thing left is to add the moved piece to it's target-square.
            // This also handles the promotion of a pawn
            let new_piece = mv.promotion.unwrap_or(piece);
            self.nnue_state
                .update_feature::<ACTIVATE>(mv.to, new_piece, stm)
        }

        let mut new_board = self.board.clone();
        new_board.play_unchecked(mv);
        self.board = new_board;
    }

    pub fn play_null(&mut self) {
        self.board = self.board.null_move().unwrap();
    }

    #[must_use]
    pub fn is_capture(self, mv: Move) -> bool {
        self.board.colors(!self.board.side_to_move()).has(mv.to)
    }

    #[must_use]
    pub fn is_quiet(self, mv: Move) -> bool {
        mv.promotion.is_none() && !self.is_capture(mv)
    }

    pub fn evaluate(&self) -> i32 {
        self.nnue_state.evaluate(self.board.side_to_move())
    }
}
