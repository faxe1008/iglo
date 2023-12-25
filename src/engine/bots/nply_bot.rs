use std::cmp::{max, min};

use crate::{
    chess::{
        board::{self, ChessBoardState, PieceColor},
        chess_move::Move,
    },
    engine::{
        board_eval::{EvaluationFunction, PieceCountEvaluation, PieceSquareTableEvaluation},
        bot::{ChessBot, TimeControl},
    },
};
#[derive(Default)]
pub struct NPlyBot();
impl ChessBot for NPlyBot {
    fn search_best_move(
        &mut self,
        board_state: &mut crate::chess::board::ChessBoardState,
        tc: TimeControl,
        _stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Move {
        let mut moves = board_state.generate_legal_moves_for_current_player();
        let depth = match tc {
            TimeControl::FixedDepth(d) => d,
            TimeControl::Infinite => 4,
        };

        moves.sort_by(|a, b| {
            let board_a = board_state.exec_move(*a);
            let board_b = board_state.exec_move(*b);

            if board_state.side == PieceColor::White {
                Self::minimax_alpha_beta(&board_b, depth, i32::MIN, i32::MAX).cmp(
                    &Self::minimax_alpha_beta(&board_a, depth, i32::MIN, i32::MAX),
                )
            } else {
                Self::minimax_alpha_beta(&board_a, depth, i32::MIN, i32::MAX).cmp(
                    &Self::minimax_alpha_beta(&board_b, depth, i32::MIN, i32::MAX),
                )
            }
        });

        let best_move = moves[0];
        board_state.exec_move(best_move);
        best_move
    }
    fn set_option(&mut self, _name: String, _value: String) {}
    fn get_options() -> &'static str {
        ""
    }
}

impl NPlyBot {
    fn minimax_alpha_beta(
        board_state: &ChessBoardState,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
    ) -> i32 {
        if depth == 0 {
            return Self::eval(board_state);
        }

        let moves = board_state.generate_legal_moves_for_current_player();
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
                    Self::minimax_alpha_beta(&new_board, depth - 1, alpha, beta),
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
                    Self::minimax_alpha_beta(&new_board, depth - 1, alpha, beta),
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

impl EvaluationFunction for NPlyBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state) + PieceSquareTableEvaluation::eval(board_state)
    }
}
