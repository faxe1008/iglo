use super::chess_move::Move;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);
const PIECE_TYPE_COUNT: usize = 6;

macro_rules! bb {
    ($expr: expr) => {
        BitBoard($expr)
    };
}

#[derive(Debug)]
pub enum ChessPiece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Debug, PartialEq)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

#[derive(Debug, PartialEq)]
pub struct CastlingRights {
    pub white_queen_side: bool,
    pub white_king_side: bool,
    pub black_queen_side: bool,
    pub black_king_side: bool,
}

#[derive(Default)]
pub struct ChessBoard {
    pub white_pieces: [BitBoard; PIECE_TYPE_COUNT],
    pub black_pieces: [BitBoard; PIECE_TYPE_COUNT],
}

pub struct ChessBoardState {
    pub board: ChessBoard,
    pub side: PieceColor,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<u8>,
    pub half_moves: u8,
    pub full_moves: u8,
}

impl From<usize> for ChessPiece {
    fn from(val: usize) -> Self {
        match val {
            0 => ChessPiece::Pawn,
            1 => ChessPiece::Knight,
            2 => ChessPiece::Bishop,
            3 => ChessPiece::Rook,
            4 => ChessPiece::Queen,
            5 => ChessPiece::King,
            _ => panic!("Non matching value!"),
        }
    }
}

impl TryFrom<&str> for PieceColor {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(());
        }
        match value.chars().nth(0).unwrap() {
            'w' => Ok(PieceColor::White),
            'b' => Ok(PieceColor::Black),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for CastlingRights {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut rights = Self {
            white_queen_side: false,
            white_king_side: false,
            black_queen_side: false,
            black_king_side: false,
        };

        if value == "-" {
            return Ok(rights);
        }

        for chr in value.chars() {
            match chr {
                'Q' => {
                    rights.white_queen_side = true;
                }
                'q' => {
                    rights.black_queen_side = true;
                }
                'K' => {
                    rights.white_king_side = true;
                }
                'k' => {
                    rights.black_king_side = true;
                }
                _ => return Err(()),
            };
        }
        Ok(rights)
    }
}


impl ChessPiece {
   pub fn eval_value(&self) -> u32 {
        match self {
            ChessPiece::Pawn => 100,
            ChessPiece::Knight => 300,
            ChessPiece::Bishop => 315,
            ChessPiece::Rook => 500,
            ChessPiece::Queen => 900,
            ChessPiece::King => 1200
        }
    }
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    pub fn get_bit(&self, pos: usize) -> bool {
        (self.0 & (1u64 << pos)) != 0
    }

    pub fn set_bit(&self, pos: usize) -> Self {
        Self(self.0 | (1u64 << pos))
    }

    pub fn clear_bit(&self, pos: usize) -> Self {
        Self(self.0 & !(1u64 << pos))
    }

    pub fn bit_count(&self) -> u32 {
        self.0.count_ones()
    }
}

impl ChessBoard {
    pub fn from_FEN_notation(fen: &str) -> Result<Self, ()> {
        let mut board = Self::default();
        let mut cur_index: usize = 0;

        let mut place_piece_of_color = |piece: ChessPiece, col: PieceColor, index_to_set: usize| {
            let piece_bitboard = if col == PieceColor::White {
                &mut board.white_pieces[piece as usize].0
            } else {
                &mut board.black_pieces[piece as usize].0
            };
            *piece_bitboard = *piece_bitboard + (1 << index_to_set);
        };

        for chr in fen.chars() {
            if chr.is_digit(10) {
                cur_index += chr.to_digit(10).unwrap() as usize;
                continue;
            }

            if chr == '/' {
                continue;
            }

            let piece_col = if chr.is_uppercase() {
                PieceColor::White
            } else {
                PieceColor::Black
            };

            match &chr.to_lowercase().to_string() as &str {
                "p" => place_piece_of_color(ChessPiece::Pawn, piece_col, cur_index),
                "n" => place_piece_of_color(ChessPiece::Knight, piece_col, cur_index),
                "b" => place_piece_of_color(ChessPiece::Bishop, piece_col, cur_index),
                "r" => place_piece_of_color(ChessPiece::Rook, piece_col, cur_index),
                "q" => place_piece_of_color(ChessPiece::Queen, piece_col, cur_index),
                "k" => place_piece_of_color(ChessPiece::King, piece_col, cur_index),
                _ => return Err(()),
            }
            cur_index += 1;
        }
        Ok(board)
    }

    pub fn piece_at(&self, pos: usize) -> Option<(ChessPiece, PieceColor)> {
        for (piece, bb) in self.white_pieces.iter().enumerate() {
            if bb.0 & (1 << pos) == 1 {
                return Some((piece.into(), PieceColor::White));
            }
        }
        for (piece, bb) in self.black_pieces.iter().enumerate() {
            if bb.0 & (1 << pos) == 1 {
                return Some((piece.into(), PieceColor::Black));
            }
        }
        None
    }
}

impl ChessBoardState {
    pub fn pos_code_to_index(code: &str) -> Result<Option<u8>, ()> {
        if code == "-" {
            return Ok(None);
        }

        if code.len() != 2 {
            return Err(());
        }

        let col_designator = code.chars().nth(0).unwrap();
        let row_designator = code.chars().nth(1).unwrap();

        if col_designator < 'a' || col_designator > 'h' {
            return Err(());
        }

        if row_designator < '1' || row_designator > '9' {
            return Err(());
        }

        let col = col_designator as u8 - 'a' as u8;
        let row = 7 - (row_designator as u8 - '1' as u8);

        Ok(Some(col + row * 8))
    }

    pub fn from_FEN(text: &str) -> Result<Self, ()> {
        let fen_parts: Vec<&str> = text.trim().split(" ").collect();
        if fen_parts.len() != 6 {
            return Err(());
        }

        Ok(ChessBoardState {
            board: ChessBoard::from_FEN_notation(fen_parts[0])?,
            side: PieceColor::try_from(fen_parts[1])?,
            castling_rights: CastlingRights::try_from(fen_parts[2])?,
            en_passant_target: Self::pos_code_to_index(fen_parts[3])?,
            half_moves: fen_parts[4].parse::<u8>().map_err(|_| ())?,
            full_moves: fen_parts[5].parse::<u8>().map_err(|_| ())?,
        })
    }

}

#[cfg(test)]
mod board_tests {

    use crate::engine::board::BitBoard;
    use crate::{
        engine::board::{CastlingRights, ChessBoard, PieceColor, ChessBoardState},
    };

    fn check_board_equality(state: &ChessBoardState, state_expected: &ChessBoardState) {
        for piece_type in 0..=5 {
            let board_white = state.board.white_pieces[piece_type];
            let exp_white = state_expected.board.white_pieces[piece_type];

            assert_eq!(
                board_white.0, exp_white.0,
                "Non matching for white piece_type {:?}",
                piece_type
            );

            let board_black = state.board.black_pieces[piece_type];
            let exp_black = state_expected.board.black_pieces[piece_type];

            assert_eq!(
                board_black.0, exp_black.0,
                "Non matching for black piece_type {:?}",
                piece_type
            );
        }
        assert_eq!(state.side, state_expected.side);
        assert_eq!(state.castling_rights, state_expected.castling_rights);
        assert_eq!(state.en_passant_target, state_expected.en_passant_target);
        assert_eq!(state.half_moves, state_expected.half_moves);
        assert_eq!(state.full_moves, state_expected.full_moves);
    }

    #[test]
    fn board_from_fen_simple() {
        let board =
            ChessBoardState::from_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
        assert!(board.is_ok());

        let expected = ChessBoardState {
            board: ChessBoard {
                white_pieces: [
                    bb!(71776119061217280),
                    bb!(4755801206503243776),
                    bb!(2594073385365405696),
                    bb!(9295429630892703744),
                    bb!(576460752303423488),
                    bb!(1152921504606846976),
                ],
                black_pieces: [bb!(65280), bb!(66), bb!(36), bb!(129), bb!(8), bb!(16)],
            },
            side: PieceColor::White,
            castling_rights: CastlingRights {
                white_queen_side: true,
                white_king_side: true,
                black_queen_side: true,
                black_king_side: true,
            },
            en_passant_target: None,
            half_moves: 0,
            full_moves: 0,
        };

        check_board_equality(&board.unwrap(), &expected);
    }

    #[test]
    fn board_from_fen_complex() {
        let board = ChessBoardState::from_FEN(
            "2r2k1r/1pqn1p2/5P2/1p5p/1b1PQ3/PP5P/1BP3P1/2KRR3 b - - 0 21",
        );

        let expected = ChessBoardState {
            board: ChessBoard {
                white_pieces: [
                    bb!(19284368801398784),
                    bb!(0),
                    bb!(562949953421312),
                    bb!(1729382256910270464),
                    bb!(68719476736),
                    bb!(288230376151711744),
                ],
                black_pieces: [
                    bb!(2181046784),
                    bb!(2048),
                    bb!(8589934592),
                    bb!(132),
                    bb!(1024),
                    bb!(32),
                ],
            },
            side: PieceColor::Black,
            castling_rights: CastlingRights {
                white_queen_side: false,
                white_king_side: false,
                black_queen_side: false,
                black_king_side: false,
            },
            en_passant_target: None,
            half_moves: 0,
            full_moves: 21,
        };

        check_board_equality(&board.unwrap(), &expected);
    }
}
