use crate::engine::psqt;
use cozy_chess::{Board, Color, Piece};

pub fn evaluate(board: &Board) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;

    for pt in Piece::ALL {
        for mut square in board.pieces(pt) {
            let piece_val = match pt {
                Piece::Pawn => 100,
                Piece::Knight => 320,
                Piece::Bishop => 330,
                Piece::Rook => 500,
                Piece::Queen => 900,
                Piece::King => 2000,
            };

            let color = board.color_on(square).unwrap();
            // Flip rank for PSQT if white
            if color == Color::White {
                square = square.flip_rank();
            }

            // get square index for PSQT
            let square_index = square as usize;
            let rank = square_index / 8;
            let file = square_index % 8;
            let pos_val = match pt {
                Piece::Pawn => psqt::PAWN_TABLE[rank][file],
                Piece::Knight => psqt::KNIGHT_TABLE[rank][file],
                Piece::Bishop => psqt::BISHOP_TABLE[rank][file],
                Piece::Rook => psqt::ROOK_TABLE[rank][file],
                Piece::Queen => psqt::QUEEN_TABLE[rank][file],
                Piece::King => psqt::KING_TABLE[rank][file],
            };

            match color {
                Color::White => white_score += piece_val + pos_val,
                Color::Black => black_score += piece_val + pos_val,
            }
        }
    }

    match board.side_to_move() {
        Color::White => return white_score - black_score,
        Color::Black => return (white_score - black_score) * -1,
    };
}
