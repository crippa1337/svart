use crate::{constants::*, engine::eval};
use cozy_chess::{BitBoard, Board, Move, Piece, Square};

use super::{
    movegen,
    tt::{Flag, TranspositionTable},
};

pub struct Search {
    pub pv_length: [i32; MAX_PLY as usize],
    pub pv_table: [[Option<Move>; MAX_PLY as usize]; MAX_PLY as usize],
    pub transposition_table: TranspositionTable,
    pub nodes: u32,
    pub tt_hits: u32,
}

impl Search {
    pub fn new() -> Self {
        return Search {
            pv_length: [0; MAX_PLY as usize],
            pv_table: [[None; MAX_PLY as usize]; MAX_PLY as usize],
            transposition_table: TranspositionTable::new(),
            nodes: 0,
            tt_hits: 0,
        };
    }

    fn qsearch(&mut self, board: &Board, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        if ply >= MAX_PLY {
            return eval::evaluate(board);
        }

        let mut stand_pat = eval::evaluate(board);
        if stand_pat >= beta {
            return stand_pat;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Transposition table lookup
        let hash_key = board.hash();
        let tt_entry = self.transposition_table.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key;
        let tt_move: Option<Move>;
        if tt_hit {
            tt_move = tt_entry.best_move;
        } else {
            tt_move = None;
        }
        let tt_score: i32;
        if tt_hit {
            tt_score = self.transposition_table.score_from_tt(tt_entry.score, ply)
        } else {
            tt_score = NONE;
        }

        // Treating depth as 0
        if tt_hit {
            if tt_entry.flag == Flag::EXACTBOUND
                || (tt_entry.flag == Flag::LOWERBOUND && tt_score >= beta)
                || (tt_entry.flag == Flag::UPPERBOUND && tt_score <= alpha)
            {
                self.tt_hits += 1;
                return tt_score;
            }
        }

        // Generate sorted captures
        let capture_list: Vec<Move> = movegen::capture_moves(board);

        let old_alpha = alpha;
        let mut best_move: Option<Move> = None;
        for mv in capture_list {
            self.nodes += 1;
            let victim = board.piece_on(mv.to);
            let captured: Piece;

            // En Passant
            if victim.is_none() {
                captured = Piece::Pawn;
            } else {
                captured = victim.unwrap();
            }

            // Delta Pruning
            if (piece_val(captured) + 400 + stand_pat) < alpha && !mv.promotion.is_some() {
                continue;
            }

            let mut new_board = board.clone();
            new_board.play(mv);
            let score = -self.qsearch(&new_board, -beta, -alpha, ply + 1);

            if score > stand_pat {
                stand_pat = score;

                if score > alpha {
                    alpha = score;
                    best_move = Some(mv);

                    if score >= beta {
                        break;
                    }
                }
            }
        }

        // Calculate bound and save to TT if no cutoff
        let bound: Flag = if stand_pat >= beta {
            Flag::LOWERBOUND
        } else if stand_pat > old_alpha {
            Flag::EXACTBOUND
        } else {
            Flag::UPPERBOUND
        };
        self.transposition_table
            .store(hash_key, 0, bound, stand_pat, best_move.into(), ply);

        return stand_pat;
    }

    pub fn absearch(
        &mut self,
        board: &Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u8,
        ply: i32,
        hash_history: &mut Vec<u64>,
    ) -> i32 {
        if ply >= MAX_PLY {
            return eval::evaluate(board);
        }

        // init PV
        self.pv_length[ply as usize] = ply;

        let history_key = board.hash_without_ep();
        let root = if ply == 0 { true } else { false };
        if !root {
            // Check for draw by 50 move rule
            let mut counter = 0;
            for key in hash_history.iter() {
                if *key == history_key {
                    counter += 1;
                }

                if counter >= 3 {
                    return 0;
                }
            }

            // Return 0 for draw by 50-move rule
            if board.halfmove_clock() >= 100 {
                return 0;
            }

            // Mate distance pruning
            let mate_val = mate_in(ply);
            if mate_val < beta {
                beta = mate_val;
                if alpha >= mate_val {
                    return mate_val;
                }
            }
        }

        // Escape condition
        if depth == 0 {
            return self.qsearch(board, alpha, beta, ply);
        }

        // Transposition table lookup
        let hash_key = board.hash();
        let tt_entry = self.transposition_table.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key;
        let tt_move: Option<Move>;
        if tt_hit {
            tt_move = tt_entry.best_move;
        } else {
            tt_move = None;
        }
        let tt_score: i32;
        if tt_hit {
            tt_score = self.transposition_table.score_from_tt(tt_entry.score, ply)
        } else {
            tt_score = NONE;
        }

        if !root && tt_hit && tt_entry.depth >= depth as i32 {
            self.tt_hits += 1;
            match tt_entry.flag {
                Flag::EXACTBOUND => {
                    return tt_score;
                }
                Flag::UPPERBOUND => beta = beta.min(tt_score),
                Flag::LOWERBOUND => alpha = alpha.max(tt_score),
                Flag::NONEBOUND => (),
            }
            if alpha >= beta {
                return tt_score;
            }
        }

        let old_alpha = alpha;
        let mut best_score = -INFINITY;
        let mut moves_done: u32 = 0;
        let mut best_move: Option<Move> = None;
        let mut move_list = movegen::all_moves(board);

        for mv in move_list {
            let mut new_board = board.clone();
            new_board.play(mv);
            self.nodes += 1;
            moves_done += 1;

            hash_history.push(history_key);
            let score = -self.absearch(&new_board, -beta, -alpha, depth - 1, ply + 1, hash_history);
            hash_history.pop();

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = Some(mv);
                    alpha = score;

                    let usize_ply = ply as usize;
                    // Write to PV table
                    self.pv_table[usize_ply][usize_ply] = Some(mv);

                    // Loop over next ply
                    let mut next_ply = usize_ply + 1;
                    while next_ply < self.pv_length[usize_ply + 1] as usize {
                        self.pv_table[usize_ply][next_ply] = self.pv_table[usize_ply + 1][next_ply];
                        next_ply += 1;
                    }

                    // Update PV length
                    self.pv_length[usize_ply] = self.pv_length[usize_ply + 1];

                    if score >= beta {
                        break;
                    }
                }
            }
        }

        // ENDSTATES
        if moves_done == 0 {
            // Mate
            if board.checkers() != BitBoard::EMPTY {
                return mated_in(ply);
            } else {
                // Stalemate
                return 0;
            }
        }

        // Calculate bound and save to TT if no cutoff
        let bound: Flag;
        if best_score >= beta {
            bound = Flag::LOWERBOUND;
        } else {
            if alpha != old_alpha {
                bound = Flag::EXACTBOUND;
            } else {
                bound = Flag::UPPERBOUND;
            }
        }
        self.transposition_table.store(
            hash_key,
            depth as i32,
            bound,
            best_score,
            best_move.into(),
            ply,
        );

        return best_score;
    }

    fn iterative_deepening(&mut self) {
        let mut depth = 3;
        let mut best_move: Option<Move> = None;

        for d in 0..MAX_PLY {}
    }
}
