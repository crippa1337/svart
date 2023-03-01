use crate::{constants::*, uci::SearchType};
use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};
use once_cell::sync::Lazy;
use std::cmp::{max, min};
use std::time::Instant;

use super::{
    eval,
    history::History,
    lmr::LMRTable,
    movegen,
    pv_table::PVTable,
    stat_vec::StaticVec,
    tt::{TTFlag, TT},
};

static LMR: Lazy<LMRTable> = Lazy::new(LMRTable::new);
const RFP_MARGIN: i16 = 75;

pub struct Search {
    pub stop: bool,
    pub search_type: SearchType,
    pub timer: Option<Instant>,
    pub goal_time: Option<u64>,
    pub nodes: u32,
    pub seldepth: u8,
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
            nodes: 0,
            seldepth: 0,
            tt,
            game_history: vec![],
            killers: [[None; 2]; MAX_PLY as usize],
            history: History::new(),
        }
    }

    // Zero Window Search - A way to reduce the search space in alpha-beta like search algorithms,
    // to perform a boolean test, whether a move produces a worse or better score than a passed value.
    // (https://www.chessprogramming.org/Null_Window)
    fn zw_search(
        &mut self,
        board: &Board,
        pv: &mut PVTable,
        alpha: i16,
        beta: i16,
        depth: u8,
        ply: u8,
    ) -> i16 {
        self.pvsearch::<false>(board, pv, alpha, beta, depth, ply)
    }

    pub fn pvsearch<const PV: bool>(
        &mut self,
        board: &Board,
        pv: &mut PVTable,
        mut alpha: i16,
        beta: i16,
        depth: u8,
        ply: u8,
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

        self.seldepth = max(self.seldepth, ply);
        pv.length = 0;
        let mut old_pv = PVTable::new();

        match board.status() {
            GameStatus::Won => return ply as i16 - MATE,
            GameStatus::Drawn => return 8 - (self.nodes as i16 & 7),
            _ => (),
        }

        let hash_key = board.hash();
        let root = ply == 0;

        if !root {
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
        let tt_hit = tt_entry.key == hash_key as u16;
        let mut tt_move: Option<Move> = None;
        if tt_hit {
            let tt_score = self.tt.score_from_tt(tt_entry.score, ply);
            tt_move = tt_entry.mv;
            // Use the TT score if available since eval is expensive
            eval = tt_score;

            if !PV && tt_entry.depth >= depth {
                assert!(tt_score != NONE && tt_entry.flag != TTFlag::None);

                if (tt_entry.flag == TTFlag::Exact)
                    || (tt_entry.flag == TTFlag::LowerBound && tt_score >= beta)
                    || (tt_entry.flag == TTFlag::UpperBound && tt_score <= alpha)
                {
                    return tt_score;
                }
            }
        } else {
            eval = eval::evaluate(board);
        }

        let in_check = !board.checkers().is_empty();

        ///////////////////////////////////
        // Pre-search pruning techniques //
        ///////////////////////////////////

        if !PV {
            // Null Move Pruning (NMP)
            // If we can give the opponent a free move and still cause a beta cutoff,
            // we can safely prune this node. This does not work in zugzwang positions
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

                let score = -self.zw_search(&new_board, &mut old_pv, -beta, -beta + 1, d, ply + 1);

                if score >= beta {
                    if score >= TB_WIN_IN_PLY {
                        return beta;
                    }

                    return score;
                }
            }

            // Reverse Futility Pruning (RFP)
            // If static eval plus a margin can beat beta, then we can safely prune this node.
            // The margin is multiplied by depth to make it harder to prune at higher depths
            // as pruning there can be inaccurate as it prunes a large amount of potential nodes
            // and static eval isn't the most accurate.
            if depth < 9 && eval >= beta + RFP_MARGIN * depth as i16 {
                return eval;
            }
        }

        /////////////////
        // Search body //
        /////////////////

        let old_alpha = alpha;
        let mut best_score: i16 = -INFINITY;
        let mut best_move: Option<Move> = None;
        let mut move_list = movegen::all_moves(self, board, tt_move, ply);
        let mut quiet_moves = StaticVec::<Option<Move>, MAX_MOVES_POSITION>::new(None);
        let lmr_depth = if PV { 4 } else { 2 };

        for i in 0..move_list.len() {
            let mv = movegen::pick_move(&mut move_list, i);

            if quiet_move(board, mv) {
                quiet_moves.push(Some(mv));
            }

            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            self.game_history.push(new_board.hash()); // Repetition detection
            self.nodes += 1;
            let gives_check = !new_board.checkers().is_empty();

            // Principal Variation Search
            let mut score: i16;
            if i == 0 {
                score = -self.pvsearch::<PV>(
                    &new_board,
                    &mut old_pv,
                    -beta,
                    -alpha,
                    depth - 1,
                    ply + 1,
                );
            } else {
                // Late Move Reduction (LMR)
                // Assuming our move ordering is good, later moves will be worse
                // and can be searched with a reduced depth, if they beat alpha
                // we do a full re-search.
                let r = if depth >= 3 && i > lmr_depth {
                    // Probe LMR table (src/lmr.rs)
                    let mut r = LMR.reduction(depth, i) as u8;

                    // Bonus for non PV nodes
                    r += u8::from(!PV);

                    // Malus for capture moves and checks
                    r -= u8::from(capture_move(board, mv));
                    r -= u8::from(gives_check);

                    // Clamping
                    r = r.min(depth - 1);
                    r.max(1)
                } else {
                    1
                };

                score = -self.zw_search(
                    &new_board,
                    &mut old_pv,
                    -alpha - 1,
                    -alpha,
                    depth - r,
                    ply + 1,
                );
                if alpha < score && score < beta {
                    score = -self.pvsearch::<PV>(
                        &new_board,
                        &mut old_pv,
                        -beta,
                        -alpha,
                        depth - 1,
                        ply + 1,
                    );
                }
            }

            self.game_history.pop();

            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                    best_move = Some(mv);
                    pv.store(board, mv, &old_pv);

                    if score >= beta {
                        if quiet_move(board, mv) {
                            self.killers[ply as usize][1] = self.killers[ply as usize][0];
                            self.killers[ply as usize][0] = Some(mv);

                            // Update best move with a positive bonus
                            self.history.update_table::<true>(board, mv, depth);
                            // Update all other quiet moves with a negative bonus
                            let qi = quiet_moves.as_slice();
                            let qi = &qi[..quiet_moves.len() - 1];
                            for qm in qi {
                                self.history
                                    .update_table::<false>(board, qm.unwrap(), depth);
                            }
                        }

                        break;
                    }
                }
            }
        }

        // Storing to TT
        let flag = if best_score >= beta {
            TTFlag::LowerBound
        } else if best_score != old_alpha {
            TTFlag::Exact
        } else {
            TTFlag::UpperBound
        };

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

        self.seldepth = max(self.seldepth, ply);
        let is_pv = (beta - alpha) > 1;
        let stand_pat = eval::evaluate(board);
        alpha = max(alpha, stand_pat);
        if stand_pat >= beta {
            return stand_pat;
        }

        let hash_key = board.hash();
        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key as u16;
        let mut tt_move: Option<Move> = None;
        if tt_hit && !is_pv && tt_entry.flag != TTFlag::None {
            let tt_score = self.tt.score_from_tt(tt_entry.score, ply);
            tt_move = tt_entry.mv;

            assert!(tt_score != NONE);

            if (tt_entry.flag == TTFlag::Exact)
                || (tt_entry.flag == TTFlag::LowerBound && tt_score >= beta)
                || (tt_entry.flag == TTFlag::UpperBound && tt_score <= alpha)
            {
                return tt_score;
            }
        }

        let mut captures = movegen::capture_moves(self, board, tt_move, ply);
        let mut best_score = stand_pat;
        let mut best_move: Option<Move> = None;

        for i in 0..captures.len() {
            let mv = movegen::pick_move(&mut captures, i);
            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            self.nodes += 1;

            let score = -self.qsearch(&new_board, -beta, -alpha, ply + 1);

            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                    best_move = Some(mv);

                    if score >= beta {
                        break;
                    }
                }
            }
        }

        let flag = if best_score >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::UpperBound
        };

        if !self.stop {
            self.tt.store(hash_key, best_move, best_score, 0, flag, ply);
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
            SearchType::Depth(d) => depth = d,
        };

        let info_timer = Instant::now();
        let mut best_move: Option<Move> = None;

        let mut score: i16 = 0;
        let mut pv = PVTable::new();

        for d in 1..depth + 1 {
            self.seldepth = 0;
            score = self.aspiration_window(board, &mut pv, score, d);

            // Search wasn't complete, do not update best move with garbage
            if self.stop && d > 1 {
                break;
            }

            best_move = pv.table[0];

            println!(
                "info depth {} seldepth {} score {} nodes {} time {} pv{}",
                d,
                self.seldepth,
                self.format_score(score),
                self.nodes,
                info_timer.elapsed().as_millis(),
                pv.pv_string()
            );
        }

        println!("bestmove {}", best_move.unwrap());
    }

    fn aspiration_window(
        &mut self,
        board: &Board,
        pv: &mut PVTable,
        prev_eval: i16,
        depth: u8,
    ) -> i16 {
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
            score = self.pvsearch::<true>(board, pv, alpha, beta, depth, 0);

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
        (board.pieces(Piece::Knight)
            | board.pieces(Piece::Bishop)
            | board.pieces(Piece::Rook)
            | board.pieces(Piece::Queen))
            & board.colors(color)
    }
}
