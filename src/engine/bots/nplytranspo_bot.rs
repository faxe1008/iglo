use std::{
    cmp::{max, min, Ordering},
    sync::{atomic::AtomicBool, Arc},
};

use rand::random;

use crate::{
    chess::{
        board::{ChessBoardState, PieceColor},
        chess_move::{Move, MoveType},
        zobrist_hash::ZHash,
    },
    engine::{
        board_eval::{
            EvaluationFunction, PassedPawnEvaluation, PieceCountEvaluation,
            PieceSquareTableEvaluation,
        },
        bot::ChessBot,
        move_ordering::order_moves,
        time_control::TimeControl,
        transposition_table::{NodeType, TranspositionEntry, TranspositionTable},
    },
};

pub const TABLE_SIZE: usize = 64 * 1024 * 1024;
pub const TABLE_ENTRY_SIZE: usize = std::mem::size_of::<TranspositionEntry>();
pub const TABLE_ENTRY_COUNT: usize = TABLE_SIZE / TABLE_ENTRY_SIZE;

const INFINITY: i32 = 50000;
const MAX_EXTENSIONS: u16 = 3;

#[derive(Default)]
pub struct SearchInfo {
    nodes_searched: usize,
    num_extensions: u16,
    pub history: Vec<ZHash>,
}

pub struct Searcher<const T: usize> {
    transposition_table: Box<TranspositionTable<T>>,
    pub info: SearchInfo,
    eval_fn: fn(&ChessBoardState) -> i32,
    pub stop: Arc<AtomicBool>,
}

impl<const T: usize> Searcher<T> {
    pub fn new(
        eval_fn: fn(&ChessBoardState) -> i32,
    ) -> Self {
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
        }
    }
}

pub struct NPlyTranspoBot {
    searcher: Searcher<TABLE_ENTRY_COUNT>
}

impl Default for NPlyTranspoBot {
    fn default() -> Self {
        Self { searcher: Searcher::new(Self::eval) }
    }
}


impl ChessBot for NPlyTranspoBot {
    fn search_best_move(
        &mut self,
        board_state: &mut ChessBoardState,
        tc: TimeControl,
        stop: &Arc<AtomicBool>,
    ) -> Move {
        let depth = match tc {
            TimeControl::FixedDepth(d) => d,
            TimeControl::Infinite => 6,
        } as u16;

        // Print current board eval
        let cur_board_eval = Self::eval(board_state);
        println!("info score cp {}", cur_board_eval as f32 / 100.0);

        let mut moves = board_state.generate_legal_moves_for_current_player();

        // Sort moves by expected value
        order_moves(&mut moves, board_state);

        if board_state.full_moves == 0 {
            return moves[random::<usize>() % moves.len()];
        }

        // Iterative deepening
        for d in 1..=depth {
            self.minimax_root(d, board_state, &mut moves);
        }

        let best_move = moves[0];
        board_state.exec_move(best_move);
        self.searcher.info.history.push(board_state.zhash);
        best_move
    }

    fn set_option(&mut self, _name: String, _value: String) {}
    fn get_options() -> &'static str {
        ""
    }
    fn append_to_history(&mut self, board_state: &mut ChessBoardState) {
        self.searcher.info.history.push(board_state.zhash);
    }
    fn clear_history(&mut self) {
        self.searcher.info.history.clear();
    }
}

impl NPlyTranspoBot {
    pub fn minimax_root(
        &mut self,
        depth: u16,
        board_state: &ChessBoardState,
        moves: &mut Vec<Move>,
    ) {
        let mut ratings = if board_state.side == PieceColor::White {
            vec![i32::MIN; moves.len()]
        } else {
            vec![i32::MAX; moves.len()]
        };

        // Evaluate moves
        for (mv_index, mv) in moves.iter().enumerate() {
            let board_new = board_state.exec_move(*mv);
            ratings[mv_index] =
                self.minimax(&board_new, depth, 0, i32::MIN, i32::MAX, 0);
        }

        if self.searcher.stop.load(std::sync::atomic::Ordering::SeqCst) {
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

    fn is_draw(&self, board_state: &ChessBoardState, depth: u16) -> bool {
        board_state.half_moves >= 100 || self.is_repetition(board_state, depth)
    }

    fn is_repetition(&self, board_state: &ChessBoardState, depth: u16) -> bool {
        let rollback = 1 + (depth as usize).min(board_state.half_moves as usize);

        // Rollback == 1 implies we only look at the opponent's position.
        if rollback == 1 {
            return false;
        }

        self.searcher.info.history
            .iter()
            .rev() // step through history in reverse
            .take(rollback) // only check elements within rollback
            .skip(1) // first element is opponent, skip.
            .step_by(2) // don't check opponent moves
            .any(|b| *b == board_state.zhash) // stop at first repetition
    }

    fn minimax(
        &mut self,
        board_state: &ChessBoardState,
        mut ply_remaining: u16,
        ply_from_root: u16,
        mut alpha: i32,
        mut beta: i32,
        mut extensions: u16,
    ) -> i32 {
        if self.searcher.stop.load(std::sync::atomic::Ordering::SeqCst) {
            return 0;
        }

        if let Some(eval) =
            self.searcher.transposition_table
                .lookup(board_state.zhash, ply_remaining, alpha, beta)
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
            return Self::eval(board_state);
        }

        let mut moves = board_state.generate_legal_moves_for_current_player();

        // No moves, either draw or checkmate
        if moves.len() == 0 {
            let score = match (board_state.side, is_in_check) {
                (PieceColor::White, true) => -INFINITY * ply_remaining as i32,
                (PieceColor::White, false) => 0,
                (PieceColor::Black, true) => INFINITY * ply_remaining as i32,
                (PieceColor::Black, false) => 0,
            };
            self.searcher.transposition_table
                .add_entry(board_state, score, ply_remaining, NodeType::Exact);
            return score;
        }

        // Check for drawing moves
        if self.is_draw(board_state, ply_remaining) {
            self.searcher.transposition_table
                .add_entry(board_state, 0, ply_remaining, NodeType::Exact);
            return 0;
        }

        // Sort moves by expected value
        order_moves(&mut moves, board_state);

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
                    self.searcher.transposition_table.add_entry(
                        board_state,
                        beta,
                        ply_remaining,
                        NodeType::LowerBound,
                    );
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
                    self.searcher.transposition_table.add_entry(
                        board_state,
                        alpha,
                        ply_remaining,
                        NodeType::UpperBound,
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

impl EvaluationFunction for NPlyTranspoBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state)
            + PieceSquareTableEvaluation::eval(board_state)
            + PassedPawnEvaluation::eval(board_state)
    }
}

#[cfg(test)]
mod nplytranspo_tests {
    use crate::chess::chess_move::MoveType;

    #[test]
    fn test_move_ordering() {
        assert!(MoveType::Capture > MoveType::Silent);

        let mut moves = vec![MoveType::Silent, MoveType::Capture];
        moves.sort_by(|a, b| b.cmp(a));
        assert!(moves[0] == MoveType::Capture);
    }
}
