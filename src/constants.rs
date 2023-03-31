pub const MAX_PLY: i32 = 250;
pub const MAX_MOVES_POSITION: usize = 218;

pub const NONE: i32 = 32002;
pub const INFINITY: i32 = 32001;

pub const MATE: i32 = 32000;
pub const MATE_IN: i32 = MATE - MAX_PLY;

pub const TB_WIN: i32 = MATE_IN;
pub const TB_WIN_IN_PLY: i32 = TB_WIN - MAX_PLY;
pub const TB_LOSS_IN_PLY: i32 = -TB_WIN_IN_PLY;

pub const TIME_OVERHEAD: u64 = 10;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn quiet_moves() {
//         use cozy_chess::{Board, Move, Piece, Square};

//         let board_1 = Board::from_fen(
//             "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
//             false,
//         )
//         .unwrap();
//         let board_2 = Board::from_fen("8/1k6/1q3R2/8/2p5/5K2/1p6/8 w - - 0 1", false).unwrap();
//         let board_3 = Board::from_fen("2q5/1k5P/5n2/8/8/5K2/1p6/8 b - - 0 1", false).unwrap();

//         // Quiet pawn move
//         let mv = Move {
//             from: Square::A2,
//             to: Square::A3,
//             promotion: None,
//         };

//         // Queen promotion
//         let mv_1 = Move {
//             from: Square::H7,
//             to: Square::H8,
//             promotion: Some(Piece::Queen),
//         };

//         // Quiet rook move
//         let mv_2 = Move {
//             from: Square::F6,
//             to: Square::F8,
//             promotion: None,
//         };

//         // Queen takes awn
//         let mv_3 = Move {
//             from: Square::E2,
//             to: Square::B2,
//             promotion: None,
//         };

//         // Queen promotion
//         let mv_4 = Move {
//             from: Square::B2,
//             to: Square::B1,
//             promotion: Some(Piece::Queen),
//         };

//         // Knight takes pawn
//         let mv_5 = Move {
//             from: Square::F6,
//             to: Square::H7,
//             promotion: None,
//         };

//         // Quiet check
//         let mv_6 = Move {
//             from: Square::C8,
//             to: Square::C3,
//             promotion: None,
//         };

//         assert!(quiet_move(&board_1, mv));
//         assert!(!quiet_move(&board_2, mv_1));
//         assert!(quiet_move(&board_2, mv_2));
//         assert!(!quiet_move(&board_2, mv_3));
//         assert!(!quiet_move(&board_3, mv_4));
//         assert!(!quiet_move(&board_3, mv_5));
//         assert!(quiet_move(&board_3, mv_6));
//     }

//     #[test]
//     fn capture_moves() {
//         use crate::constants::capture_move;
//         use cozy_chess::{Board, Move, Piece, Square};
//         let board_1 = Board::from_fen(
//             "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
//             false,
//         )
//         .unwrap();

//         let board_2 = Board::from_fen("8/1k6/1q3R2/8/2p5/5K2/1p6/8 w - - 0 1", false).unwrap();
//         let board_3 = Board::from_fen("2q5/1k5P/5n2/8/8/5K2/1p6/8 b - - 0 1", false).unwrap();

//         // Quiet pawn push
//         let mv = Move {
//             from: Square::A2,
//             to: Square::A3,
//             promotion: None,
//         };

//         // Rook takes queen
//         let mv_1 = Move {
//             from: Square::F6,
//             to: Square::B6,
//             promotion: None,
//         };

//         // Quiet king move
//         let mv_2 = Move {
//             from: Square::F3,
//             to: Square::E2,
//             promotion: None,
//         };

//         // Queen promotion
//         let mv_3 = Move {
//             from: Square::B2,
//             to: Square::B1,
//             promotion: Some(Piece::Queen),
//         };

//         // Knight takes pawn
//         let mv_4 = Move {
//             from: Square::F6,
//             to: Square::H7,
//             promotion: None,
//         };

//         assert!(!capture_move(&board_1, mv));
//         assert!(capture_move(&board_2, mv_1));
//         assert!(!capture_move(&board_3, mv_2));
//         assert!(!capture_move(&board_3, mv_3));
//         assert!(capture_move(&board_3, mv_4));
//     }
// }
