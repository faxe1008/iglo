use lerp::{num_traits::clamp, Lerp};

use crate::chess::{
    bitboard::BitBoard,
    board::{ChessBoardState, ChessPiece, PieceColor},
    square::Square,
};

pub trait EvaluationFunction {
    fn eval(board_state: &ChessBoardState) -> i32;
}

// Strategy: Value per Piece on either side
pub struct PieceCountEvaluation;
impl EvaluationFunction for PieceCountEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        let calc_piece_val_sum = |bitboards: &[BitBoard]| -> i32 {
            bitboards
                .iter()
                .enumerate()
                .map(|(p, board)| board.bit_count() * (ChessPiece::from(p).eval_value()))
                .sum::<u32>() as i32
        };

        calc_piece_val_sum(&board_state.board.white_pieces)
            - calc_piece_val_sum(&board_state.board.black_pieces)
    }
}

fn endgame_lerp_value(board_state: &ChessBoardState) -> f32 {
    // Get number of pieces (non-pawns and non-kings)
    let piece_count: f32 = ((board_state.board.all_black_pieces
        | board_state.board.all_white_pieces)
        & (!board_state.board.white_pieces[ChessPiece::Pawn as usize])
        & (!board_state.board.black_pieces[ChessPiece::Pawn as usize]))
        .bit_count() as f32
        - 2.0;

    clamp(-0.1 * piece_count + 1.4, 0.0, 1.0)
}

// Strategy: Piece Square Table
type PieceSquareTable = [i32; 64];
#[rustfmt::skip]
const GLOBAL_PIECE_SQUARE_TABLE : [PieceSquareTable; 6] = [
    [ 
        // Pawn
        0,   0,   0,   0,   0,   0,   0,   0,
        50,  50,  50,  50,  50,  50,  50,  50,
        10,  10,  20,  30,  30,  20,  10,  10,
         5,   5,  10,  25,  25,  10,   5,   5,
         0,   0,   0,  20,  20,   0,   0,   0,
         5,  -5, -10,   0,   0, -10,  -5,   5,
         5,  10,  10, -20, -20,  10,  10,   5,
         0,   0,   0,   0,   0,   0,   0,   0
    ],
    [   // Knight
        -50,-40,-30,-30,-30,-30,-40,-50, 
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50
    ],
    [   // Bishop
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -20,-10,-10,-10,-10,-10,-10,-20
    ],
    [  // Rook
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0
    ],
    [  // Queen
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5,  5,  5,  5,  0,-10,
         -5,  0,  5,  5,  5,  5,  0, -5,
          0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  0,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20
    ],
    [  // King
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -10,-20,-20,-20,-20,-20,-20,-10,
         20, 20,  0,  0,  0,  0, 20, 20,
         20, 30, 10,  0,  0, 10, 30, 20
    ]
];

#[rustfmt::skip]
const PAWN_END_GAME_TABLE : PieceSquareTable = 
[ 
    // Pawn
    0,   0,   0,   0,   0,   0,   0,   0,
    80,  80,  80,  80,  80,  80,  80,  80,
    50,  50,  50,  50,  50,  50,  50,  50,
    30,  30,  30,  30,  30,  30,  30,  30,
    20,  20,  20,  20,  20,  20,  20,  20,
    10,  10,  10,  10,  10,  10,  10,  10,
    10,  10,  10,  10,  10,  10,  10,  10,
     0,   0,   0,   0,   0,   0,   0,   0
];

#[rustfmt::skip]
const KING_END_GAME_TABLE : PieceSquareTable = 
 [
    // king end game
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50
];

pub struct PieceSquareTableEvaluation;
impl EvaluationFunction for PieceSquareTableEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        let endgame_factor = endgame_lerp_value(board_state);

        #[inline(always)]
        fn eval_sqt(bitboards: &[BitBoard], color: PieceColor, endgame_factor: f32) -> i32 {
            bitboards
                .iter()
                .enumerate()
                .fold(0, |mut sum, (piece, board)| {
                    if board.0 == 0 {
                        return sum;
                    }

                    let square_table = &GLOBAL_PIECE_SQUARE_TABLE[piece];

                    for i in board.into_iter() {
                        let table_pos = if color == PieceColor::White {
                            i
                        } else {
                            63 - i
                        };

                        let piece_value = match ChessPiece::from(piece) {
                            ChessPiece::King => (square_table[table_pos] as f32)
                                .lerp(KING_END_GAME_TABLE[table_pos] as f32, endgame_factor)
                                as i32,
                            ChessPiece::Pawn => (square_table[table_pos] as f32)
                                .lerp(PAWN_END_GAME_TABLE[table_pos] as f32, endgame_factor)
                                as i32,
                            _ => square_table[table_pos],
                        };

                        sum += piece_value;
                    }

                    sum
                })
        }

        eval_sqt(
            &board_state.board.white_pieces,
            PieceColor::White,
            endgame_factor,
        ) - eval_sqt(
            &board_state.board.black_pieces,
            PieceColor::Black,
            endgame_factor,
        )
    }
}

// Strategy: Give Bonus for Passed Pawns
pub struct PassedPawnEvaluation;
impl EvaluationFunction for PassedPawnEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        let endgame_factor = endgame_lerp_value(board_state);
        let eval_passed_pawns = |color: PieceColor| -> i32 {
            let own_pawns = board_state
                .board
                .get_piece_bitboard(ChessPiece::Pawn, color);
            let opposing_pawns = board_state
                .board
                .get_piece_bitboard(ChessPiece::Pawn, !color);

            let mut bonus = 0;

            for pawn in own_pawns {
                let pp_mask = Self::mask_infront_of_pawn(pawn as u64, color)
                    & Self::mask_neighbor_file_of_pawn(pawn as u64);
                if opposing_pawns & pp_mask == BitBoard::EMPTY {
                    bonus +=
                        (endgame_factor * Self::bonus_for_passed_pawn(pawn, color) as f32) as i32;
                }
            }
            bonus
        };

        eval_passed_pawns(PieceColor::White) - eval_passed_pawns(PieceColor::Black)
    }
}

impl PassedPawnEvaluation {
    fn mask_infront_of_pawn(pos: u64, color: PieceColor) -> BitBoard {
        let rank_index = pos / 8;
        if color == PieceColor::White {
            BitBoard(0xFFFFFFFFFFFFFFFF >> (8 * (7 - rank_index + 1)))
        } else {
            BitBoard(0xFFFFFFFFFFFFFFFF << (8 * (rank_index + 1)))
        }
    }

    pub fn mask_neighbor_file_of_pawn(pos: u64) -> BitBoard {
        let file_index = pos % 8;
        const A_FILE_MASK: u64 = 0x101010101010101;

        let mut mask = A_FILE_MASK << file_index;
        if file_index > 0 {
            mask |= A_FILE_MASK << (file_index - 1);
        }
        if file_index < 7 {
            mask |= A_FILE_MASK << (file_index + 1);
        }
        BitBoard(mask)
    }

    pub fn bonus_for_passed_pawn(pos: usize, color: PieceColor) -> i32 {
        const BONUS_FOR_PASSED_PAWN: [i32; 8] = [0, 120, 80, 50, 30, 15, 15, 0];
        let rank = pos / 8;
        if color == PieceColor::White {
            BONUS_FOR_PASSED_PAWN[rank]
        } else {
            BONUS_FOR_PASSED_PAWN[7 - rank]
        }
    }
}

// Strategy Give Bonus for having the bishop pair
pub struct BishopPairEvaluation;
impl EvaluationFunction for BishopPairEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        let eval_bishop_pair = |color: PieceColor| -> i32 {
            let bishop_board = board_state
                .board
                .get_piece_bitboard(ChessPiece::Bishop, color);

            let mut white_bishop_count = 0;
            let mut black_bishop_count = 0;

            for bishop in bishop_board {
                match Square::square_color(bishop as u16) {
                    PieceColor::White => white_bishop_count += 1,
                    PieceColor::Black => black_bishop_count += 1,
                }
            }

            /* Half a pawn bonus for the pair */
            if white_bishop_count >= 1 && black_bishop_count >= 1 {
                ChessPiece::Pawn.eval_value() as i32 / 2
            } else {
                0
            }
        };

        eval_bishop_pair(PieceColor::White) - eval_bishop_pair(PieceColor::Black)
    }
}

pub struct KingPawnShieldEvaluation;
impl EvaluationFunction for KingPawnShieldEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        const PUNISHMENT_PER_PAWN: f32 = -20.0;

        // The earlier in the game the more important
        let end_game_factor = 1.0 - endgame_lerp_value(board_state);

        let white_punishment = {
            let king_bb = board_state
                .board
                .get_piece_bitboard(ChessPiece::King, PieceColor::White);
            let pawn_bb = board_state
                .board
                .get_piece_bitboard(ChessPiece::Pawn, PieceColor::White);

            let white_king_ks_pawns = BitBoard(0xe0000000000000);
            let white_king_ks_squares = BitBoard(0xe000000000000000);

            let white_king_qs_squares = BitBoard(0x700000000000000);
            let white_king_qs_pawns = BitBoard(0x7000000000000);

            if !(king_bb & white_king_ks_squares).is_empty() {
                // King tucked away king side
                let missing_pawns = 3 - (pawn_bb & white_king_ks_pawns).bit_count();
                (end_game_factor * missing_pawns as f32 * PUNISHMENT_PER_PAWN) as i32
            } else if !(king_bb & white_king_qs_squares).is_empty() {
                // King tucked away queen side
                let missing_pawns = 3 - (pawn_bb & white_king_qs_pawns).bit_count();
                (end_game_factor * missing_pawns as f32 * PUNISHMENT_PER_PAWN) as i32
            } else {
                0
            }
        };

        let black_punishment = {
            let king_bb = board_state
                .board
                .get_piece_bitboard(ChessPiece::King, PieceColor::Black);
            let pawn_bb = board_state
                .board
                .get_piece_bitboard(ChessPiece::Pawn, PieceColor::Black);

            let black_king_ks_pawns = BitBoard(0xe000);
            let black_king_ks_squares = BitBoard(0xe0);

            let black_king_qs_pawns = BitBoard(0x700);
            let black_king_qs_squares = BitBoard(0x7);

            if !(king_bb & black_king_ks_squares).is_empty() {
                // King tucked away king side
                let missing_pawns = 3 - (pawn_bb & black_king_ks_pawns).bit_count();
                (end_game_factor * missing_pawns as f32 * PUNISHMENT_PER_PAWN) as i32
            } else if !(king_bb & black_king_qs_squares).is_empty() {
                // King tucked away queen side
                let missing_pawns = 3 - (pawn_bb & black_king_qs_pawns).bit_count();
                (end_game_factor * missing_pawns as f32 * PUNISHMENT_PER_PAWN) as i32
            } else {
                0
            }
        };

        white_punishment - black_punishment
    }
}

pub struct PieceConnectivityEvaluation;
impl EvaluationFunction for PieceConnectivityEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        let eval_connectivity = |color: PieceColor| -> i32 {
            let attacked_squares = board_state.board.squares_attacked_by_side(color, false);

            let defended_pieces = match color {
                PieceColor::White => attacked_squares & board_state.board.all_white_pieces,
                PieceColor::Black => attacked_squares & board_state.board.all_black_pieces,
            };

            defended_pieces.bit_count() as i32 * 5
        };

        eval_connectivity(PieceColor::White) - eval_connectivity(PieceColor::Black)
    }
}

pub struct DoublePawnsEvaluation;
impl EvaluationFunction for DoublePawnsEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
        const PUNISHMET_PER_PAWN: i32 = -10;

        let eval_doubled_pawns = |color: PieceColor| -> i32 {
            let piece_board = board_state
                .board
                .get_piece_bitboard(ChessPiece::Pawn, color);
            let shifted_board = match color {
                PieceColor::White => piece_board.s_no(),
                PieceColor::Black => piece_board.s_so(),
            };

            let doubled_pawns = piece_board & shifted_board;

            doubled_pawns.bit_count() as i32 * PUNISHMET_PER_PAWN
        };

        eval_doubled_pawns(PieceColor::White) - eval_doubled_pawns(PieceColor::Black)
    }
}

#[cfg(test)]
mod eval_tests {
    use crate::{
        chess::board::ChessBoardState,
        engine::board_eval::{
            EvaluationFunction, KingPawnShieldEvaluation, PassedPawnEvaluation,
            PieceCountEvaluation,
        },
    };

    #[test]
    fn eval_start_pos() {
        let start_board =
            ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
        assert!(start_board.is_ok());
        let start_board = start_board.unwrap();
        assert_eq!(PieceCountEvaluation::eval(&start_board), 0);
    }

    #[test]
    fn eval_passed_pawn() {
        let board_state_passer =
            ChessBoardState::from_fen("4k3/8/6P1/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(PassedPawnEvaluation::eval(&board_state_passer) > 0);

        let board_state_no_passer =
            ChessBoardState::from_fen("4k3/6p1/6P1/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(PassedPawnEvaluation::eval(&board_state_no_passer) == 0);

        let board_opposing_passer =
            ChessBoardState::from_fen("4k3/8/8/8/6p1/8/8/4K3 w - - 0 1").unwrap();
        assert!(PassedPawnEvaluation::eval(&board_opposing_passer) < 0);
    }

    #[test]
    fn eval_king_pawn_shield() {
        let board_white_damaged_shield = ChessBoardState::from_fen(
            "rnbq2kr/pppppppp/8/4bn2/3Q1N2/1PN1BB2/P1PPPPPP/1KR4R w Kkq - 0 1",
        )
        .unwrap();
        assert!(KingPawnShieldEvaluation::eval(&board_white_damaged_shield) < 0);

        let board_black_damaged_shield = ChessBoardState::from_fen(
            "rnbq2kr/pppppp1p/6p1/4bn2/3Q1N2/2N1BB2/PPPPPPPP/1KR4R w Kkq - 0 1",
        )
        .unwrap();
        assert!(KingPawnShieldEvaluation::eval(&board_black_damaged_shield) > 0);

        assert!(KingPawnShieldEvaluation::eval(&ChessBoardState::starting_state()) == 0);
    }
}
