use cozy_chess::Piece;

pub const MAX_PLY: i16 = 64;

pub const INFINITY: i16 = 32000;
pub const NEG_INFINITY: i16 = -32000;
pub const MATE: i16 = INFINITY - 1;
pub const MATE_IN: i16 = MATE - MAX_PLY;
pub const MATED_IN: i16 = -MATE_IN;
pub const NONE: i16 = INFINITY + 1;

pub const TB_WIN: i16 = MATE_IN;
pub const TB_WIN_MAX: i16 = TB_WIN - MAX_PLY;
pub const TB_LOSS_MAX: i16 = -TB_WIN_MAX;

pub fn mated_in(ply: i16) -> i16 {
    return ply - MATE;
}

pub fn mate_in(ply: i16) -> i16 {
    return MATE - ply;
}

pub fn piece_val(piece: Piece) -> i16 {
    return match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 2000,
    };
}
