use cozy_chess::Piece;

pub const MAX_PLY: i16 = 64;

pub const NONE: i16 = 32002;
pub const INFINITY: i16 = 32001;
pub const NEG_INFINITY: i16 = -INFINITY;

pub const MATE: i16 = 32000;
pub const MATE_IN: i16 = MATE - MAX_PLY;
pub const MATED_IN: i16 = -MATE_IN;

pub const TB_WIN: i16 = MATE_IN;
pub const TB_WIN_IN_PLY: i16 = TB_WIN - MAX_PLY;
pub const TB_LOSS_IN_PLY: i16 = -TB_WIN_IN_PLY;

pub const TIME_OVERHEAD: u64 = 3;

pub fn mated_in(ply: i16) -> i16 {
    return ply - MATE;
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
