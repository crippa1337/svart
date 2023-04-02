use crate::{constants::*, uci::handler::SearchType};
use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};
use once_cell::sync::Lazy;
use std::time::Instant;

use super::movegen::Picker;
use super::nnue::inference::NNUEState;
use super::position::{board_default, is_capture, is_quiet, play_move};
use super::{
    history::History,
    lmr::LMRTable,
    movegen,
    pv_table::PVTable,
    stat_vec::StaticVec,
    tt::{TTFlag, TT},
};

static LMR: Lazy<LMRTable> = Lazy::new(LMRTable::new);
const RFP_MARGIN: i32 = 75;
const LMP_TABLE: [usize; 4] = [0, 5, 8, 18];

pub struct Search {
    pub stop: bool,
    pub search_type: SearchType,
    pub timer: Option<Instant>,
    pub goal_time: Option<u64>,
    pub nodes: u64,
    pub seldepth: i32,
    pub tt: TT,
    pub game_history: Vec<u64>,
    pub killers: [[Option<Move>; 2]; MAX_PLY as usize],
    pub history: History,
    pub nnue: Box<NNUEState>,
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
            nnue: board_default().1,
        }
    }

    /*
        Zero Window Search - A way to reduce the search space in alpha-beta like search algorithms,
        to perform a boolean test, whether a move produces a worse or better score than a passed value.
        (https://www.chessprogramming.org/Null_Window)
    */
    #[must_use]
    fn zw_search(
        &mut self,
        board: &Board,
        pv: &mut PVTable,
        alpha: i32,
        beta: i32,
        depth: i32,
        ply: i32,
    ) -> i32 {
        self.pvsearch::<false>(board, pv, alpha, beta, depth, ply)
    }

    #[must_use]
    pub fn pvsearch<const PV: bool>(
        &mut self,
        board: &Board,
        pv: &mut PVTable,
        mut alpha: i32,
        beta: i32,
        mut depth: i32,
        ply: i32,
    ) -> i32 {
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

        let stm = board.side_to_move();

        if ply >= MAX_PLY {
            return self.nnue.evaluate(stm);
        }

        let hash_key = board.hash();
        self.tt.prefetch(hash_key);
        self.seldepth = self.seldepth.max(ply);
        depth = depth.max(0);
        let mut old_pv = PVTable::new();
        pv.length = 0;

        match board.status() {
            GameStatus::Won => return ply - MATE,
            GameStatus::Drawn => return 8 - (self.nodes as i32 & 7),
            _ => (),
        }

        let root = ply == 0;

        if !root {
            if self.repetition(board, hash_key) {
                return 8 - (self.nodes as i32 & 7);
            }

            // Mate distance pruning
            let mate_alpha = alpha.max(ply - MATE);
            let mate_beta = beta.min(MATE - (ply + 1));
            if mate_alpha >= mate_beta {
                return mate_alpha;
            }
        }

        // Escape condition
        if depth == 0 {
            return self.qsearch::<PV>(board, alpha, beta, ply);
        }

        // Static eval used for pruning
        let eval;

        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key as u16;
        let mut tt_move: Option<Move> = None;
        if tt_hit {
            // Use the TT score if available since eval is expensive
            // and any score from the TT is better than the static eval
            let tt_score = self.tt.score_from_tt(tt_entry.score, ply) as i32;
            eval = tt_score;
            tt_move = tt_entry.mv;

            if !PV && i32::from(tt_entry.depth) >= depth {
                debug_assert!(tt_score != NONE && tt_entry.flag != TTFlag::None);

                if (tt_entry.flag == TTFlag::Exact)
                    || (tt_entry.flag == TTFlag::LowerBound && tt_score >= beta)
                    || (tt_entry.flag == TTFlag::UpperBound && tt_score <= alpha)
                {
                    return tt_score;
                }
            }
        } else {
            eval = self.nnue.evaluate(stm);
        }

        let in_check = !board.checkers().is_empty();

        if !PV && !in_check {
            /*
                Null Move Pruning (NMP)
                If we can give the opponent a free move and still cause a beta cutoff,
                we can safely prune this node. This does not work in zugzwang positions
                because then it is always better to give a free move, hence some checks for it are needed.
            */
            if depth >= 3 && eval >= beta && !self.non_pawn_material(board, stm).is_empty() {
                let r = 3 + depth / 3 + 3.min((eval.saturating_sub(beta)) / 200);
                let new_b = board.null_move().unwrap();

                let score =
                    -self.zw_search(&new_b, &mut old_pv, -beta, -beta + 1, depth - r, ply + 1);

                if score >= beta {
                    if score >= TB_WIN_IN_PLY {
                        return beta;
                    }

                    return score;
                }
            }

            /*
                Reverse Futility Pruning (RFP)
                If static eval plus a margin can beat beta, then we can safely prune this node.
                The margin is multiplied by depth to make it harder to prune at higher depths
                as pruning there can be inaccurate as it prunes a large amount of potential nodes
                and static eval isn't the most accurate.
            */
            if depth < 9 && eval >= beta + RFP_MARGIN * depth {
                return eval;
            }
        }

        let old_alpha = alpha;
        let mut best_score = -INFINITY;
        let mut best_move: Option<Move> = None;
        let mut moves_played = 0;

        let move_list = movegen::all_moves(self, board, tt_move, ply);
        let mut quiet_moves = StaticVec::<Option<Move>, MAX_MOVES_POSITION>::new(None);
        let mut picker = Picker::new(move_list);

        let lmr_depth = if PV { 5 } else { 3 };
        let mut quiets_checked = 0;
        let quiets_to_check = match depth {
            d @ 1..=3 => LMP_TABLE[d as usize],
            _ => MAX_MOVES_POSITION,
        };

        while let Some(mv) = picker.pick_move() {
            let is_quiet = is_quiet(board, mv);
            if is_quiet {
                quiets_checked += 1;

                // Late Move Pruning (LMP)
                // If we have searched too many moves, we stop searching here
                if !PV && !in_check && quiets_checked >= quiets_to_check {
                    break;
                }

                quiet_moves.push(Some(mv));
            }

            let mut new_b = board.clone();
            play_move(&mut new_b, &mut self.nnue, mv);

            moves_played += 1;
            self.game_history.push(board.hash());
            self.nodes += 1;
            let gives_check = !board.checkers().is_empty();

            let mut score: i32;
            if moves_played == 1 {
                score =
                    -self.pvsearch::<PV>(&new_b, &mut old_pv, -beta, -alpha, depth - 1, ply + 1);
            } else {
                /*
                    Late Move Reduction (LMR)
                    Assuming our move ordering is good, later moves will be worse
                    and can be searched with a reduced depth, if they beat alpha
                    we do a full re-search.
                */
                let r = if depth >= 3 && moves_played > lmr_depth {
                    // Probe LMR table (src/lmr.rs)
                    let mut r = LMR.reduction(depth, moves_played);

                    // Bonus for non PV nodes
                    r += i32::from(!PV);

                    // Malus for capture moves and checks
                    r -= i32::from(is_capture(board, mv));
                    r -= i32::from(gives_check);

                    r.clamp(1, depth - 1)
                } else {
                    1
                };

                score =
                    -self.zw_search(&new_b, &mut old_pv, -alpha - 1, -alpha, depth - r, ply + 1);

                if alpha < score && score < beta {
                    score = -self.pvsearch::<PV>(
                        &new_b,
                        &mut old_pv,
                        -beta,
                        -alpha,
                        depth - 1,
                        ply + 1,
                    );
                }
            }

            self.game_history.pop();
            self.nnue.pop();

            if score <= best_score {
                continue;
            }
            best_score = score;

            if score <= alpha {
                continue;
            }
            // New best move
            alpha = score;
            best_move = Some(mv);
            pv.store(board, mv, &old_pv);

            // Fail-high
            if score >= beta {
                if is_quiet {
                    // Killer moves
                    self.killers[ply as usize][1] = self.killers[ply as usize][0];
                    self.killers[ply as usize][0] = Some(mv);

                    // History Heuristic
                    self.history.update_table::<true>(board, mv, depth);
                    let qi = quiet_moves.as_slice();
                    let qi = &qi[..quiet_moves.len() - 1];
                    for qm in qi {
                        self.history.update_table::<false>(board, qm.unwrap(), depth);
                    }
                }

                break;
            }
        }

        self.tt.prefetch(hash_key);

        let flag = if best_score >= beta {
            TTFlag::LowerBound
        } else if best_score != old_alpha {
            TTFlag::Exact
        } else {
            TTFlag::UpperBound
        };

        debug_assert!((-INFINITY..=INFINITY).contains(&best_score));

        if !self.stop {
            self.tt.store(hash_key, best_move, best_score as i16, depth as u8, flag, ply);
        }

        best_score
    }

    #[must_use]
    fn qsearch<const PV: bool>(
        &mut self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        ply: i32,
    ) -> i32 {
        if let (Some(timer), Some(goal)) = (self.timer, self.goal_time) {
            if self.nodes % 1024 == 0 && timer.elapsed().as_millis() as u64 >= goal {
                self.stop = true;
                return 0;
            }
        }

        if self.stop && ply > 0 {
            return 0;
        }

        let stm = board.side_to_move();

        if ply >= MAX_PLY {
            return self.nnue.evaluate(stm);
        }

        let hash_key = board.hash();
        self.tt.prefetch(hash_key);
        self.seldepth = self.seldepth.max(ply);

        let stand_pat = self.nnue.evaluate(stm);
        alpha = alpha.max(stand_pat);
        if stand_pat >= beta {
            return stand_pat;
        }

        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key as u16;
        let mut tt_move: Option<Move> = None;

        if tt_hit && !PV && tt_entry.flag != TTFlag::None {
            let tt_score = self.tt.score_from_tt(tt_entry.score, ply) as i32;
            tt_move = tt_entry.mv;

            debug_assert!(tt_score != NONE);

            if (tt_entry.flag == TTFlag::Exact)
                || (tt_entry.flag == TTFlag::LowerBound && tt_score >= beta)
                || (tt_entry.flag == TTFlag::UpperBound && tt_score <= alpha)
            {
                return tt_score;
            }
        }

        let captures = movegen::capture_moves(self, board, tt_move, ply);
        let mut picker = Picker::new(captures);
        let mut best_score = stand_pat;
        let mut best_move: Option<Move> = None;

        while let Some(mv) = picker.pick_move() {
            let mut new_b = board.clone();
            play_move(&mut new_b, &mut self.nnue, mv);

            self.nodes += 1;

            let score = -self.qsearch::<PV>(&new_b, -beta, -alpha, ply + 1);

            self.nnue.pop();

            if score <= best_score {
                continue;
            }
            best_score = score;

            if score <= alpha {
                continue;
            }
            alpha = score;
            best_move = Some(mv);

            if score >= beta {
                break;
            }
        }

        self.tt.prefetch(hash_key);

        let flag = if best_score >= beta { TTFlag::LowerBound } else { TTFlag::UpperBound };

        if !self.stop {
            self.tt.store(hash_key, best_move, best_score as i16, 0, flag, ply);
        }

        best_score
    }

    pub fn iterative_deepening(&mut self, board: &Board, st: SearchType) {
        let depth: i32;
        let mut goal_nodes: Option<u64> = None;
        match st {
            SearchType::Time(t) => {
                depth = MAX_PLY;
                self.timer = Some(Instant::now());
                self.goal_time = Some(t - TIME_OVERHEAD);
            }
            SearchType::Infinite => {
                depth = MAX_PLY;
            }
            SearchType::Depth(d) => depth = d.min(MAX_PLY),
            SearchType::Nodes(n) => {
                depth = MAX_PLY;
                goal_nodes = Some(n);
            }
        };

        let info_timer = Instant::now();
        let mut best_move: Option<Move> = None;

        let mut score = 0;
        let mut pv = PVTable::new();

        for d in 1..=depth {
            self.seldepth = 0;
            score = self.aspiration_window(board, &mut pv, score, d);

            if let Some(nodes) = goal_nodes {
                if self.nodes >= nodes {
                    break;
                }
            }

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
        prev_eval: i32,
        mut depth: i32,
    ) -> i32 {
        let mut score: i32;
        let init_depth = depth;

        // Window size
        let mut delta = 25;

        // Window bounds
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;

        if depth >= 5 {
            alpha = (-INFINITY).max(prev_eval - delta);
            beta = (INFINITY).min(prev_eval + delta);
        }

        loop {
            score = self.pvsearch::<true>(board, pv, alpha, beta, depth, 0);

            if self.stop {
                return 0;
            }

            // Search failed low
            if score <= alpha {
                beta = (alpha + beta) / 2;
                alpha = (-INFINITY).max(score - delta);
                depth = init_depth;
            }
            // Search failed high
            else if score >= beta {
                beta = (INFINITY).min(score + delta);

                depth -= i32::from(score.abs() < MATE_IN);
            }
            // Search succeeded
            else {
                return score;
            }

            delta += delta / 2;
            debug_assert!(alpha >= -INFINITY && beta <= INFINITY);
        }
    }

    pub fn format_score(&self, score: i32) -> String {
        debug_assert!(score < NONE);
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

    pub fn reset(&mut self) {
        self.stop = false;
        self.search_type = SearchType::Depth(0);
        self.timer = None;
        self.goal_time = None;
        self.nodes = 0;
        self.seldepth = 0;
        self.killers = [[None; 2]; MAX_PLY as usize];
        self.history.age_table();
        self.tt.age();
    }

    pub fn hard_reset(&mut self) {
        self.stop = false;
        self.search_type = SearchType::Depth(0);
        self.timer = None;
        self.goal_time = None;
        self.nodes = 0;
        self.seldepth = 0;
        self.killers = [[None; 2]; MAX_PLY as usize];
        self.tt.reset();
    }

    pub fn data_search(&mut self, board: &Board, st: SearchType) -> (i32, Move) {
        let depth: i32;
        let mut goal_nodes: Option<u64> = None;
        match st {
            SearchType::Depth(d) => depth = d.min(MAX_PLY),
            SearchType::Nodes(n) => {
                depth = MAX_PLY;
                goal_nodes = Some(n);
            }
            _ => unreachable!(),
        };

        let mut best_move: Option<Move> = None;
        self.nnue.refresh(board);

        let mut score = 0;
        let mut pv = PVTable::new();

        for d in 1..=depth {
            self.seldepth = 0;
            score = self.aspiration_window(board, &mut pv, score, d);

            if let Some(nodes) = goal_nodes {
                if self.nodes >= nodes {
                    break;
                }
            }

            if self.stop && d > 1 {
                break;
            }

            best_move = pv.table[0];
        }

        (score, best_move.unwrap())
    }
}
