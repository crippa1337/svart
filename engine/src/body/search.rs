use super::movegen::Picker;
use super::nnue::inference::NNUEState;
use super::position::{is_capture, is_quiet, play_move};
use super::{
    history::History,
    lmr::LMRTable,
    movegen,
    pv_table::PVTable,
    stat_vec::StaticVec,
    tt::{score_from_tt, AgeAndFlag, PackedMove, TTFlag, TT},
};

use crate::definitions::*;
use crate::uci::handler::SearchType;

use cozy_chess::{BitBoard, Board, Color, GameStatus, Move, Piece};
use once_cell::sync::Lazy;
use std::time::Instant;

static LMR: Lazy<LMRTable> = Lazy::new(LMRTable::new);
const RFP_MARGIN: i32 = 75;
const LMP_TABLE: [usize; 4] = [0, 5, 8, 18];
const FP_COEFFICIENT: i32 = 100;
const FP_MARGIN: i32 = 75;
const FP_DEPTH: i32 = 6;

pub struct StackEntry {
    pub eval: i32,
}

impl Default for StackEntry {
    fn default() -> Self {
        StackEntry { eval: NONE }
    }
}

pub struct SearchInfo {
    pub stop: bool,
    pub search_type: SearchType,
    pub timer: Option<Instant>,
    pub base_optimum: Option<u64>,
    pub max_time: Option<u64>,
    pub nodes: u64,
    pub node_table: [[u64; 64]; 64],
    pub seldepth: usize,
    pub game_history: Vec<u64>,
    pub killers: [[Option<Move>; 2]; MAX_PLY],
    pub history: History,
    pub stack: [StackEntry; MAX_PLY],
}

impl SearchInfo {
    pub fn new() -> Self {
        SearchInfo {
            stop: false,
            search_type: SearchType::Depth(0),
            timer: None,
            base_optimum: None,
            max_time: None,
            nodes: 0,
            node_table: [[0; 64]; 64],
            seldepth: 0,
            game_history: vec![],
            killers: [[None; 2]; MAX_PLY],
            history: History::new(),
            stack: std::array::from_fn(|_| StackEntry::default()),
        }
    }
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Search {
    pub nnue: Box<NNUEState>,
    pub tt: TT,
    pub info: SearchInfo,
}

impl Search {
    pub fn new(tt: TT) -> Self {
        Search { tt, nnue: NNUEState::from_board(&Board::default()), info: SearchInfo::new() }
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
        ply: usize,
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
        ply: usize,
    ) -> i32 {
        // Every 1024 nodes, check if it's time to stop
        if let (Some(timer), Some(max)) = (self.info.timer, self.info.max_time) {
            if self.info.nodes % 1024 == 0 && timer.elapsed().as_millis() as u64 >= max {
                self.info.stop = true;
            }
        }

        if self.info.stop && ply > 0 {
            return 0;
        }

        let stm = board.side_to_move();

        if ply >= MAX_PLY {
            return self.nnue.evaluate(stm);
        }

        let hash_key = board.hash();
        self.tt.prefetch(hash_key);
        self.info.seldepth = self.info.seldepth.max(ply);
        depth = depth.max(0);
        let mut old_pv = PVTable::new();
        pv.length = 0;

        match board.status() {
            GameStatus::Won => return ply as i32 - MATE,
            GameStatus::Drawn => return 8 - (self.info.nodes as i32 & 7),
            _ => (),
        }

        let root = ply == 0;

        if !root {
            if self.repetition(board, hash_key) {
                return 8 - (self.info.nodes as i32 & 7);
            }

            // Mate distance pruning
            let mate_alpha = alpha.max(ply as i32 - MATE);
            let mate_beta = beta.min(MATE - (ply as i32 + 1));
            if mate_alpha >= mate_beta {
                return mate_alpha;
            }
        }

        let in_check = !board.checkers().is_empty();

        // Escape condition
        if depth == 0 && !in_check {
            return self.qsearch::<PV>(board, alpha, beta, ply);
        }

        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key as u16;
        let tt_score = score_from_tt(tt_entry.score, ply) as i32;
        let mut tt_move: Option<Move> = None;

        if tt_hit {
            tt_move = Some(PackedMove::unpack(tt_entry.mv));

            if !PV && i32::from(tt_entry.depth) >= depth {
                debug_assert!(tt_score != NONE && tt_entry.age_flag != AgeAndFlag(0));
                let flag = tt_entry.age_flag.flag();

                if (flag == TTFlag::Exact)
                    || (flag == TTFlag::LowerBound && tt_score >= beta)
                    || (flag == TTFlag::UpperBound && tt_score <= alpha)
                {
                    return tt_score;
                }
            }
        }

        let eval = if tt_hit {
            // Use the TT score if available since eval is expensive
            // and any score from the TT is better than the static eval
            tt_score
        } else if in_check {
            // If we're in check, it's unstable to use the static eval
            -INFINITY
        } else {
            self.nnue.evaluate(stm)
        };

        // Internal Iterative Reduction (IIR)
        if depth >= 3 && !tt_hit {
            depth -= 1
        }

        // Improving
        // If the previous eval from our point of view is worse than what it currently is
        // then we are improving our position. This is used in some heuristics to improve pruning.
        self.info.stack[ply].eval = eval;
        let mut improving = false;
        if ply >= 2 {
            improving = !in_check && eval > self.info.stack[ply - 2].eval;
        }

        if !PV && !in_check {
            // Reverse Futility Pruning (RFP)
            // If static eval plus a margin can beat beta, then we can safely prune this node.
            // The margin is multiplied by depth to make it harder to prune at higher depths
            // as pruning there can be inaccurate as it prunes a large amount of potential nodes
            // and static eval isn't the most accurate.
            if depth < 9 && eval >= beta + RFP_MARGIN * depth / (i32::from(improving) + 1) {
                return eval;
            }

            // Null Move Pruning (NMP)
            // If we can give the opponent a free move and still cause a beta cutoff,
            // we can safely prune this node. This does not work in zugzwang positions
            // because then it is always better to give a free move, hence some checks for it are needed.
            if depth >= 3 && eval >= beta && !self.non_pawn_material(board, stm).is_empty() {
                let r = 3 + depth / 3 + 3.min((eval.saturating_sub(beta)) / 200);
                let new_b = board.null_move().unwrap();

                let score =
                    -self.zw_search(&new_b, &mut old_pv, -beta, -beta + 1, depth - r, ply + 1);

                if score >= beta {
                    return beta;
                }
            }
        }

        let old_alpha = alpha;
        let mut best_score = -INFINITY;
        let mut best_move: Option<Move> = None;
        let mut moves_played = 0;

        let move_list = movegen::all_moves(self, board, tt_move, ply);
        let mut quiet_moves = StaticVec::<Option<Move>, MAX_MOVES_POSITION>::new(None);
        let mut picker = Picker::new(move_list);

        let lmr_threshold = if PV { 5 } else { 3 };
        let mut quiets_checked = 0;
        let quiets_to_check = match depth {
            d @ 1..=3 => LMP_TABLE[d as usize],
            _ => MAX_MOVES_POSITION,
        };

        // Check extension
        depth += i32::from(in_check);

        while let Some(mv) = picker.pick_move() {
            let is_quiet = is_quiet(board, mv);
            let lmr_reduction = LMR.reduction(depth, moves_played.max(1));
            let lmr_depth = 0.max(depth - lmr_reduction);

            if is_quiet {
                quiets_checked += 1;

                if !PV && !in_check && best_score > TB_LOSS_IN_PLY {
                    // Late Move Pruning (LMP)
                    // If we have searched too many moves, we stop searching here
                    if quiets_checked >= quiets_to_check {
                        break;
                    }

                    // Futility Pruning (FP)
                    // If static eval plus a margin can't beat alpha, we stop searching here
                    let fp_margin = lmr_depth * FP_COEFFICIENT + FP_MARGIN;
                    if lmr_depth < FP_DEPTH && eval + fp_margin <= alpha {
                        break;
                    }
                }

                quiet_moves.push(Some(mv));
            }

            let mut new_b = board.clone();
            play_move(&mut new_b, &mut self.nnue, mv);

            moves_played += 1;
            self.info.game_history.push(board.hash());
            self.info.nodes += 1;
            let previous_nodes = self.info.nodes;
            let gives_check = !board.checkers().is_empty();

            let mut score: i32;
            if moves_played == 1 {
                score =
                    -self.pvsearch::<PV>(&new_b, &mut old_pv, -beta, -alpha, depth - 1, ply + 1);
            } else {
                // Late Move Reduction (LMR)
                // Assuming our move ordering is good, later moves will be worse
                // and can be searched with a reduced depth, if they beat alpha
                // we do a full re-search.
                let r = if depth >= 3 && moves_played > lmr_threshold {
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

                // Three-fold LMR
                // If the ZW beats alpha, then it might be
                // worth looking at this good position fully
                if score > alpha && r > 1 {
                    score = -self.zw_search(
                        &new_b,
                        &mut old_pv,
                        -alpha - 1,
                        -alpha,
                        depth - 1,
                        ply + 1,
                    );
                }

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

            self.info.game_history.pop();
            self.nnue.pop();

            if root {
                // Difference in node count
                self.info.node_table[mv.from as usize][mv.to as usize] +=
                    self.info.nodes - previous_nodes;
            }

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
                    self.info.killers[ply][1] = self.info.killers[ply][0];
                    self.info.killers[ply][0] = Some(mv);

                    // History Heuristic
                    self.info.history.update_table::<true>(board, mv, depth);
                    let qi = quiet_moves.as_slice();
                    let qi = &qi[..quiet_moves.len() - 1];
                    for qm in qi {
                        self.info.history.update_table::<false>(board, qm.unwrap(), depth);
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
            best_move = None;
            TTFlag::UpperBound
        };

        debug_assert!((-INFINITY..=INFINITY).contains(&best_score));

        if !self.info.stop {
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
        ply: usize,
    ) -> i32 {
        if let (Some(timer), Some(max)) = (self.info.timer, self.info.max_time) {
            if self.info.nodes % 1024 == 0 && timer.elapsed().as_millis() as u64 >= max {
                self.info.stop = true;
                return 0;
            }
        }

        if self.info.stop && ply > 0 {
            return 0;
        }

        let stm = board.side_to_move();

        if ply >= MAX_PLY {
            return self.nnue.evaluate(stm);
        }

        let hash_key = board.hash();
        self.tt.prefetch(hash_key);
        self.info.seldepth = self.info.seldepth.max(ply);

        let stand_pat = self.nnue.evaluate(stm);
        alpha = alpha.max(stand_pat);
        if stand_pat >= beta {
            return stand_pat;
        }

        let tt_entry = self.tt.probe(hash_key);
        let tt_hit = tt_entry.key == hash_key as u16;
        let mut tt_move: Option<Move> = None;

        if tt_hit && !PV && tt_entry.age_flag != AgeAndFlag(0) {
            let tt_score = score_from_tt(tt_entry.score, ply) as i32;
            debug_assert!(tt_score != NONE);

            tt_move = Some(PackedMove::unpack(tt_entry.mv));
            let flag = tt_entry.age_flag.flag();

            if (flag == TTFlag::Exact)
                || (flag == TTFlag::LowerBound && tt_score >= beta)
                || (flag == TTFlag::UpperBound && tt_score <= alpha)
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

            self.info.nodes += 1;

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

        if !self.info.stop {
            self.tt.store(hash_key, best_move, best_score as i16, 0, flag, ply);
        }

        best_score
    }

    pub fn iterative_deepening(&mut self, board: &Board, st: SearchType, pretty: bool) {
        let depth: usize;
        let mut opt_time: Option<u64> = None;
        let mut goal_nodes: Option<u64> = None;

        match st {
            SearchType::Time(opt, max) => {
                depth = MAX_PLY;
                self.info.timer = Some(Instant::now());
                self.info.max_time = Some(max);
                self.info.base_optimum = Some(opt);
                opt_time = Some(opt);
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
            self.info.seldepth = 0;
            score = self.aspiration_window(board, &mut pv, score, d as i32);

            // Max time is up
            if self.info.stop && d > 1 {
                break;
            }

            best_move = pv.table[0];

            if pretty {
                crate::uci::handler::pretty_print(
                    d,
                    self.info.seldepth,
                    score,
                    self.info.nodes,
                    info_timer.elapsed().as_millis(),
                    pv.pv_string(),
                );
            } else {
                println!(
                    "info depth {} seldepth {} score {} nodes {} time {} pv{}",
                    d,
                    self.info.seldepth,
                    format_score(score),
                    self.info.nodes,
                    info_timer.elapsed().as_millis(),
                    pv.pv_string()
                );
            }

            // Nodes search type
            if let Some(nodes) = goal_nodes {
                if self.info.nodes >= nodes {
                    break;
                }
            }

            // Optimal time check
            if let Some(mut opt) = opt_time {
                // Time bound adjustments
                #[rustfmt::skip]
                let best_move_fraction = 
                    self.info.node_table
                    [best_move.unwrap().from as usize]
                    [best_move.unwrap().to as usize] as f64
                    / self.info.nodes as f64;

                let time_factor = (1.5 - best_move_fraction) * 1.2;
                opt = (self.info.base_optimum.unwrap() as f64 * time_factor) as u64;
                

                if info_timer.elapsed().as_millis() as u64 >= opt {
                    break;
                }
            }
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

            if self.info.stop {
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

    fn repetition(&self, board: &Board, hash: u64) -> bool {
        self.info
            .game_history
            .iter()
            .rev()
            .take(board.halfmove_clock() as usize)
            // Skip the current position
            .skip(1)
            // Two fold
            .any(|&key| key == hash)
    }

    fn non_pawn_material(&self, board: &Board, color: Color) -> BitBoard {
        (board.pieces(Piece::Knight)
            | board.pieces(Piece::Bishop)
            | board.pieces(Piece::Rook)
            | board.pieces(Piece::Queen))
            & board.colors(color)
    }

    pub fn go_reset(&mut self) {
        self.info.stop = false;
        self.info.search_type = SearchType::Depth(0);
        self.info.timer = None;
        self.info.max_time = None;
        self.info.base_optimum = None;
        self.info.nodes = 0;
        self.info.node_table = [[0; 64]; 64];
        self.info.seldepth = 0;
        self.info.killers = [[None; 2]; MAX_PLY];
        self.info.history.age_table();
        self.tt.age();
    }

    pub fn game_reset(&mut self) {
        self.tt.reset();
        self.info = SearchInfo::new();
    }

    pub fn data_search(&mut self, board: &Board, st: SearchType) -> (i32, Move) {
        let depth: usize;
        let mut goal_nodes: Option<u64> = None;
        match st {
            SearchType::Depth(d) => depth = (d).min(MAX_PLY),
            SearchType::Nodes(n) => {
                depth = MAX_PLY;
                goal_nodes = Some(n);
            }
            _ => unreachable!(),
        };

        let mut best_move: Option<Move> = None;

        let mut score = 0;
        let mut pv = PVTable::new();

        for d in 1..=depth {
            self.info.seldepth = 0;
            score = self.aspiration_window(board, &mut pv, score, d as i32);

            if self.info.stop && d > 1 {
                break;
            }

            best_move = pv.table[0];

            if let Some(nodes) = goal_nodes {
                if self.info.nodes >= nodes {
                    break;
                }
            }
        }

        (score, best_move.unwrap())
    }
}

pub fn format_score(score: i32) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repetitions() {
        const FENS: [&str; 3] = [
            "5k2/4q1p1/3P1pQb/1p1B4/pP5p/P1PR4/5PP1/1K6 b - - 0 38",
            "6k1/6p1/8/6KQ/1r6/q2b4/8/8 w - - 0 32",
            "5rk1/1rP3pp/p4n2/3Pp3/1P2Pq2/2Q4P/P5P1/R3R1K1 b - - 0 32",
        ];

        let tt = TT::new(16);
        let mut search = Search::new(tt);

        for fen in FENS.iter() {
            let board = Board::from_fen(fen, false).unwrap();
            search.nnue.refresh(&board);

            let (score, _) = search.data_search(&board, SearchType::Depth(20));

            assert!((-10..=10).contains(&score), "{score} {fen}");

            search.game_reset();
        }
    }
}
