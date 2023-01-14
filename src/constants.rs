use cozy_chess::Piece;

pub const MAX_PLY: i32 = 64;

pub const INFINITY: i32 = 99999;
pub const MATE: i32 = INFINITY - 1;
pub const MATE_IN: i32 = MATE - MAX_PLY;
pub const MATED_IN: i32 = -MATE_IN;
pub const NONE: i32 = INFINITY + 1;

// https://github.com/Disservin/python-chess-engine/blob/ab54c003d3e2252c50f7a398089987c8fe803c86/src/helpers.py#L13
pub const TB_WIN: i32 = MATED_IN;
pub const TB_WIN_MAX: i32 = TB_WIN - MAX_PLY;
pub const TB_LOSS_MAX: i32 = -TB_WIN_MAX;

pub fn mated_in(ply: i32) -> i32 {
    return ply - MATE;
}

pub fn mate_in(ply: i32) -> i32 {
    return MATE - ply;
}

pub const TT_SIZE: usize = 2usize.pow(19) - 1;

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
