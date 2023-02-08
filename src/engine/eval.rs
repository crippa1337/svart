use crate::engine::psqt::*;
use cozy_chess::{Board, Color, Piece};

const PHASE_INC: [i32; 6] = [0, 1, 1, 2, 4, 0];

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
    let mut mg = 0;
    let mut eg = 0;
    let mut game_phase = 0;

    for pt in Piece::ALL {
        for square in board.pieces(pt) {
            let piece = board.piece_on(square).unwrap();
            let color = board.color_on(square).unwrap();
            let sq = square.flip_rank() as usize;
            game_phase += PHASE_INC[p_type(piece)];

            match color {
                Color::White => {
                    mg += MG_TABLE[p_type(piece)][sq];
                    eg += EG_TABLE[p_type(piece)][sq];
                }
                Color::Black => {
                    mg -= MG_TABLE[p_type(piece) + 6][sq];
                    eg -= EG_TABLE[p_type(piece) + 6][sq];
                }
            }
        }
    }

    let mut mg_weight = game_phase;
    if mg_weight > 24 {
        mg_weight = 24
    };

    let eg_weight = 24 - mg_weight;

    match board.side_to_move() {
        Color::White => (((mg * mg_weight) + (eg * eg_weight)) / 24) as i16,
        Color::Black => (((mg * mg_weight) + (eg * eg_weight)) / -24) as i16,
    }
}
