use crate::{
    chess::{chess_move::Move, board::{PieceColor, ChessBoardState}},
    engine::{
        board_eval::{EvaluationFunction, PieceCountEvaluation, PieceSquareTableEvaluation},
        bot::{ChessBot, TimeControl},
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
        let mut moves = board_state.generate_legal_moves_for_current_player();
        

        moves.sort_by(|a,b|
            if board_state.side  == PieceColor::White {
                Self::eval(&board_state.exec_move(*b)).cmp(&Self::eval(&board_state.exec_move(*a)))
            } else {
                Self::eval(&board_state.exec_move(*a)).cmp(&Self::eval(&board_state.exec_move(*b)))
            }
        );

        let selected_move = moves[0];
        *board_state = board_state.exec_move(selected_move);
        selected_move
    }
    fn set_option(&mut self, _name: String, _value: String){}
    fn get_options() -> &'static str {
        ""
    }
    fn append_to_history(&mut self, _board_state: &mut ChessBoardState) {}
    fn clear_history(&mut self) {}
}

impl EvaluationFunction for OnePlyBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state) + PieceSquareTableEvaluation::eval(board_state)
    }
}
