use std::ops::Not;

use super::{bitboard::BitBoard, chess_move::Move};

const PIECE_TYPE_COUNT: usize = 6;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub enum ChessPiece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub struct CastlingRights {
    pub white_queen_side: bool,
    pub white_king_side: bool,
    pub black_queen_side: bool,
    pub black_king_side: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash, Default)]
pub struct ChessBoard {
    pub white_pieces: [BitBoard; PIECE_TYPE_COUNT],
    pub black_pieces: [BitBoard; PIECE_TYPE_COUNT],

    pub all_white_pieces: BitBoard,
    pub all_black_pieces: BitBoard,
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
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

impl Not for PieceColor {
    type Output = PieceColor;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black
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

impl ToString for CastlingRights {
    fn to_string(&self) -> String {
        let mut text = String::with_capacity(4);
        if self.white_queen_side {
            text.push('Q');
        }
        if self.white_king_side {
            text.push('K');
        }
        if self.black_queen_side {
            text.push('q');
        }
        if self.black_king_side {
            text.push('k');
        }
        text
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
            ChessPiece::King => 1200,
        }
    }
}

impl ChessBoard {

    fn place_piece_of_color(&mut self, piece: ChessPiece, col: PieceColor, index_to_set: usize) {
        let piece_bitboard = if col == PieceColor::White {
            self.all_white_pieces = self.all_white_pieces.set_bit(index_to_set);
            &mut self.white_pieces[piece as usize]
        } else {
            self.all_black_pieces = self.all_black_pieces.set_bit(index_to_set);
            &mut self.black_pieces[piece as usize]
        };
        *piece_bitboard = piece_bitboard.set_bit(index_to_set);
    }

    fn get_piece_at_pos(&self, index: usize) -> Option<(ChessPiece, PieceColor)>
    {
        for (piece_type, bb) in self.white_pieces.iter().enumerate() {
            if bb.get_bit(index) {
                return Some((piece_type.into(), PieceColor::White));
            }
        }
        for (piece_type, bb) in self.black_pieces.iter().enumerate() {
            if bb.get_bit(index) {
                return Some((piece_type.into(), PieceColor::Black));
            }
        }
        None
    }

    fn remove_piece_at_pos(&mut self, piece: ChessPiece, col: PieceColor, index: usize) {
        let piece_bitboard = if col == PieceColor::White {
            self.all_white_pieces = self.all_white_pieces.clear_bit(index);
            &mut self.white_pieces[piece as usize]
        } else {
            self.all_black_pieces = self.all_black_pieces.set_bit(index);
            &mut self.black_pieces[piece as usize]
        };
        *piece_bitboard = piece_bitboard.clear_bit(index);
    }

    pub fn from_fen_notation(fen: &str) -> Result<Self, ()> {
        let mut board = Self::default();
        let mut cur_index: usize = 0;

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
                "p" => board.place_piece_of_color(ChessPiece::Pawn, piece_col, cur_index),
                "n" => board.place_piece_of_color(ChessPiece::Knight, piece_col, cur_index),
                "b" => board.place_piece_of_color(ChessPiece::Bishop, piece_col, cur_index),
                "r" => board.place_piece_of_color(ChessPiece::Rook, piece_col, cur_index),
                "q" => board.place_piece_of_color(ChessPiece::Queen, piece_col, cur_index),
                "k" => board.place_piece_of_color(ChessPiece::King, piece_col, cur_index),
                _ => return Err(()),
            }
            cur_index += 1;
        }
        Ok(board)
    }

    pub fn empty_squares(&self) -> BitBoard {
        BitBoard(!(self.all_white_pieces.0 | self.all_black_pieces.0))
    }
}

impl ChessBoardState {


    pub fn starting_state() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0").unwrap()
    }

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

    pub fn from_fen(text: &str) -> Result<Self, ()> {
        let fen_parts: Vec<&str> = text.trim().split(" ").collect();
        if fen_parts.len() != 6 {
            return Err(());
        }

        Ok(ChessBoardState {
            board: ChessBoard::from_fen_notation(fen_parts[0])?,
            side: PieceColor::try_from(fen_parts[1])?,
            castling_rights: CastlingRights::try_from(fen_parts[2])?,
            en_passant_target: Self::pos_code_to_index(fen_parts[3])?,
            half_moves: fen_parts[4].parse::<u8>().map_err(|_| ())?,
            full_moves: fen_parts[5].parse::<u8>().map_err(|_| ())?,
        })
    }

    pub fn exec_move(&self, mv: Move) -> Self {
        let mut new = *self;

        let (src_piece, src_color) = match self.board.get_piece_at_pos(mv.get_src() as usize) {
            None => panic!("No piece at src pos!"),
            Some(e) => e
        };

        assert!(src_color == self.side, "Moving piece which does not belong to current player!");
        
        // Move is a capture
        if mv.is_capture() {
            let (dst_piece, dst_color) = match self.board.get_piece_at_pos(mv.get_dst() as usize) {
                None => panic!("No piece to capture"),
                Some(e) => e
            };
            assert!(dst_color != src_color, "Can not capture own pieces");
            new.board.remove_piece_at_pos(dst_piece, dst_color, mv.get_dst() as usize);
            new.board.remove_piece_at_pos(src_piece, src_color, mv.get_src() as usize);
            new.board.place_piece_of_color(src_piece, src_color, mv.get_dst() as usize);
        }

        // Move is silent
        if mv.is_silent() || mv.is_double_push() {
            new.board.remove_piece_at_pos(src_piece, src_color, mv.get_src() as usize);
            new.board.place_piece_of_color(src_piece, src_color, mv.get_dst() as usize);
        }

        new.full_moves += 1;
        new.side = !new.side;
        new
    }

}

#[cfg(test)]
mod board_tests {

    use crate::bb;
    use crate::engine::board::BitBoard;
    use crate::engine::board::{CastlingRights, ChessBoard, ChessBoardState, PieceColor};

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
        assert_eq!(
            state.board.all_black_pieces,
            state_expected.board.all_black_pieces
        );
        assert_eq!(
            state.board.all_white_pieces,
            state_expected.board.all_white_pieces
        );
    }

    #[test]
    fn board_from_fen_simple() {
        let board =
            ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
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
                all_white_pieces: bb!(18446462598732840960),
                all_black_pieces: bb!(65535),
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
        let board = ChessBoardState::from_fen(
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
                all_white_pieces: bb!(2037460020536279040),
                all_black_pieces: bb!(10770984612),
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

    #[test]
    fn test_empty_squares(){
        let board_state =
            ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
        assert!(board_state.is_ok());
        let board_state = board_state.unwrap();
        assert_eq!(board_state.board.empty_squares(), BitBoard(281474976645120));
    }
}
