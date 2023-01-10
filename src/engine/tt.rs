use crate::constants::*;
use cozy_chess::Move;

#[derive(Copy, Clone, PartialEq)]
pub enum Flag {
    NONEBOUND,
    UPPERBOUND,
    LOWERBOUND,
    EXACTBOUND,
}

#[derive(Copy, Clone)]
pub struct TEntry {
    pub key: u64,
    pub depth: i32,
    pub flag: Flag,
    pub score: i32,
    pub best_move: Option<Move>,
}

impl TEntry {
    pub fn new() -> Self {
        return TEntry {
            key: 0,
            depth: 0,
            flag: Flag::NONEBOUND,
            score: NONE,
            best_move: None,
        };
    }
}

pub struct TranspositionTable {
    pub table: Vec<TEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        return TranspositionTable {
            table: vec![TEntry::new(); TT_SIZE],
        };
    }

    pub fn index(&self, key: u64) -> usize {
        return key as usize % TT_SIZE;
    }

    pub fn probe(&self, key: u64) -> TEntry {
        let index = self.index(key);
        return self.table[index];
    }

    pub fn store(
        &mut self,
        key: u64,
        depth: i32,
        flag: Flag,
        score: i32,
        best_move: Option<Move>,
        ply: i32,
    ) {
        let index = self.index(key);
        let mut entry = self.table[index];

        // Replacement scheme - https://github.com/Disservin/python-chess-engine/blob/master/src/tt.py
        if entry.key != key || entry.best_move != best_move {
            entry.best_move = best_move;
        }

        if entry.key != key || flag == Flag::EXACTBOUND || depth + 4 > entry.depth {
            entry.depth = depth;
            entry.score = self.score_to_tt(score, ply);
            entry.key = key;
            entry.flag = flag;
        }

        self.table[index] = entry;
    }

    pub fn score_to_tt(&self, score: i32, ply: i32) -> i32 {
        if score >= TB_WIN_MAX {
            return score + ply;
        } else {
            if score <= TB_LOSS_MAX {
                return score - ply;
            } else {
                return score;
            }
        }
    }

    pub fn score_from_tt(&self, score: i32, ply: i32) -> i32 {
        if score >= TB_WIN_MAX {
            return score - ply;
        } else {
            if score <= TB_LOSS_MAX {
                return score + ply;
            } else {
                return score;
            }
        }
    }
}
