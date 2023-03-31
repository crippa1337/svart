use crate::constants::*;
use cozy_chess::Move;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TTFlag {
    None,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub mv: Option<Move>, // 4 bytes
    pub key: u16,         // 2 bytes
    pub epoch: u16,       // 2 bytes
    pub score: i16,       // 2 bytes
    pub depth: u8,        // 1 byte
    pub flag: TTFlag,     // 1 byte
}

impl TTEntry {
    #[must_use]
    fn quality(&self) -> u16 {
        self.epoch * 2 + self.depth as u16
    }
}

#[derive(Clone)]
pub struct TT {
    pub entries: Vec<TTEntry>,
    pub epoch: u16,
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
                epoch: 0,
                depth: 0,
                flag: TTFlag::None,
            });
        }

        Self { entries, epoch: 0 }
    }

    #[must_use]
    pub fn index(&self, key: u64) -> usize {
        // Cool hack Cosmo taught me
        let key = key as u128;
        let len = self.entries.len() as u128;
        ((key * len) >> 64) as usize
    }

    #[must_use]
    pub fn probe(&self, key: u64) -> TTEntry {
        self.entries[self.index(key)]
    }

    pub fn age(&mut self) {
        self.epoch += 1;
    }

    pub fn store(
        &mut self,
        key: u64,
        mv: Option<Move>,
        score: i16,
        depth: u8,
        flag: TTFlag,
        ply: i32,
    ) {
        let target_index = self.index(key);
        let target = self.entries[target_index];
        let entry = TTEntry {
            key: key as u16,
            mv,
            score: self.score_to_tt(score, ply),
            epoch: self.epoch,
            depth,
            flag,
        };

        // Only replace entries of similar or higher quality
        if entry.quality() >= target.quality() {
            self.entries[target_index] = entry;
        }
    }

    // hint to cpu that this memory adress will be accessed soon
    // by slapping it in the cpu cache
    pub fn prefetch(&self, key: u64) {
        let index = self.index(key);
        let entry = &self.entries[index];
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            _mm_prefetch((entry as *const TTEntry).cast::<i8>(), _MM_HINT_T0);
        }
    }

    #[must_use]
    pub fn score_to_tt(&self, score: i16, ply: i32) -> i16 {
        if score >= TB_WIN_IN_PLY as i16 {
            score + ply as i16
        } else if score <= TB_LOSS_IN_PLY as i16 {
            score - ply as i16
        } else {
            score
        }
    }

    #[must_use]
    pub fn score_from_tt(&self, score: i16, ply: i32) -> i16 {
        if score >= TB_WIN_IN_PLY as i16 {
            score - ply as i16
        } else if score <= TB_LOSS_IN_PLY as i16 {
            score + ply as i16
        } else {
            score
        }
    }
}

const _TT_TEST: () = assert!(std::mem::size_of::<TTEntry>() == 12);
