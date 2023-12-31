use super::{
    move_ordering::order_moves,
    time_control::TimeControl,
    transposition_table::{NodeType, TranspositionTable},
};
use crate::chess::{
    board::{ChessBoardState, PieceColor},
    chess_move::Move,
    zobrist_hash::ZHash,
};
use std::{
    cmp::{max, min},
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

const INFINITY: i32 = 50000;
const MAX_EXTENSIONS: usize = 3;

pub const MAX_PLY: u16 = 64;
pub const MAX_KILLER_MOVES: usize = 2;
type KillerMoves = [[Move; MAX_PLY as usize]; MAX_KILLER_MOVES];

pub struct SearchInfo {
    nodes_searched: usize,
    sel_depth: usize,
    pub history: Vec<ZHash>,
    pub killer_moves: KillerMoves,
    search_start_time: Instant,
    self_color: PieceColor,
}

impl Default for SearchInfo {
    fn default() -> Self {
        Self {
            nodes_searched: 0,
            sel_depth: 0,
            search_start_time: Instant::now(),
            history: Default::default(),
            killer_moves: [[Move::NULL_MOVE; MAX_PLY as usize]; MAX_KILLER_MOVES],
            self_color: PieceColor::White,
        }
    }
}

impl SearchInfo {
    fn reset(&mut self) {
        self.nodes_searched = 0;
        self.sel_depth = 0;
        self.search_start_time = Instant::now();
        self.killer_moves = [[Move::NULL_MOVE; MAX_PLY as usize]; MAX_KILLER_MOVES];
    }

    fn store_killer_move(&mut self, current_move: Move, ply_from_root: u16) {
        let ply = ply_from_root as usize;
        let first_killer = self.killer_moves[0][ply];

        // First killer must not be the same as the move being stored.
        if first_killer != current_move {
            // Shift all the moves one index upward...
            for i in (1..MAX_KILLER_MOVES).rev() {
                let n = i as usize;
                let previous = self.killer_moves[n - 1][ply];
                self.killer_moves[n][ply] = previous;
            }

            // and add the new killer move in the first spot.
            self.killer_moves[0][ply] = current_move;
        }
    }
}

pub struct Searcher<const T: usize> {
    transposition_table: Box<TranspositionTable<T>>,
    pub info: SearchInfo,
    eval_fn: fn(&ChessBoardState) -> i32,
    pub stop: Arc<AtomicBool>,
    time_control: TimeControl,
}

impl<const T: usize> Searcher<T> {
    pub fn new(eval_fn: fn(&ChessBoardState) -> i32) -> Self {
        let transposition_table = unsafe {
            let layout = std::alloc::Layout::new::<TranspositionTable<T>>();
            let ptr = std::alloc::alloc_zeroed(layout) as *mut TranspositionTable<T>;
            Box::from_raw(ptr)
        };
        Self {
            transposition_table,
            info: SearchInfo::default(),
            eval_fn,
            stop: Arc::new(false.into()),
            time_control: TimeControl::FixedDepth(5),
        }
    }

    fn should_stop(&mut self) -> bool {
        if self.stop.load(std::sync::atomic::Ordering::SeqCst) {
            return true;
        }

        if self.info.nodes_searched % 2048 != 0 {
            return false;
        }

        let should_stop = match &self.time_control {
            TimeControl::Infinite => false,
            TimeControl::FixedDepth(_) => false, // Handeled by the iterative deepening
            TimeControl::FixedNodes(n) => self.info.nodes_searched >= *n as usize,
            TimeControl::FixedTime(t) => {
                let duration = Instant::now().duration_since(self.info.search_start_time);
                duration.as_millis() >= *t as u128
            }
            TimeControl::Variable(cc) => {
                let duration = Instant::now().duration_since(self.info.search_start_time);

                let (time, inc) = if self.info.self_color == PieceColor::White {
                    (cc.white_time.unwrap(), cc.white_inc.unwrap_or(0))
                } else {
                    (cc.black_time.unwrap(), cc.black_inc.unwrap_or(0))
                };

                const OVERHEAD: u64 = 50;
                let time = time - OVERHEAD.min(time);
                let inc = if time < OVERHEAD { 0 } else { inc };

                let duration_for_move = if let Some(moves) = cc.movestogo {
                    let scale = 0.7 / (moves.min(50) as f64);
                    let eight = 0.8 * time as f64;

                    let opt_time = (scale * time as f64).min(eight);
                    opt_time
                } else {
                    let total = ((time / 20) + (inc * 3 / 4)) as f64;

                    0.6 * total
                };

                duration.as_millis() >= duration_for_move as u128
            }
        };

        if should_stop {
            self.stop
                .store(should_stop, std::sync::atomic::Ordering::SeqCst);
        }

        should_stop
    }

    fn depth_from_time_control(&mut self, time_control: &TimeControl) -> u16 {
        match time_control {
            TimeControl::Infinite => MAX_PLY,
            TimeControl::FixedDepth(d) => *d as u16,
            TimeControl::FixedNodes(_) => MAX_PLY,
            TimeControl::FixedTime(_) => MAX_PLY,
            TimeControl::Variable(_) => MAX_PLY,
        }
    }

    pub fn search(
        &mut self,
        board_state: &mut ChessBoardState,
        time_control: TimeControl,
        stop: &Arc<AtomicBool>,
    ) -> Move {
        let mut moves = board_state.generate_legal_moves_for_current_player();
        // Sort moves by expected value
        order_moves(&mut moves, board_state, &self.info, 0);

        let search_depth = self.depth_from_time_control(&time_control);
        self.stop = stop.clone();
        self.stop.store(false, std::sync::atomic::Ordering::SeqCst);
        self.info.reset();
        self.time_control = dbg!(time_control);
        self.info.self_color = board_state.side;

        // Iterative deepening
        for d in 1..=search_depth {
            self.minimax_root(board_state, &mut moves, d);
        }

        let search_duration = Instant::now().duration_since(self.info.search_start_time);
        let nps = (1000 * self.info.nodes_searched as u128) / (search_duration.as_millis() + 1);

        println!(
            "info time {} nodes {} nps {} hashfull {}",
            search_duration.as_millis(),
            self.info.nodes_searched,
            nps,
            self.transposition_table.hashfull()
        );

        let best_move = moves[0];
        board_state.exec_move(best_move);
        self.info.history.push(board_state.zhash);
        best_move
    }

    fn is_draw(&self, board_state: &ChessBoardState, depth: u16) -> bool {
        board_state.half_moves >= 100 || self.is_repetition(board_state, depth)
    }

    fn is_repetition(&self, board_state: &ChessBoardState, depth: u16) -> bool {
        let rollback = 1 + (depth as usize).min(board_state.half_moves as usize);

        // Rollback == 1 implies we only look at the opponent's position.
        if rollback == 1 {
            return false;
        }

        self.info
            .history
            .iter()
            .rev() // step through history in reverse
            .take(rollback) // only check elements within rollback
            .skip(1) // first element is opponent, skip.
            .step_by(2) // don't check opponent moves
            .any(|b| *b == board_state.zhash) // stop at first repetition
    }

    fn minimax_root(
        &mut self,
        board_state: &mut ChessBoardState,
        moves: &mut Vec<Move>,
        depth: u16,
    ) {
        let mut ratings = if board_state.side == PieceColor::White {
            vec![i32::MIN; moves.len()]
        } else {
            vec![i32::MAX; moves.len()]
        };

        // Evaluate moves
        for (mv_index, mv) in moves.iter().enumerate() {
            let board_new = board_state.exec_move(*mv);
            ratings[mv_index] = self.minimax(&board_new, depth, 0, i32::MIN, i32::MAX, 0);
        }

        if self.should_stop() {
            return;
        }

        // Sort moves by their rating
        let mut zipped: Vec<_> = moves.drain(..).zip(ratings.drain(..)).collect();
        if board_state.side == PieceColor::White {
            zipped.sort_by(|(_, a_rt), (_, b_rt)| b_rt.cmp(a_rt));
        } else {
            zipped.sort_by(|(_, a_rt), (_, b_rt)| a_rt.cmp(b_rt));
        }

        // Push sorted moves back to caller
        *moves = zipped.drain(..).map(|(mv, _)| mv).collect();
    }

    fn minimax(
        &mut self,
        board_state: &ChessBoardState,
        mut ply_remaining: u16,
        ply_from_root: u16,
        mut alpha: i32,
        mut beta: i32,
        mut extensions: usize,
    ) -> i32 {
        if self.should_stop() {
            return 0;
        }

        if let Some(eval) =
            self.transposition_table
                .lookup(board_state.zhash, ply_remaining, &mut alpha, &mut beta)
        {
            return eval;
        }

        // Extend the search
        let is_in_check = board_state.is_in_check();
        if is_in_check && extensions < MAX_EXTENSIONS {
            ply_remaining += 1;
            extensions += 1;
        }

        if ply_remaining == 0 {
            return (self.eval_fn)(board_state);
        }

        self.info.nodes_searched += 1;
        self.info.sel_depth = self.info.sel_depth.max(ply_from_root as usize);
        let mut moves = board_state.generate_legal_moves_for_current_player();

        // No moves, either draw or checkmate
        if moves.len() == 0 {
            let score = match (board_state.side, is_in_check) {
                (PieceColor::White, true) => -INFINITY * ply_remaining as i32,
                (PieceColor::White, false) => 0,
                (PieceColor::Black, true) => INFINITY * ply_remaining as i32,
                (PieceColor::Black, false) => 0,
            };
            self.transposition_table.add_entry(
                board_state,
                score,
                ply_remaining,
                NodeType::Exact,
                &self.stop,
            );

            return score;
        }

        // Check for drawing moves
        if self.is_draw(board_state, ply_remaining) {
            self.transposition_table.add_entry(
                board_state,
                0,
                ply_remaining,
                NodeType::Exact,
                &self.stop,
            );

            return 0;
        }

        // Sort moves by expected value
        order_moves(&mut moves, board_state, &self.info, ply_from_root);

        let value = if board_state.side == PieceColor::White {
            let mut value = i32::MIN;

            for mv in &moves {
                let new_board = board_state.exec_move(*mv);

                value = max(
                    value,
                    self.minimax(
                        &new_board,
                        ply_remaining - 1,
                        ply_from_root + 1,
                        alpha,
                        beta,
                        extensions,
                    ),
                );
                if value >= beta {
                    self.transposition_table.add_entry(
                        board_state,
                        alpha,
                        ply_remaining,
                        NodeType::LowerBound,
                        &self.stop,
                    );

                    if !mv.is_capture() {
                        self.info.store_killer_move(*mv, ply_from_root);
                    }
                    break;
                }
                alpha = max(alpha, value);
            }
            value
        } else {
            let mut value = i32::MAX;
            for mv in &moves {
                let new_board = board_state.exec_move(*mv);
                value = min(
                    value,
                    self.minimax(
                        &new_board,
                        ply_remaining - 1,
                        ply_from_root + 1,
                        alpha,
                        beta,
                        extensions,
                    ),
                );
                if value < alpha {
                    self.transposition_table.add_entry(
                        board_state,
                        beta,
                        ply_remaining,
                        NodeType::UpperBound,
                        &self.stop,
                    );

                    break;
                }
                beta = min(beta, value);
            }
            value
        };

        value
    }
}