use std::time::Instant;

use crate::engine::tt::TTFlag;
use crate::{constants::*, engine::eval, uci::SearchType};
use cozy_chess::{BitBoard, Board, Move};

use super::movegen::{self};
use super::tt::TT;

pub struct Search {
    pub stop: bool,
    pub search_type: SearchType,
    pub timer: Option<Instant>,
    pub goal_time: Option<u64>,
    pub pv_length: [i16; MAX_PLY as usize],
    pub pv_table: [[Option<Move>; MAX_PLY as usize]; MAX_PLY as usize],
    pub nodes: u32,
    pub tt: TT,
    pub game_history: Vec<u64>,
}

impl Search {
    pub fn new(tt: TT) -> Self {
        Search {
            stop: false,
            search_type: SearchType::Depth(0),
            timer: None,
            goal_time: None,
            pv_length: [0; MAX_PLY as usize],
            pv_table: [[None; MAX_PLY as usize]; MAX_PLY as usize],
            nodes: 0,
            tt,
            game_history: vec![],
        }
    }

    pub fn pvsearch(
        &mut self,
        board: &Board,
        mut alpha: i16,
        mut beta: i16,
        depth: u8,
        ply: i16,
        is_pv: bool,
    ) -> i16 {
        ///////////////////
        // Early returns //
        ///////////////////

        // Every 1024 nodes, check if it's time to stop
        if let (Some(timer), Some(goal)) = (self.timer, self.goal_time) {
            if self.nodes % 1024 == 0 && timer.elapsed().as_millis() as u64 >= goal {
                self.stop = true;
                return 0;
            }
        }

        if self.stop && ply > 0 {
            return 0;
        }

        if ply >= MAX_PLY {
            return eval::evaluate(board);
        }

        // Init PV
        self.pv_length[ply as usize] = ply;
        let hash_key = board.hash();
        let root = ply == 0;

        if !root {
            if board.halfmove_clock() >= 100 {
                return 0;
            }

            // Avoids three-fold repetition blindness - Elo difference: 70.4 +/- 29.8
            if self.repetition(board, hash_key) {
                return 8 - (self.nodes as i16 & 7);
            }
        }

        // Escape condition
        if depth == 0 {
            return self.qsearch(board, alpha, beta, ply);
        }

        /////////////////////////////////
        // Transposition table cut-off //
        /////////////////////////////////

        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key;

        let tt_move: Option<Move> = if tt_hit { tt_entry.mv } else { None };
        let tt_score = if tt_hit {
            self.tt.score_from_tt(tt_entry.score, ply)
        } else {
            NONE
        };

        if !is_pv && tt_hit && tt_entry.depth >= depth {
            debug_assert!(tt_score < NONE);

            match tt_entry.flags {
                TTFlag::Exact => return tt_score,
                TTFlag::LowerBound => alpha = std::cmp::max(alpha, tt_score),
                TTFlag::UpperBound => beta = std::cmp::min(beta, tt_score),
                _ => unreachable!("Invalid TTFlag!"),
            }

            if alpha >= beta {
                return tt_score;
            }
        }

        /////////////////
        // Search body //
        /////////////////

        let old_alpha = alpha;
        let mut best_score: i16 = NEG_INFINITY;
        let mut best_move: Option<Move> = None;
        let mut moves_done: u32 = 0;
        let mut move_list = movegen::all_moves(board, tt_move);

        for i in 0..move_list.len() {
            let mv = movegen::pick_move(&mut move_list, i);

            let mut new_board = board.clone();
            new_board.play(mv);
            self.game_history.push(new_board.hash());

            self.nodes += 1;
            moves_done += 1;

            // Principal Variation Search
            let mut score: i16;
            if moves_done == 1 {
                score = -self.pvsearch(&new_board, -beta, -alpha, depth - 1, ply + 1, is_pv);
            } else {
                score = -self.pvsearch(&new_board, -alpha - 1, -alpha, depth - 1, ply + 1, false);
                if alpha < score && score < beta {
                    score = -self.pvsearch(&new_board, -beta, -alpha, depth - 1, ply + 1, true);
                }
            }

            self.game_history.pop();

            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                    best_move = Some(mv);

                    /////////////////////////////////////////////////////////////
                    // PV LOGIC - https://www.youtube.com/watch?v=LOR-dkAkUyM  //
                    /////////////////////////////////////////////////////////////

                    // Write to PV table
                    let uply = ply as usize;
                    self.pv_table[uply][uply] = Some(mv);

                    // Loop over the next ply
                    for i in (uply + 1)..self.pv_length[uply + 1] as usize {
                        // Copy move from deeper ply into current line
                        self.pv_table[uply][i] = self.pv_table[uply + 1][i];
                    }

                    // Update PV length
                    self.pv_length[uply] = self.pv_length[uply + 1];

                    if score >= beta {
                        break;
                    }
                }
            }
        }

        ///////////////
        // ENDSTATES //
        ///////////////

        // Checkmates and stalemates
        if moves_done == 0 {
            if board.checkers() != BitBoard::EMPTY {
                return mated_in(ply);
            } else {
                return 0;
            }
        }

        // Storing to TT
        let flag;
        if best_score >= beta {
            flag = TTFlag::LowerBound;
        } else if best_score != old_alpha {
            flag = TTFlag::Exact;
        } else {
            flag = TTFlag::UpperBound;
        }

        if !self.stop {
            self.tt
                .store(hash_key, best_move, best_score, depth, flag, ply);
        }

        best_score
    }

    fn qsearch(&mut self, board: &Board, mut alpha: i16, beta: i16, ply: i16) -> i16 {
        // Early returns
        if let (Some(timer), Some(goal)) = (self.timer, self.goal_time) {
            if self.nodes % 1024 == 0 && timer.elapsed().as_millis() as u64 >= goal {
                self.stop = true;
                return 0;
            }
        }

        if self.stop && ply > 0 {
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

        let mut captures = movegen::capture_moves(board);
        let mut best_score = stand_pat;

        for i in 0..captures.len() {
            let mv = movegen::pick_move(&mut captures, i);

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

        best_score
    }

    pub fn iterative_deepening(&mut self, board: &Board, st: SearchType) {
        let depth: u8;
        match st {
            SearchType::Time(t) => {
                depth = MAX_PLY as u8;
                self.timer = Some(Instant::now());
                self.goal_time = Some(t - TIME_OVERHEAD);
            }
            SearchType::Infinite => {
                depth = MAX_PLY as u8;
            }
            SearchType::Depth(d) => depth = d + 1,
        };

        let info_timer = Instant::now();
        let mut best_move: Option<Move> = None;

        let mut score: i16 = 0;
        let mut alpha = NEG_INFINITY;
        let mut beta = INFINITY;

        for d in 1..depth {
            let mut delta = 40;
            if d <= 3 {
                score = self.pvsearch(board, alpha, beta, d, 0, true);
            } else {
                loop {
                    score = self.pvsearch(board, alpha, beta, d, 0, true);

                    // Search was within window
                    if (score > alpha && score < beta) || self.stop {
                        break;
                    }

                    // Search failed low, adjust window and reset depth
                    if score <= alpha {
                        beta = (alpha + beta) / 2;
                        alpha = (alpha - delta).max(NEG_INFINITY);
                    }
                    // Search failed high, adjust window and reset depth
                    else if score >= beta {
                        beta = INFINITY.min(beta + delta);
                    }

                    delta = delta + delta / 2;
                }
            };

            if self.stop && d > 1 {
                break;
            }

            best_move = self.pv_table[0][0];

            println!(
                "info depth {} score {} nodes {} time {} pv{}",
                d,
                self.parse_score(score),
                self.nodes,
                info_timer.elapsed().as_millis(),
                self.show_pv()
            );
        }

        println!("bestmove {}", best_move.unwrap());
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

        pv
    }

    // Parse score to UCI standard
    pub fn parse_score(&self, score: i16) -> String {
        assert!(score < NONE);
        let print_score: String;
        if score >= MATE_IN {
            print_score = format!("mate {}", (((MATE - score) / 2) + ((MATE - score) & 1)));
        } else if score <= MATED_IN {
            print_score = format!("mate {}", -(((MATE + score) / 2) + ((MATE + score) & 1)));
        } else {
            print_score = format!("cp {score}");
        }

        print_score
    }

    // Tantabaus repetition detection
    fn repetition(&self, board: &Board, hash: u64) -> bool {
        for key in self
            .game_history
            .iter()
            .rev()
            .take(board.halfmove_clock() as usize + 1)
            .skip(2)
            .step_by(2)
        {
            if *key == hash {
                return true;
            }
        }

        false
    }
}
