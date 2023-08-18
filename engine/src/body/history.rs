use cozy_chess::{Board, Move};

pub const MAX_HISTORY: i32 = i16::MAX as i32;

#[derive(Clone)]
pub struct History {
    pub table: [[[i32; 64]; 64]; 2],
}

impl History {
    pub fn new() -> History {
        History {
            table: [[[0; 64]; 64]; 2],
        }
    }

    #[must_use]
    pub fn get_score(&self, board: &Board, mv: Move) -> i32 {
        let color = board.side_to_move() as usize;
        let from = mv.from as usize;
        let to = mv.to as usize;

        self.table[color][from][to]
    }

    pub fn update_table<const POSITIVE: bool>(&mut self, board: &Board, mv: Move, depth: i32) {
        let delta = (16 * (depth * depth)).min(1200);
        let bonus = if POSITIVE { delta } else { -delta };

        self.update_score(board, mv, bonus);
    }

    pub fn update_score(&mut self, board: &Board, mv: Move, bonus: i32) {
        let scaled_bonus = bonus - self.get_score(board, mv) * bonus.abs() / MAX_HISTORY;

        let color = board.side_to_move() as usize;
        let from = mv.from as usize;
        let to = mv.to as usize;

        self.table[color][from][to] += scaled_bonus;
    }

    pub fn age_table(&mut self) {
        self.table
            .iter_mut()
            .flatten()
            .flatten()
            .for_each(|x| *x /= 2);
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
