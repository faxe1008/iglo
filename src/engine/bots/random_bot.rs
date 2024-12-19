use rand::random;

use crate::{
    chess::{board::ChessBoardState, chess_move::Move},
    engine::{board_eval::EvaluationFunction, bot::ChessBot, time_control::TimeControl},
};

#[derive(Default)]
pub struct RandomBot();
impl ChessBot for RandomBot {
    fn search_best_move(
        &mut self,
        board_state: &mut crate::chess::board::ChessBoardState,
        _tc: TimeControl,
        _stop: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Move {
        let moves = board_state.generate_legal_moves_for_current_player::<false>();
        let index = random::<usize>() % moves.len();
        let selected_move = moves[index];
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

impl EvaluationFunction for RandomBot {
    fn eval(&mut self, _board_state: &crate::chess::board::ChessBoardState) -> i32 {
        0
    }
}
