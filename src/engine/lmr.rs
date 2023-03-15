const BASE_REDUCTION: f64 = 0.75;
const DIVISOR: f64 = 2.25;

pub struct LMRTable {
    pub table: [[i16; 64]; 64],
}

impl LMRTable {
    pub fn new() -> LMRTable {
        let mut lmr = LMRTable {
            table: [[0; 64]; 64],
        };

        for d in 0..64 {
            for m in 0..64 {
                let ld = f64::ln(d as f64);
                let lp = f64::ln(m as f64);
                lmr.table[d][m] = (BASE_REDUCTION + ld * lp / DIVISOR) as i16;
            }
        }

        lmr
    }

    #[must_use]
    pub fn reduction(&self, depth: i16, move_count: i16) -> i16 {
        let d = (depth.min(63)) as usize;
        let c = (move_count.min(63)) as usize;

        self.table[d][c]
    }
}
