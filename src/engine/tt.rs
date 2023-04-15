use crate::definitions::{NOMOVE, TB_LOSS_IN_PLY, TB_WIN_IN_PLY};
use cozy_chess::{Move, Piece, Square};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TTFlag {
    None,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackedMove(u16);

impl PackedMove {
    pub fn new(mv: Option<Move>) -> Self {
        if mv.is_none() {
            return Self(NOMOVE);
        }
        let mv = mv.unwrap();
        let from = mv.from as u16; // 0..63, 6 bits
        let to = mv.to as u16; // 0..63, 6 bits

        // First bit represents promotion, next 2 bits represent piece type
        let promotion: u16 = match mv.promotion {
            None => 0b000,
            Some(Piece::Knight) => 0b100,
            Some(Piece::Bishop) => 0b101,
            Some(Piece::Rook) => 0b110,
            Some(Piece::Queen) => 0b111,
            _ => unreachable!(),
        };

        // 6 + 6 + 3 bits and one for padding gets a 2 byte move
        let packed = from | to << 6 | promotion << 12;

        Self(packed)
    }

    pub fn unpack(self) -> Move {
        let from = Square::index((self.0 & 0b111111) as usize);
        let to = Square::index(((self.0 >> 6) & 0b111111) as usize);

        let promotion = match (self.0 >> 12) & 0b111 {
            0b000 => None,
            0b100 => Some(Piece::Knight),
            0b101 => Some(Piece::Bishop),
            0b110 => Some(Piece::Rook),
            0b111 => Some(Piece::Queen),
            _ => unreachable!(),
        };

        Move { from, to, promotion }
    }
}

struct AgeAndFlag(u8);

impl AgeAndFlag {
    fn new(age: u8, flag: TTFlag) -> Self {
        let flag = match flag {
            TTFlag::None => 0b00,
            TTFlag::Exact => 0b01,
            TTFlag::LowerBound => 0b10,
            TTFlag::UpperBound => 0b11,
        };

        Self(age << 2 | flag)
    }

    fn age(&self) -> u8 {
        self.0 >> 2
    }

    fn flag(&self) -> TTFlag {
        match self.0 & 0b11 {
            0b00 => TTFlag::None,
            0b01 => TTFlag::Exact,
            0b10 => TTFlag::LowerBound,
            0b11 => TTFlag::UpperBound,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub mv: PackedMove, // 2 byte move wrapper (6 + 6 + 3 bits)
    pub key: u16,       // 2 bytes
    pub epoch: u16,     // 2 bytes
    pub score: i16,     // 2 bytes
    pub depth: u8,      // 1 byte
    pub flag: TTFlag,   // 1 byte
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
                mv: PackedMove(NOMOVE),
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
        // Cap at 63 for wrapping into 6 bits
        self.epoch = 63.min(self.epoch + 1);
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
            mv: PackedMove::new(mv),
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

    pub fn reset(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = TTEntry {
                key: 0,
                mv: PackedMove(NOMOVE),
                score: 0,
                epoch: 0,
                depth: 0,
                flag: TTFlag::None,
            };
        }
    }
}

const _TT_TEST: () = assert!(std::mem::size_of::<TTEntry>() == 10);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tt_reset() {
        let mut tt = TT::new(1);
        let mv = Move { from: Square::A1, to: Square::A2, promotion: None };
        tt.store(5, Some(mv), 1, 3, TTFlag::UpperBound, 22);
        assert_eq!(tt.probe(5).score, 1);

        tt.reset();
        for entry in tt.entries.iter() {
            assert_eq!(entry.score, 0);
            assert_eq!(entry.flag, TTFlag::None);
            assert_eq!(entry.depth, 0);
            assert_eq!(entry.key, 0);
            assert_eq!(entry.mv, PackedMove(NOMOVE));
        }
    }

    #[test]
    fn packed_moves() {
        let mv = Move { from: Square::A1, to: Square::A2, promotion: None };
        let packed = PackedMove::new(Some(mv));
        assert_eq!(packed.unpack(), mv);

        let mv = Move { from: Square::B7, to: Square::A2, promotion: Some(Piece::Knight) };
        let packed = PackedMove::new(Some(mv));
        assert_eq!(packed.unpack(), mv);

        let mv = Move { from: Square::C1, to: Square::A2, promotion: Some(Piece::Bishop) };
        let packed = PackedMove::new(Some(mv));
        assert_eq!(packed.unpack(), mv);

        let mv = Move { from: Square::H3, to: Square::H4, promotion: Some(Piece::Rook) };
        let packed = PackedMove::new(Some(mv));
        assert_eq!(packed.unpack(), mv);

        let mv = Move { from: Square::D8, to: Square::D7, promotion: Some(Piece::Queen) };
        let packed = PackedMove::new(Some(mv));
        assert_eq!(packed.unpack(), mv);
    }
}
