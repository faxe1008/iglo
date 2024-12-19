use crate::{
    chess::{
        board::{ChessBoardState, PieceColor},
        chess_move::Move,
    },
    engine::{
        board_eval::{EvaluationFunction, PieceCountEvaluation, PieceSquareTableEvaluation},
        bot::ChessBot,
        time_control::TimeControl,
    },
};

#[derive(Default)]
pub struct OnePlyBot();
impl ChessBot for OnePlyBot {
    fn search_best_move(
        &mut self,
        board_state: &mut crate::chess::board::ChessBoardState,
        _tc: TimeControl,
        _stop: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Move {
        let mut moves = board_state.generate_legal_moves_for_current_player::<false>();

        moves.sort_by(|a, b| {
            if board_state.side == PieceColor::White {
                self.eval(&board_state.exec_move(*b)).cmp(&self.eval(&board_state.exec_move(*a)))
            } else {
                self.eval(&board_state.exec_move(*a)).cmp(&self.eval(&board_state.exec_move(*b)))
            }
        });

        let selected_move = moves[0];
        *board_state = board_state.exec_move(selected_move);
        selected_move
    }
    fn set_option(&mut self, _name: String, _value: String) {}
    fn get_options() -> &'static str {
        ""
    }
    fn append_to_history(&mut self, _board_state: &mut ChessBoardState) {}
    fn clear_history(&mut self) {}
}

impl EvaluationFunction for OnePlyBot {
    fn eval(&mut self, board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation.eval(board_state) + PieceSquareTableEvaluation.eval(board_state)
    }
}
