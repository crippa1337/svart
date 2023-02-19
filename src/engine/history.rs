use cozy_chess::{Board, Move};

pub struct History {
    pub table: [[[i32; 64]; 64]; 2],
}

impl History {
    pub fn new() -> History {
        History {
            table: [[[0; 64]; 64]; 2],
        }
    }

    pub fn get_score(&self, board: &Board, mv: Move) -> i32 {
        let color = board.side_to_move() as usize;
        let from = mv.from as usize;
        let to = mv.to as usize;

        self.table[color][from][to]
    }

    pub fn update_table(
        &mut self,
        board: &Board,
        best_move: Move,
        quiet_moves: Vec<Move>,
        depth: u8,
    ) {
        let bonus = std::cmp::min(16 * (depth * depth) as i32, 1200);

        // Update best move
        self.update_score(board, best_move, bonus);

        // Decay the history table for all the quiet moves passed in
        for mv in quiet_moves {
            if mv == best_move {
                continue;
            }
            self.update_score(board, mv, -bonus)
        }
    }

    pub fn update_score(&mut self, board: &Board, mv: Move, bonus: i32) {
        let scaled_bonus = bonus - self.get_score(board, mv) * bonus.abs() / 32768;
        let color = board.side_to_move() as usize;
        let from = mv.from as usize;
        let to = mv.to as usize;

        self.table[color][from][to] += scaled_bonus;
    }
}
