use super::bitboard::BitBoard;
use super::board::{ChessBoardState, ChessPiece, PieceColor};
use super::square::Square;
use core::fmt::Debug;
use std::fmt::Write;
use std::result;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default, Hash)]
pub struct Move(pub u16);

const MOVE_SRC_MASK: u16 = 0x003F;
const MOVE_DST_MASK: u16 = 0x0FC0;
const MOVE_DST_SHIFT: u16 = 6;
const MOVE_TYPE_MASK: u16 = 0xF000;
const MOVE_TYPE_SHIFT: u16 = 12;

const PAWN_START_SQUARE_BB : BitBoard = BitBoard(0xff00000000ff00);
const PAWN_DOUBLE_PUSH_SQUARE_BB : BitBoard = BitBoard(0xffff000000);

pub const PROMOTION_TARGETS: [MoveType; 4] = [
    MoveType::KnightPromotion,
    MoveType::BishopPromotion,
    MoveType::RookPromotion,
    MoveType::QueenPromotion,
];
pub const PROMOTION_CAPTURE_TARGETS: [MoveType; 4] = [
    MoveType::KnightCapPromotion,
    MoveType::BishopCapPromotion,
    MoveType::RookCapPromotion,
    MoveType::QueenCapPromotion,
];

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum MoveType {
    Silent = 0b0000,
    DoublePush = 0b0001,
    CastleKingSide = 0b0010,
    CastleQueenSide = 0b0011,

    Capture = 0b0100,
    EnPassant = 0b0101,

    KnightPromotion = 0b1000,
    BishopPromotion = 0b1001,
    RookPromotion = 0b1010,
    QueenPromotion = 0b1011,

    KnightCapPromotion = 0b1100,
    BishopCapPromotion = 0b1101,
    RookCapPromotion = 0b1110,
    QueenCapPromotion = 0b1111,
}

impl From<u8> for MoveType {
    fn from(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl MoveType {
    pub fn is_capture(&self) -> bool {
        *self as u8 & 0b0100 != 0
    }

    pub fn is_promotion(&self) -> bool {
        *self as u8 & 0b1000 != 0
    }

    pub fn get_promotion_piece_type(&self) -> ChessPiece {
        assert!(self.is_promotion());
        const PIECE_PROMOTIONS: [ChessPiece; 4] = [
            ChessPiece::Knight,
            ChessPiece::Bishop,
            ChessPiece::Rook,
            ChessPiece::Queen,
        ];
        PIECE_PROMOTIONS[*self as usize & 0b0011]
    }
}

impl Move {
    pub const NULL_MOVE : Move = Move(0);

    pub fn new(src: u16, dst: u16, ty: MoveType) -> Self {
        Self(src | (dst << MOVE_DST_SHIFT) | ((ty as u16) << MOVE_TYPE_SHIFT))
    }

    pub fn get_src(&self) -> u16 {
        self.0 & MOVE_SRC_MASK
    }

    pub fn get_dst(&self) -> u16 {
        (self.0 & MOVE_DST_MASK) >> MOVE_DST_SHIFT
    }

    pub fn get_type(&self) -> MoveType {
        (((self.0 & MOVE_TYPE_MASK) >> MOVE_TYPE_SHIFT) as u8).into()
    }

    pub fn is_capture(&self) -> bool {
        ((self.0 & MOVE_TYPE_MASK) >> MOVE_TYPE_SHIFT) & 0b0100 != 0
    }

    pub fn is_silent(&self) -> bool {
        self.get_type() == MoveType::Silent
    }

    pub fn is_double_push(&self) -> bool {
        self.get_type() == MoveType::DoublePush
    }

    pub fn is_en_passant(&self) -> bool {
        self.get_type() == MoveType::EnPassant
    }

    pub fn is_promotion(&self) -> bool {
        (self.0 >> MOVE_TYPE_SHIFT) & 0b1000 != 0
    }

    pub fn promotion_target(&self) -> ChessPiece {
        match (self.0 >> MOVE_TYPE_SHIFT) & 0b11 {
            0 => ChessPiece::Knight,
            1 => ChessPiece::Bishop,
            2 => ChessPiece::Rook,
            3 => ChessPiece::Queen,
            _ => panic!(""),
        }
    }

    pub fn set_move_type(&mut self, move_type: MoveType) {
        self.0 |= (move_type as u16) << MOVE_TYPE_SHIFT;
    }
    pub fn set_is_capture(&mut self, is_capture: bool) {
        if is_capture {
            self.0 |= 0b0100 << MOVE_TYPE_SHIFT;
        } else {
            self.0 &= !(0b0100 << MOVE_TYPE_SHIFT);
        }
    }
}

impl TryFrom<(&str, &ChessBoardState)> for Move {
    type Error = ();

    fn try_from(v: (&str, &ChessBoardState)) -> Result<Self, Self::Error> {
        let (value, board_state) = v;
        if value.len() < 4 {
            return Err(());
        }
        let current_side = board_state.side;
        let opposing_side= !current_side;
        let parse_square_from_slice = |str_slc: &str| -> Result<u16, Self::Error> {
            if str_slc.len() != 2 {
                return Err(());
            }
            let square = Square::from_square_name(str_slc)?;
            if let Some(sq) = square {
                Ok(sq as u16)
            } else {
                return Err(());
            }
        };
        let mv_src = parse_square_from_slice(&value[0..2])?;
        let mv_dst = parse_square_from_slice(&value[2..4])?;
        let mut resulting_move = Move::new(mv_src, mv_dst, MoveType::Silent);

        // Promotion Move
        match value.chars().nth(4) {
            Some('r') => resulting_move.set_move_type(MoveType::RookPromotion),
            Some('b') => resulting_move.set_move_type(MoveType::BishopPromotion),
            Some('n') => resulting_move.set_move_type(MoveType::KnightPromotion),
            Some('q') => resulting_move.set_move_type(MoveType::QueenPromotion),
            Some(_) => {}
            None => {}
        };

        let (src_piece, src_color) = match board_state.board.get_piece_at_pos(mv_src as usize) {
            Some(e) => e,
            _ => return Err(())
        };
        // Capture Move
        if let Some((_piece, col)) = board_state.board.get_piece_at_pos(mv_dst as usize) {
            dbg!(resulting_move);
            if col == current_side {
                // Capturing Own Piece??
                return Err(());
            } else {
                resulting_move.set_is_capture(true);
            }
        } else {
            // Check for Pawn Moves
            if src_piece == ChessPiece::Pawn {
                // En Passant
                if board_state.en_passant_target.is_some()  && mv_dst == board_state.en_passant_target.unwrap()as u16{
                    let enpassanted_pawn = if current_side == PieceColor::White {
                        board_state.en_passant_target.unwrap() + 8
                    } else {
                        board_state.en_passant_target.unwrap() - 8
                    };
                    match board_state.board.get_piece_at_pos(enpassanted_pawn as  usize) {
                        Some((ChessPiece::Pawn, side)) if side == opposing_side => resulting_move.set_move_type(MoveType::EnPassant),
                        _ => {}
                    }
                } else {
                    if PAWN_START_SQUARE_BB.get_bit(mv_src as usize) && PAWN_DOUBLE_PUSH_SQUARE_BB.get_bit(mv_dst as usize) {
                        resulting_move.set_move_type(MoveType::DoublePush);
                    }
                }
            }
           
            // Castling
            if src_piece == ChessPiece::King {
                match (src_color, mv_src, mv_dst) {
                    (PieceColor::White, Square::E1, Square::WHITE_KING_SIDE_CASTLE_SQUARE) => {
                        if board_state.castling_rights.white_king_side {
                            resulting_move.set_move_type(MoveType::CastleKingSide);
                        } else {
                            // Attempt to perform non legal castle
                            return Err(())
                        }
                    },
                    (PieceColor::White, Square::E1, Square::WHITE_QUEEN_SIDE_CASTLE_SQUARE) => {
                        if board_state.castling_rights.white_queen_side {
                            resulting_move.set_move_type(MoveType::CastleQueenSide);
                        } else {
                            // Attempt to perform non legal castle
                            return Err(())
                        }
                    },
                    (PieceColor::Black, Square::E8, Square::BLACK_KING_SIDE_CASTLE_SQAURE) => {
                        if board_state.castling_rights.black_king_side {
                            resulting_move.set_move_type(MoveType::CastleKingSide);
                        } else {
                            // Attempt to perform non legal castle
                            return Err(())
                        }
                    },
                    (PieceColor::Black, Square::E8, Square::BLACK_QUEEN_SIDE_CASTLE_SQAURE) => {
                        if board_state.castling_rights.black_queen_side {
                            resulting_move.set_move_type(MoveType::CastleQueenSide);
                        } else {
                            // Attempt to perform non legal castle
                            return Err(())
                        }
                    },
                    _ => {
                        // TODO: check king moves
                    }
                }
            }

        }


        Ok(resulting_move)
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            &Square::to_square_name(Some(self.get_src() as u8)),
            &Square::to_square_name(Some(self.get_dst() as u8)),
        ))?;
        if self.is_promotion() {
            f.write_char(ChessBoardState::piece_to_fen_notation(
                self.promotion_target(),
                PieceColor::Black,
            ))?;
        }
        Ok(())
    }
}
