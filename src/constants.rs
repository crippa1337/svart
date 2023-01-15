use cozy_chess::Piece;

pub const MAX_PLY: i32 = 64;

pub const INFINITY: i32 = 99999;
pub const NEG_INFINITY: i32 = -99999;
pub const MATE: i32 = INFINITY - 1;
pub const MATE_IN: i32 = MATE - MAX_PLY;
pub const MATED_IN: i32 = -MATE_IN;
pub const NONE: i32 = INFINITY + 1;

pub fn mated_in(ply: i32) -> i32 {
    return ply - MATE;
}

pub fn mate_in(ply: i32) -> i32 {
    return MATE - ply;
}

pub fn piece_val(piece: Piece) -> i32 {
    return match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 2000,
    };
}
