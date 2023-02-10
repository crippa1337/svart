use crate::engine::pv_table::PVTable;
use crate::engine::tt::TTFlag;
use crate::{constants::*, engine::eval, uci::SearchType};
use cozy_chess::{BitBoard, Board, Move};
use std::time::Instant;

use super::movegen::{self};
use super::tt::TT;

pub struct Search {
    pub stop: bool,
    pub search_type: SearchType,
    pub timer: Option<Instant>,
    pub goal_time: Option<u64>,
    pub pv_table: PVTable,
    pub nodes: u32,
    pub tt: TT,
    pub game_history: Vec<u64>,
    pub killers: [[Option<Move>; 2]; MAX_PLY as usize],
    pub history: [[[u16; 64]; 64]; 2],
}

impl Search {
    pub fn new(tt: TT) -> Self {
        Search {
            stop: false,
            search_type: SearchType::Depth(0),
            timer: None,
            goal_time: None,
            pv_table: PVTable::new(),
            nodes: 0,
            tt,
            game_history: vec![],
            killers: [[None; 2]; MAX_PLY as usize],
            history: [[[0; 64]; 64]; 2],
        }
    }

    pub fn pvsearch(
        &mut self,
        board: &Board,
        mut alpha: i16,
        mut beta: i16,
        depth: u8,
        ply: u8,
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
        self.pv_table.length[ply as usize] = ply;
        let hash_key = board.hash();
        let root = ply == 0;

        if !root {
            if board.halfmove_clock() >= 100 {
                return 0;
            }

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
            assert!(tt_score < NONE);

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
        let mut best_score: i16 = -INFINITY;
        let mut best_move: Option<Move> = None;
        let mut move_list = movegen::all_moves(self, board, tt_move, ply);

        // Checkmates and stalemates
        if move_list.is_empty() {
            if board.checkers() != BitBoard::EMPTY {
                return ply as i16 - MATE;
            } else {
                return 0;
            }
        }

        for i in 0..move_list.len() {
            let mv = movegen::pick_move(&mut move_list, i);

            let mut new_board = board.clone();
            new_board.play(mv);
            self.game_history.push(new_board.hash());

            self.nodes += 1;

            // Principal Variation Search
            let mut score: i16;
            if i == 0 {
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
                    self.pv_table.store(ply, mv);

                    if score >= beta {
                        if quiet_move(board, mv) {
                            self.killers[ply as usize][1] = self.killers[ply as usize][0];
                            self.killers[ply as usize][0] = Some(mv);

                            self.history[board.side_to_move() as usize][mv.to as usize]
                                [mv.from as usize] += (depth * depth) as u16;
                        }

                        break;
                    }
                }
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

    fn qsearch(&mut self, board: &Board, mut alpha: i16, beta: i16, ply: u8) -> i16 {
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
                depth = MAX_PLY;
                self.timer = Some(Instant::now());
                self.goal_time = Some(t - TIME_OVERHEAD);
            }
            SearchType::Infinite => {
                depth = MAX_PLY;
            }
            SearchType::Depth(d) => depth = d + 1,
        };

        let info_timer = Instant::now();
        let mut best_move: Option<Move> = None;

        let mut score: i16 = 0;

        for d in 1..depth {
            score = self.aspiration_window(board, score, d);

            // Search wasn't complete, do not update best move with garbage
            if self.stop && d > 1 {
                break;
            }

            best_move = self.pv_table.table[0][0];

            println!(
                "info depth {} score {} nodes {} time {} pv{}",
                d,
                self.format_score(score),
                self.nodes,
                info_timer.elapsed().as_millis(),
                self.pv_table.pv_string()
            );
        }

        println!("bestmove {}", best_move.unwrap());
    }

    fn aspiration_window(&mut self, board: &Board, prev_eval: i16, depth: u8) -> i16 {
        let mut score: i16;

        // Window size
        let mut delta = 50;

        // Window bounds
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;

        if depth >= 5 {
            alpha = (prev_eval - delta).max(-INFINITY);
            beta = (prev_eval + delta).min(INFINITY);
        }

        loop {
            score = self.pvsearch(board, alpha, beta, depth, 0, true);

            // This result won't be used
            if self.stop {
                return 0;
            }

            // Search failed low, adjust window
            if score <= alpha {
                beta = (alpha + beta) / 2;
                alpha = (score - delta).max(-INFINITY);
            }
            // Search failed high, adjust window
            else if score >= beta {
                beta = (score + delta).min(INFINITY);
            }
            // Search succeeded
            else {
                return score;
            }

            // Always increase window size on search failure
            delta += delta / 2;
            assert!(alpha >= -INFINITY && beta <= INFINITY);
        }
    }

    pub fn format_score(&self, score: i16) -> String {
        assert!(score < NONE);
        let print_score: String;
        if score >= MATE_IN {
            print_score = format!("mate {}", (((MATE - score) / 2) + ((MATE - score) & 1)));
        } else if score <= -MATE_IN {
            print_score = format!("mate {}", -(((MATE + score) / 2) + ((MATE + score) & 1)));
        } else {
            print_score = format!("cp {score}");
        }

        print_score
    }

    // Tantabus inspired repetition detection
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

#[cfg(test)]
mod tests {
    use crate::constants::*;
    use crate::engine::search::Search;
    use crate::engine::tt::TT;
    use cozy_chess::{Board, Move, Square};

    #[test]
    fn mate_in_1() {
        let board = Board::from_fen(
            "4r2k/1p3rbp/2p1N1p1/p3n3/P2NB1nq/1P6/4R1P1/B1Q2RK1 b - - 4 32",
            false,
        )
        .unwrap();
        let mut search = Search::new(TT::new(32));
        let score = search.pvsearch(&board, -INFINITY, INFINITY, 2, 0, true);
        assert_eq!(
            search.pv_table.table[0][0],
            Some(Move {
                from: Square::H4,
                to: Square::H2,
                promotion: None,
            })
        );
        assert_eq!(score, MATE - 1);

        let board = Board::from_fen(
            "r3qr2/p4k2/bpn1pp1Q/3pP3/P2Nn3/1Pb1RNPP/5PB1/3R2K1 w - - 1 24",
            false,
        )
        .unwrap();
        let mut search = Search::new(TT::new(32));
        let score = search.pvsearch(&board, -INFINITY, INFINITY, 2, 0, true);
        assert_eq!(
            search.pv_table.table[0][0],
            Some(Move {
                from: Square::H6,
                to: Square::H7,
                promotion: None,
            })
        );
        assert_eq!(score, MATE - 1);

        let board = Board::from_fen(
            "r3nk2/4r1b1/q2p4/1P5p/3P1p1P/4P3/1BQN1P1R/1KR5 b - - 0 33",
            false,
        )
        .unwrap();
        let mut search = Search::new(TT::new(32));
        let score = search.pvsearch(&board, -INFINITY, INFINITY, 2, 0, true);
        assert_eq!(
            search.pv_table.table[0][0],
            Some(Move {
                from: Square::A6,
                to: Square::A2,
                promotion: None,
            })
        );
        assert_eq!(score, MATE - 1);
    }
}
