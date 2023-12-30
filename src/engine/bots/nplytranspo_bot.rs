use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    chess::{board::ChessBoardState, chess_move::Move},
    engine::{
        board_eval::{
            EvaluationFunction, PassedPawnEvaluation, PieceCountEvaluation,
            PieceSquareTableEvaluation,
        },
        bot::ChessBot,
        search::Searcher,
        time_control::TimeControl,
        transposition_table::TranspositionEntry,
    },
};

pub const TABLE_SIZE: usize = 64 * 1024 * 1024;
pub const TABLE_ENTRY_SIZE: usize = std::mem::size_of::<TranspositionEntry>();
pub const TABLE_ENTRY_COUNT: usize = TABLE_SIZE / TABLE_ENTRY_SIZE;

pub struct NPlyTranspoBot {
    searcher: Searcher<TABLE_ENTRY_COUNT>,
}

impl Default for NPlyTranspoBot {
    fn default() -> Self {
        Self {
            searcher: Searcher::new(Self::eval),
        }
    }
}

impl ChessBot for NPlyTranspoBot {
    fn search_best_move(
        &mut self,
        board_state: &mut ChessBoardState,
        tc: TimeControl,
        stop: &Arc<AtomicBool>,
    ) -> Move {
        let cur_board_eval = Self::eval(board_state);
        println!("info score cp {}", cur_board_eval as f32 / 100.0);

        self.searcher.search(board_state, tc, stop)
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

impl EvaluationFunction for NPlyTranspoBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state)
            + PieceSquareTableEvaluation::eval(board_state)
            + PassedPawnEvaluation::eval(board_state)
    }
}
