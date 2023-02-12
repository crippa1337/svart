use crate::constants::MAX_PLY;
use crate::uci::reverse_castling_move;
use cozy_chess::{Board, Move};

pub struct PVTable {
    pub length: [u8; MAX_PLY as usize],
    pub table: [[Option<Move>; MAX_PLY as usize]; MAX_PLY as usize],
}

impl PVTable {
    pub fn new() -> Self {
        PVTable {
            length: [0; MAX_PLY as usize],
            table: [[None; MAX_PLY as usize]; MAX_PLY as usize],
        }
    }

    pub fn store(&mut self, board: &Board, ply: u8, mut mv: Move) {
        // Write to PV table
        let uply = ply as usize;
        mv = reverse_castling_move(board, mv);
        self.table[uply][uply] = Some(mv);

        // Loop over the next ply
        for i in (uply + 1)..self.length[uply + 1] as usize {
            // Copy move from deeper ply into current line
            self.table[uply][i] = self.table[uply + 1][i];
        }

        // Update PV length
        self.length[uply] = self.length[uply + 1];
    }

    pub fn pv_string(&self) -> String {
        let mut pv = String::new();
        for i in 0..self.length[0] {
            if self.table[0][i as usize].is_none() {
                break;
            }
            pv.push(' ');
            pv.push_str(&self.table[0][i as usize].unwrap().to_string());
        }

        pv
    }
}
