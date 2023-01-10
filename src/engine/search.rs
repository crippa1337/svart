use crate::{constants::*, engine::eval};
use cozy_chess::{BitBoard, Board, Move};

use super::tt::{Flag, TranspositionTable};

pub struct Search {
    pub transposition_table: TranspositionTable,
    pub nodes: u32,
    pub tables_used: u32,
    hash_history: Vec<u64>,
}

impl Search {
    pub fn new() -> Self {
        return Search {
            transposition_table: TranspositionTable::new(),
            nodes: 0,
            tables_used: 0,
            hash_history: Vec::new(),
        };
    }

    pub fn absearch(
        &mut self,
        board: &Board,
        mut alpha: i32,
        mut beta: i32,
        depth: u8,
        ply: i32,
    ) -> (i32, Option<Move>) {
        if ply >= MAX_PLY {
            return (eval::evaluate(board), None);
        }

        let root = if ply == 0 { true } else { false };
        let hash_key = board.hash();
        if !root {
            // TODO
            // if self.is_repetition(hash_key) {
            //     return (-5, None);
            // }

            if board.halfmove_clock() >= 100 {
                return (0, None);
            }

            // Mate distance pruning
            alpha = alpha.max(mated_in(ply));
            beta = beta.min(mate_in(ply + 1));

            if alpha >= beta {
                return (alpha, None);
            }
        }

        if depth == 0 {
            return (eval::evaluate(board), None);
        }

        // Transposition table lookup
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
            self.tables_used += 1;
            match tt_entry.flag {
                Flag::EXACTBOUND => {
                    return (tt_score, tt_move);
                }
                Flag::UPPERBOUND => alpha = alpha.max(tt_score),
                Flag::LOWERBOUND => beta = beta.min(tt_score),
                Flag::NONEBOUND => (),
            }
            if alpha >= beta {
                return (tt_score, tt_move);
            }
        }

        let old_alpha = alpha;
        let mut best_score = -INFINITY;
        let mut moves_done: u32 = 0;
        let mut best_move: Option<Move> = None;
        let mut move_list = Vec::new();
        board.generate_moves(|moves| {
            // Unpack dense move set into move list
            move_list.extend(moves);
            false
        });

        for mv in move_list {
            let mut new_board = board.clone();
            new_board.play(mv);
            self.nodes += 1;
            moves_done += 1;
            self.hash_history.push(hash_key);

            let score = -self
                .absearch(&new_board, -beta, -alpha, depth - 1, ply + 1)
                .0;

            self.hash_history.pop();
            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
            alpha = alpha.max(score);

            // Fail-hard
            if alpha >= beta {
                break;
            }
        }

        // ENDSTATES
        if moves_done == 0 {
            // Mate
            if board.checkers() != BitBoard::EMPTY {
                return (mated_in(ply), None);
            } else {
                // Stalemate
                return (0, None);
            }
        }

        // Calculate bound and save to TT
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

        return (best_score, best_move);
    }
}
