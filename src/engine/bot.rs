use std::sync::{Arc, atomic::AtomicBool};
use crate::chess::{board::ChessBoardState, chess_move::Move};
use super::board_eval::EvaluationFunction;


pub enum TimeControl {
    Infinite
}

pub trait ChessBot : EvaluationFunction + Default {
    fn search_best_move(&mut self, board_state: &mut ChessBoardState, tc: TimeControl, stop: Arc<AtomicBool>) -> Move;
    fn set_option(&mut self, name: String, value: String);
    fn get_options() -> &'static str;
}