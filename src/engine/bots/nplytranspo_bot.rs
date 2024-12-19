use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    chess::{board::ChessBoardState, chess_move::Move},
    engine::{
        board_eval::{
            BishopPairEvaluation, DoublePawnsEvaluation, EvaluationFunction,
            KingPawnShieldEvaluation, PassedPawnEvaluation, PieceCountEvaluation,
            PieceSquareTableEvaluation,
        },
        bot::ChessBot,
        opening::polyglot::{OpeningBook, PolyglotOpeningBook},
        search::Searcher,
        time_control::TimeControl,
        transposition_table::TranspositionEntry,
    },
};

pub const TABLE_SIZE: usize = 64 * 1024 * 1024;
pub const TABLE_ENTRY_SIZE: usize = std::mem::size_of::<TranspositionEntry>();
pub const TABLE_ENTRY_COUNT: usize = TABLE_SIZE / TABLE_ENTRY_SIZE;

// include bytes from file /home/faxe/priv/iglo/src/engine/opening/Openings.bin as OPENING_BOOK_DATA

const OPENING_BOOK_DATA: &'static [u8] =
    include_bytes!("/home/faxe/priv/iglo/src/engine/opening/Openings.bin");

pub struct NPlyTranspoBot {
    searcher: Searcher<TABLE_ENTRY_COUNT, NPlyTranspoBotEval>,
    opening_book: PolyglotOpeningBook,
    use_openening_book: bool,
}

impl Default for NPlyTranspoBot {
    fn default() -> Self {
        let opening_book = PolyglotOpeningBook::from_bytes(OPENING_BOOK_DATA);
        Self {
            searcher: Searcher::new(),
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
        let cur_board_eval = self.eval(board_state);
        println!("info score cp {}", cur_board_eval as f32 / 100.0);

        if self.use_openening_book {
            let moves = self.opening_book.get(board_state);
            if !moves.is_empty() {
                let legal_moves = board_state.generate_legal_moves_for_current_player::<false>();

                // verify that each move in moves is in the legal moves
                for m in &moves {
                    if !legal_moves.contains(m) {
                        //panic!("Illegal move in opening book: {:?}", m);
                    }
                }

                println!("info string book move");
                return moves[0];
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

#[derive(Default)]
struct NPlyTranspoBotEval();
impl EvaluationFunction for NPlyTranspoBotEval {
    fn eval(&mut self, board_state: &crate::chess::board::ChessBoardState) -> i32 {
        PieceCountEvaluation.eval(board_state)
            + PieceSquareTableEvaluation.eval(board_state)
            + PassedPawnEvaluation.eval(board_state)
            + BishopPairEvaluation.eval(board_state)
            + KingPawnShieldEvaluation.eval(board_state)
            + DoublePawnsEvaluation.eval(board_state)
    }
}

impl EvaluationFunction for NPlyTranspoBot {
    fn eval(&mut self, board_state: &crate::chess::board::ChessBoardState) -> i32 {
        self.searcher.eval_fn.eval(board_state)
    }
}
