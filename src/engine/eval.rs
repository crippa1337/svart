use crate::engine::psqt::*;
use cozy_chess::{Board, Color, Piece};

const PHASE_INC: [i16; 6] = [0, 1, 1, 2, 4, 0];

fn p_type(piece: Piece) -> usize {
    match piece {
        Piece::Pawn => 0,
        Piece::Knight => 1,
        Piece::Bishop => 2,
        Piece::Rook => 3,
        Piece::Queen => 4,
        Piece::King => 5,
    }
}

pub fn evaluate(board: &Board) -> i16 {
    let mut white_mg: i16 = 0;
    let mut white_eg: i16 = 0;
    let mut black_mg: i16 = 0;
    let mut black_eg: i16 = 0;
    let mut game_phase = 0;

    for pt in Piece::ALL {
        for square in board.pieces(pt) {
            let piece = board.piece_on(square).unwrap();
            let color = board.color_on(square).unwrap();
            let sq = square.flip_rank() as usize;
            game_phase += PHASE_INC[p_type(piece)];

            match color {
                Color::White => {
                    white_mg += MG_TABLE[p_type(piece)][sq];
                    white_eg += EG_TABLE[p_type(piece)][sq];
                }
                Color::Black => {
                    black_mg += MG_TABLE[p_type(piece) + 6][sq];
                    black_eg += EG_TABLE[p_type(piece) + 6][sq];
                }
            }
        }
    }

    let mg_score = white_mg - black_mg;
    let eg_score = white_eg - black_eg;

    let mut mg_weight = game_phase;
    if mg_weight > 24 {
        mg_weight = 24
    };

    let eg_weight = 24 - mg_weight;

    match board.side_to_move() {
        Color::White => (mg_score * mg_weight + eg_score * eg_weight) / 24,
        Color::Black => (mg_score * mg_weight + eg_score * eg_weight) / -24,
    }
}
