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
        _tc: TimeControl,
        _stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Move {
        let mut moves = board_state.generate_legal_moves_for_current_player();

        moves.sort_by(|a, b| {

            let board_a = board_state.exec_move(*a);
            let board_b = board_state.exec_move(*b);

            if board_state.side == PieceColor::White {
                Self::minimax(&board_b, 2).cmp(&Self::minimax(&board_a, 2))
            } else {
                Self::minimax(&board_a, 2).cmp(&Self::minimax(&board_b, 2))
            }
        });

        let best_move = moves[0];
        board_state.exec_move(best_move);
        best_move
    }
}

impl NPlyBot {
    #[inline(always)]
    fn minimax(board_state: &ChessBoardState, depth: u32) -> i32 {
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
            let mut best = i32::MIN;

            for mv in &moves {
                let new_board = board_state.exec_move(*mv);
                let eval = Self::minimax(&new_board, depth - 1);
                if eval > best {
                    best = eval;
                }
            }
            best
        } else {
            let mut best = i32::MAX;
            for mv in &moves {
                let new_board = board_state.exec_move(*mv);
                let eval = Self::minimax(&new_board, depth - 1);
                if eval < best {
                    best = eval;
                }
            }
            best
        }
    }
}

impl EvaluationFunction for NPlyBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state) + PieceSquareTableEvaluation::eval(board_state)
    }
}
