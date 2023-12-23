use super::board::{ChessPiece, PieceColor, ChessBoardState};
use super::square::Square;
use core::fmt::Debug;
use std::fmt::Write;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default, Hash)]
pub struct Move(pub u16);

const MOVE_SRC_MASK: u16 = 0x003F;
const MOVE_DST_MASK: u16 = 0x0FC0;
const MOVE_DST_SHIFT: u16 = 6;
const MOVE_TYPE_MASK: u16 = 0xF000;
const MOVE_TYPE_SHIFT: u16 = 12;

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

}



impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            &Square::to_square_name(Some(self.get_src() as u8)),
            &Square::to_square_name(Some(self.get_dst() as u8)),
        ))?;
        if self.is_promotion() {
            f.write_char(ChessBoardState::piece_to_fen_notation(self.promotion_target(), PieceColor::Black))?;
        }
        Ok(())
    }
}