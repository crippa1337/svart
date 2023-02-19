use crate::engine::pv_table::PVTable;
use crate::engine::tt::TTFlag;
use crate::{constants::*, engine::eval, uci::SearchType};
use cozy_chess::{BitBoard, Board, Color, Move, Piece};
use std::cmp::{max, min};
use std::time::Instant;

use super::history::History;
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
    pub history: History,
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
            history: History::new(),
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

            // Mate distance pruning
            let mate_score = MATE - ply as i16;
            if mate_score < beta && alpha >= mate_score {
                return mate_score;
            }
        }

        // Escape condition
        if depth == 0 {
            return self.qsearch(board, alpha, beta, ply);
        }

        // Static eval used for pruning
        let eval;

        /////////////////////////////////
        // Transposition table cut-off //
        /////////////////////////////////

        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key;
        let mut tt_move: Option<Move> = None;
        if tt_hit {
            tt_move = tt_entry.mv;
            let tt_score = self.tt.score_from_tt(tt_entry.score, ply);

            eval = tt_score;

            if !is_pv && tt_entry.depth >= depth {
                assert!(tt_score != NONE);

                match tt_entry.flags {
                    TTFlag::Exact => return tt_score,
                    TTFlag::LowerBound => alpha = max(alpha, tt_score),
                    TTFlag::UpperBound => beta = min(beta, tt_score),
                    _ => unreachable!("Invalid TTFlag!"),
                }

                if alpha >= beta {
                    return tt_score;
                }
            }
        } else {
            eval = eval::evaluate(board);
        }

        let in_check = !board.checkers().is_empty();

        if !is_pv {
            // Null move pruning
            // If we can give the opponent a free move and still cause a beta cutoff,
            // we can safely prune this branch. This does not work in zugzwang positions
            // because then it is always better to give a free move, hence some checks for it are needed.
            if depth >= 3
                && !in_check
                && eval >= beta
                && !self
                    .non_pawn_material(board, board.side_to_move())
                    .is_empty()
            {
                let r = 3 + depth / 4;
                let d = depth.saturating_sub(r);
                let new_board = board.null_move().unwrap();

                let score = -self.pvsearch(&new_board, -beta, -beta + 1, d, ply + 1, false);

                if score >= beta {
                    if score >= TB_WIN_IN_PLY {
                        return beta;
                    }

                    return score;
                }
            }
        }

        /////////////////
        // Search body //
        /////////////////

        let old_alpha = alpha;
        let mut best_score: i16 = -INFINITY;
        let mut best_move: Option<Move> = None;
        let mut move_list = movegen::all_moves(self, board, tt_move, ply);
        let mut quiet_moves: Vec<Move> = vec![];

        // Checkmates and stalemates
        if move_list.is_empty() {
            if in_check {
                return ply as i16 - MATE;
            } else {
                return 0;
            }
        }

        for i in 0..move_list.len() {
            let mv = movegen::pick_move(&mut move_list, i);

            if quiet_move(board, mv) {
                quiet_moves.push(mv);
            }

            let mut new_board = board.clone();
            new_board.play(mv);

            self.game_history.push(new_board.hash()); // Repetition detection
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
                    self.pv_table.store(board, ply, mv);

                    if score >= beta {
                        if quiet_move(board, mv) {
                            self.killers[ply as usize][1] = self.killers[ply as usize][0];
                            self.killers[ply as usize][0] = Some(mv);

                            self.history.update_table(board, mv, quiet_moves, depth);
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
        alpha = max(alpha, stand_pat);
        if stand_pat >= beta {
            return stand_pat;
        }

        let mut captures = movegen::capture_moves(self, board, None, ply);
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
            alpha = max(-INFINITY, prev_eval - delta);
            beta = min(INFINITY, prev_eval + delta);
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
                alpha = max(-INFINITY, score - delta);
            }
            // Search failed high, adjust window
            else if score >= beta {
                beta = min(INFINITY, score + delta);
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

    fn non_pawn_material(&self, board: &Board, color: Color) -> BitBoard {
        let b = board.occupied();
        (b | board.pieces(Piece::Knight)
            | board.pieces(Piece::Bishop)
            | board.pieces(Piece::Rook)
            | board.pieces(Piece::Queen))
            & board.colors(color)
    }
}
