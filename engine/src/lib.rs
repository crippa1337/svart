pub mod definitions {
    pub const MAX_PLY: usize = 128;
    pub const MAX_MOVES_POSITION: usize = 218;

    pub const NONE: i32 = 32002;
    pub const INFINITY: i32 = 32001;

    pub const MATE: i32 = 32000;
    pub const MATE_IN: i32 = MATE - MAX_PLY as i32;

    pub const TB_WIN: i32 = MATE_IN;
    pub const TB_WIN_IN_PLY: i32 = TB_WIN - MAX_PLY as i32;
    pub const TB_LOSS_IN_PLY: i32 = -TB_WIN_IN_PLY;

    pub const TIME_OVERHEAD: u64 = 5;

    pub const NOMOVE: u16 = 0b0000_0000_0000_0000;
}

pub mod body;
pub mod uci;
