use crate::constants::MAX_PLY;
use crate::uci::handler::reverse_castling_move;
use cozy_chess::{Board, Move};

pub struct PVTable {
    pub length: usize,
    pub table: [Option<Move>; MAX_PLY as usize],
}

impl PVTable {
    pub fn new() -> Self {
        PVTable {
            length: 0,
            table: [None; MAX_PLY as usize],
        }
    }

    pub fn store(&mut self, board: &Board, mv: Move, old: &Self) {
        let mv = reverse_castling_move(board, mv);
        self.table[0] = Some(mv);
        self.table[1..=old.length].copy_from_slice(&old.table[..old.length]);
        self.length = old.length + 1;
    }

    pub fn moves(&self) -> &[Option<Move>] {
        &self.table[..self.length]
    }

    pub fn pv_string(&self) -> String {
        let mut pv = String::new();
        for &mv in self.moves() {
            pv.push(' ');
            pv.push_str(mv.unwrap().to_string().as_str());
        }

        pv
    }
}
