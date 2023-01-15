use std::time::Instant;

use crate::{constants::*, engine::eval, uci::SearchType};
use cozy_chess::{BitBoard, Board, Move};

use super::movegen;

pub struct Search {
    pub stop: bool,
    pub search_type: SearchType,
    pub timer: Option<Instant>,
    pub goal_time: Option<u64>,
    pub pv_length: [i32; MAX_PLY as usize],
    pub pv_table: [[Option<Move>; MAX_PLY as usize]; MAX_PLY as usize],
    pub nodes: u32,
}

impl Search {
    pub fn new() -> Self {
        return Search {
            stop: false,
            search_type: SearchType::Depth(0),
            timer: None,
            goal_time: None,
            pv_length: [0; MAX_PLY as usize],
            pv_table: [[None; MAX_PLY as usize]; MAX_PLY as usize],
            nodes: 0,
        };
    }

    pub fn absearch(
        &mut self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        depth: u8,
        ply: i32,
    ) -> i32 {
        // Early returns
        if self.nodes % 1024 == 0 && self.timer.is_some() && self.goal_time.is_some() {
            let time = self.timer.as_ref().unwrap();
            let goal = self.goal_time.unwrap();
            if time.elapsed().as_millis() as u64 >= goal {
                self.stop = true;
                return 0;
            }
        }

        if self.stop {
            return 0;
        }

        if ply >= MAX_PLY {
            return eval::evaluate(board);
        }

        // init PV
        self.pv_length[ply as usize] = ply;

        // Escape condition
        if depth == 0 {
            return self.qsearch(board, alpha, beta, ply);
        }

        let mut best_score = NEG_INFINITY;
        let mut moves_done: u32 = 0;
        let move_list = movegen::all_moves(board);

        for mv in move_list {
            let mut new_board = board.clone();
            new_board.play(mv);
            self.nodes += 1;
            moves_done += 1;

            let score = -self.absearch(&new_board, -beta, -alpha, depth - 1, ply + 1);

            if score > best_score {
                best_score = score;

                if score > alpha {
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

        return best_score;
    }

    fn qsearch(&mut self, board: &Board, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        // Early returns
        if self.nodes % 1024 == 0 && self.timer.is_some() && self.goal_time.is_some() {
            let time = self.timer.as_ref().unwrap();
            let goal = self.goal_time.unwrap();
            if time.elapsed().as_millis() as u64 >= goal {
                self.stop = true;
                return 0;
            }
        }

        if self.stop {
            return 0;
        }

        if ply >= MAX_PLY {
            return eval::evaluate(board);
        }

        let stand_pat = eval::evaluate(board);
        if stand_pat >= beta {
            return stand_pat;
        }
        alpha = alpha.max(stand_pat);

        let captures = movegen::capture_moves(board);
        let mut best_score = stand_pat;

        for mv in captures {
            let mut new_board = board.clone();
            new_board.play(mv);
            self.nodes += 1;

            let score = -self.qsearch(&new_board, -beta, -alpha, ply + 1);

            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;

                    if score >= beta {
                        break;
                    }
                }
            }
        }

        return best_score;
    }

    pub fn iterative_deepening(&mut self, board: &Board, st: SearchType) {
        let depth: u8;
        match st {
            SearchType::Time(t) => {
                depth = MAX_PLY as u8;
                self.timer = Some(Instant::now());
                // Small overhead to make sure we don't go over time
                self.goal_time = Some(t - 3);
            }
            SearchType::Infinite => {
                depth = MAX_PLY as u8;
            }
            SearchType::Depth(d) => depth = d,
        };

        let mut best_move: Option<Move> = None;

        for d in 1..depth {
            let score = self.absearch(board, -INFINITY, INFINITY, d as u8, 0);

            if self.stop {
                break;
            }

            best_move = self.pv_table[0][0];

            println!(
                "info depth {} {} nodes {} pv{}",
                d,
                self.show_score(score),
                self.nodes,
                self.show_pv()
            );
        }

        println!("bestmove {}", best_move.unwrap().to_string());
    }

    pub fn show_pv(&self) -> String {
        let mut pv = String::new();
        for i in 0..self.pv_length[0] {
            if self.pv_table[0][i as usize].is_none() {
                break;
            }
            pv.push(' ');
            pv.push_str(&self.pv_table[0][i as usize].unwrap().to_string());
        }

        return pv;
    }

    pub fn show_score(&self, mut score: i32) -> String {
        let print_score: String;
        // check mate score
        if score > MATE_IN || score < MATED_IN {
            let plies_to_mate = MATE - score.abs();
            let moves_to_mate = (plies_to_mate + 1) / 2;
            if score > 0 {
                score = moves_to_mate;
            } else {
                score = -moves_to_mate;
            }
            print_score = format!("mate {}", score);
        } else {
            print_score = format!("cp {}", score / 100);
        }

        print_score
    }
}
