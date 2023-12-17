use super::board::ChessPiece;

pub struct Move(pub u16);

const MOVE_SRC_MASK: u16 = 0x003F;
const MOVE_DST_MASK: u16 = 0x0FC0;
const MOVE_DST_SHIFT: u16 = 6;
const MOVE_TYPE_MASK: u16 = 0x7000;
const MOVE_TYPE_SHIFT: u16 = 12;

#[macro_export] macro_rules! c_move {
    ($src: expr, $dst: expr, $ty: expr) => {
        Move(($src as u16) | (($dst as u16) << 6) | (($ty as u16) << 12))
    };
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum MoveType {
    Silent = 0b0000,
    DoublePush = 0b0001,
    Castle = 0b0010,

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
    pub fn get_src(&self) -> u16 {
        self.0 & MOVE_SRC_MASK
    }

    pub fn get_dst(&self) -> u16 {
        (self.0 & MOVE_DST_MASK) >> MOVE_DST_SHIFT
    }

    pub fn get_type(&self) -> MoveType {
        (((self.0 & MOVE_TYPE_MASK) >> MOVE_TYPE_SHIFT) as u8).into()
    }
}
