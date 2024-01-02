use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    chess::{board::ChessBoardState, chess_move::Move},
    engine::{
        board_eval::{
            EvaluationFunction, PassedPawnEvaluation, PieceCountEvaluation,
            PieceSquareTableEvaluation, BishopPairEvaluation, KingPawnShieldEvaluation,
        },
        bot::ChessBot,
        opening::opening_book::OpeningBook,
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
    opening_book: Option<OpeningBook>,
    use_openening_book: bool,
}

const OPENING_BOOK_DATA: &[u8; 225740] = include_bytes!("../opening/opening_book.bin");

impl Default for NPlyTranspoBot {
    fn default() -> Self {
        let opening_book = bincode::deserialize::<OpeningBook>(OPENING_BOOK_DATA).ok();

        Self {
            searcher: Searcher::new(Self::eval),
            opening_book: opening_book,
            use_openening_book: true,
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

        if self.use_openening_book && board_state.full_moves < 6 && self.opening_book.is_some() {
            if let Some(mv) = self
                .opening_book
                .as_ref()
                .unwrap()
                .lookup(board_state.zhash, true)
            {
                println!("info string openingbook hit {:?}", &mv);
                return mv;
            }
        }

        self.searcher.search(board_state, tc, stop)
    }

    fn set_option(&mut self, name: String, value: String) {
        match &name as &str {
            "OpeningBook" => self.use_openening_book = value == "true",
            _ => {}
        }
    }
    fn get_options() -> &'static str {
        "option name OpeningBook type check default true"
    }
    fn append_to_history(&mut self, board_state: &mut ChessBoardState) {
        self.searcher.info.history.push(board_state.zhash);
    }
    fn clear_history(&mut self) {
        self.searcher.info.history.clear();
        // TODO: Hack factor this better
        self.searcher.incr_hash_table_age();
    }
}

impl EvaluationFunction for NPlyTranspoBot {
    fn eval(board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation::eval(board_state)
            + PieceSquareTableEvaluation::eval(board_state)
            + PassedPawnEvaluation::eval(board_state)
            + BishopPairEvaluation::eval(board_state)
            + KingPawnShieldEvaluation::eval(board_state)
    }
}
