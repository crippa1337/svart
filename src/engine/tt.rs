use crate::constants::*;
use cozy_chess::Move;

#[derive(Clone, Copy)]
pub enum TTFlag {
    None,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub key: u64,         // 8 bytes
    pub mv: Option<Move>, // 4 bytes
    pub score: i16,       // 2 bytes
    pub depth: u8,        // 1 byte
    pub flags: TTFlag,
}

#[derive(Clone)]
pub struct TT {
    entries: Vec<TTEntry>,
}

impl TT {
    pub fn new(mb: u32) -> Self {
        let hash_size = mb * 1024 * 1024;
        let size = hash_size / std::mem::size_of::<TTEntry>() as u32;
        let mut entries = Vec::with_capacity(size as usize);
        for _ in 0..size {
            entries.push(TTEntry {
                key: 0,
                mv: None,
                score: 0,
                depth: 0,
                flags: TTFlag::None,
            });
        }

        return Self { entries };
    }

    pub fn index(&self, key: u64) -> usize {
        return key as usize % self.entries.len();
    }

    pub fn probe(&self, key: u64) -> TTEntry {
        let index = self.index(key);
        return self.entries[index];
    }

    pub fn store(&mut self, key: u64, mv: Option<Move>, score: i16, depth: u8, flags: TTFlag) {
        let index = self.index(key);

        // Always replace scheme
        self.entries[index] = TTEntry {
            key,
            mv,
            score,
            depth,
            flags,
        };
    }

    pub fn score_to_tt(&self, score: i16, ply: i16) -> i16 {
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

    pub fn score_from_tt(&self, score: i16, ply: i16) -> i16 {
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

#[allow(dead_code)]
pub const TT_TEST: () = assert!(std::mem::size_of::<TTEntry>() == 16, "TT IS NOT 16 BYTES");
