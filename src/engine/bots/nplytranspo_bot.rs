use std::cmp::{max, min, Ordering};

use crate::{
    chess::{
        board::{ChessBoardState, PieceColor},
        chess_move::{Move, MoveType},
    },
    engine::{
        board_eval::{EvaluationFunction, PieceCountEvaluation, PieceSquareTableEvaluation},
        bot::{ChessBot, TimeControl},
        transposition_table::{TranspositionEntry, TranspositionTable},
    },
};

pub const TABLE_SIZE: usize = 64 * 1024 * 1024;
pub const TABLE_ENTRY_SIZE: usize = std::mem::size_of::<TranspositionEntry>();
pub const TABLE_ENTRY_COUNT: usize = TABLE_SIZE / TABLE_ENTRY_SIZE;

pub struct NPlyTranspoBot {
    pub transposition_table: Box<TranspositionTable<TABLE_ENTRY_COUNT>>,
}

impl Default for NPlyTranspoBot {
    fn default() -> Self {
        let transposition_table = unsafe {
            let layout = std::alloc::Layout::new::<TranspositionTable<TABLE_ENTRY_COUNT>>();
            let ptr =
                std::alloc::alloc_zeroed(layout) as *mut TranspositionTable<TABLE_ENTRY_COUNT>;
            Box::from_raw(ptr)
        };
        Self {
            transposition_table: transposition_table,
        }
    }
}

impl Ord for MoveType {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            return Ordering::Equal;
        }

        if self.is_capture() && !other.is_capture() {
            return Ordering::Greater;
        } else if !self.is_capture() && other.is_capture() {
            return Ordering::Less;
        }

        match (self, other) {
            (MoveType::QueenPromotion, MoveType::KnightPromotion) => return Ordering::Greater,
            (MoveType::QueenPromotion, MoveType::BishopPromotion) => return Ordering::Greater,
            (MoveType::QueenPromotion, MoveType::RookPromotion) => return Ordering::Greater,
            (MoveType::QueenCapPromotion, MoveType::KnightCapPromotion) => {
                return Ordering::Greater
            }
            (MoveType::QueenCapPromotion, MoveType::BishopCapPromotion) => {
                return Ordering::Greater
            }
            (MoveType::QueenCapPromotion, MoveType::RookCapPromotion) => return Ordering::Greater,
            _ => {}
        };

        Ordering::Equal
    }
}

impl ChessBot for NPlyTranspoBot {
    fn search_best_move(
        &mut self,
        board_state: &mut ChessBoardState,
        tc: TimeControl,
        _stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Move {
        let depth = match tc {
            TimeControl::FixedDepth(d) => d,
            TimeControl::Infinite => 6,
        };

        // Purge transpo table if needed
        if board_state.full_moves % 20 == 0 {
            self.transposition_table.clear();
        }

        // Print current board eval
        let cur_board_eval = self.get_eval(board_state);
        println!("info score cp {}", cur_board_eval as f32 / 100.0);

        let mut moves = board_state.generate_legal_moves_for_current_player();
        let zipped = self.eval_moves(depth, board_state, &mut moves);

        let best_move = zipped[0].0;
        board_state.exec_move(best_move);
        best_move
    }

    fn set_option(&mut self, _name: String, _value: String) {}
    fn get_options() -> &'static str {
        ""
    }
}

impl NPlyTranspoBot {
    pub fn eval_moves(
        &mut self,
        depth: u32,
        board_state: &ChessBoardState,
        moves: &mut Vec<Move>,
    ) -> Vec<(Move, i32)> {
        let mut ratings = if board_state.side == PieceColor::White {
            vec![i32::MIN; moves.len()]
        } else {
            vec![i32::MAX; moves.len()]
        };

        // Sort moves by expected value
        moves.sort_by(|a, b| b.get_type().cmp(&a.get_type()));

        // Evaluate moves
        for (mv_index, mv) in moves.iter().enumerate() {
            let board_new = board_state.exec_move(*mv);
            ratings[mv_index] = self.minimax_alpha_beta(&board_new, depth, i32::MIN, i32::MAX);
        }

        let mut zipped: Vec<_> = moves.drain(..).zip(ratings.drain(..)).collect();

        if board_state.side == PieceColor::White {
            zipped.sort_by(|(_, a_rt), (_, b_rt)| b_rt.cmp(a_rt));
        } else {
            zipped.sort_by(|(_, a_rt), (_, b_rt)| a_rt.cmp(b_rt));
        }
        zipped
    }

    fn get_eval(&mut self, board_state: &ChessBoardState) -> i32 {
        if let Some(entry) = self.transposition_table.lookup(board_state.zhash) {
            entry.eval
        } else {
            let ev_value = Self::eval(board_state);
            self.transposition_table
                .add_entry(board_state.zhash, ev_value);
            ev_value
        }
    }

    fn minimax_alpha_beta(
        &mut self,
        board_state: &ChessBoardState,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
    ) -> i32 {
        if depth == 0 {
            return self.get_eval(board_state);
        }

        let mut moves = board_state.generate_legal_moves_for_current_player();

        // Sort moves by expected value
        moves.sort_by(|a, b| b.get_type().cmp(&a.get_type()));

        if moves.len() == 0 {
            if board_state.side == PieceColor::White {
                return i32::MIN;
            } else {
                return i32::MAX;
            }
        }

        if board_state.side == PieceColor::White {
            let mut value = i32::MIN;

            for mv in &moves {
                let new_board = board_state.exec_move(*mv);

                value = max(
                    value,
                    self.minimax_alpha_beta(&new_board, depth - 1, alpha, beta),
                );
                alpha = max(alpha, value);

                if value >= beta {
                    break;
                }
            }
            value
        } else {
            let mut value = i32::MAX;
            for mv in &moves {
                let new_board = board_state.exec_move(*mv);
                value = min(
                    value,
                    self.minimax_alpha_beta(&new_board, depth - 1, alpha, beta),
                );
                beta = min(beta, value);
                if value <= alpha {
                    break;
                }
            }
            value
        }
    }
}

impl EvaluationFunction for NPlyTranspoBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state) + PieceSquareTableEvaluation::eval(board_state)
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
