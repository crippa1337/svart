use cozy_chess::{Board, Move};

pub const MAX_PLY: i16 = 64;

pub const NONE: i16 = 32002;
pub const INFINITY: i16 = 32001;

pub const MATE: i16 = 32000;
pub const MATE_IN: i16 = MATE - MAX_PLY;

pub const TB_WIN: i16 = MATE_IN;
pub const TB_WIN_IN_PLY: i16 = TB_WIN - MAX_PLY;
pub const TB_LOSS_IN_PLY: i16 = -TB_WIN_IN_PLY;

pub const TIME_OVERHEAD: u64 = 3;

pub fn mated_in(ply: i16) -> i16 {
    ply - MATE
}

pub fn capture_move(board: &Board, mv: Move) -> bool {
    board.colors(!board.side_to_move()).has(mv.to)
}

pub fn quiet_move(board: &Board, mv: Move) -> bool {
    !capture_move(board, mv) && mv.promotion.is_none()
}

#[cfg(test)]
mod tests {
    #[test]
    fn quiet_moves() {
        use crate::constants::quiet_move;
        use cozy_chess::{Board, Move, Piece, Square};

        let board_1 = Board::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            false,
        )
        .unwrap();

        let board_2 = Board::from_fen("8/1k6/1q3R2/8/2p5/5K2/1p6/8 w - - 0 1", false).unwrap();

        // Quiet pawn move
        let mv = Move {
            from: Square::A2,
            to: Square::A3,
            promotion: None,
        };

        // Queen promotion
        let mv_1 = Move {
            from: Square::B2,
            to: Square::B1,
            promotion: Some(Piece::Queen),
        };

        // Quiet rook move
        let mv_2 = Move {
            from: Square::F6,
            to: Square::F8,
            promotion: None,
        };

        // Queen takes rook
        let mv_3 = Move {
            from: Square::B6,
            to: Square::F6,
            promotion: None,
        };

        assert!(quiet_move(&board_1, mv));
        assert!(!quiet_move(&board_2, mv_1));
        assert!(quiet_move(&board_2, mv_2));
        assert!(!quiet_move(&board_2, mv_3));
    }
}
