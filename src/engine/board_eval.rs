use lerp::Lerp;

use crate::chess::{
    bitboard::BitBoard,
    board::{ChessBoardState, ChessPiece, PieceColor},
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
    let piece_count: u32 = [
        ChessPiece::Bishop,
        ChessPiece::Knight,
        ChessPiece::Rook,
        ChessPiece::Queen,
    ]
    .iter()
    .map(|&p| {
        board_state.board.white_pieces[p as usize].bit_count()
            + board_state.board.black_pieces[p as usize].bit_count()
    })
    .sum();

    1.0 - piece_count as f32 / 14.0
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
        let eval_sqt = |bitboards: &[BitBoard], color: PieceColor| {
            let mut sum = 0;
            for (piece, board) in bitboards.iter().enumerate() {
                if board.0 == 0 {
                    continue;
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

                    sum = sum + piece_value;
                }
            }
            sum
        };

        eval_sqt(&board_state.board.white_pieces, PieceColor::White)
            - eval_sqt(&board_state.board.black_pieces, PieceColor::Black)
    }
}

pub struct PassedPawnEvaluation;
impl EvaluationFunction for PassedPawnEvaluation {
    fn eval(board_state: &ChessBoardState) -> i32 {
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
                    bonus += Self::bonus_for_passed_pawn(pawn, color);
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
        const BONUS_FOR_PASSED_PAWN: [i32; 8] = [0, 90, 70, 45, 30, 15, 0, 0];
        let rank = pos / 8;
        if color == PieceColor::White {
            BONUS_FOR_PASSED_PAWN[rank]
        } else {
            BONUS_FOR_PASSED_PAWN[7 - rank]
        }
    }
}

#[cfg(test)]
mod eval_tests {
    use crate::{
        chess::board::ChessBoardState,
        engine::board_eval::{EvaluationFunction, PassedPawnEvaluation, PieceCountEvaluation},
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
}
