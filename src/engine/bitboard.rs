#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

#[macro_export]
macro_rules! bb {
    ($expr: expr) => {
        BitBoard($expr)
    };
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    pub const NOT_A_FILE: u64 = 0xfefefefefefefefe;
    pub const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
    pub const RANK_4: u64 = 1095216660480;
    pub const RANK_5: u64 = 4278190080;

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

    #[inline(always)]
    pub fn for_each_set_bit<F: FnMut(usize)>(&self, functor: &mut F) {
        if *self == BitBoard::EMPTY {
            return;
        }
        for i in 0..64 {
            functor(i);
        }
    }

    pub fn sNoWe(&self) -> Self {
        Self((self.0 & Self::NOT_A_FILE) >> 9) //
    }

    pub fn sNo(&self) -> Self {
        Self(self.0 >> 8) //
    }

    pub fn sNoEa(&self) -> Self {
        Self((self.0 & Self::NOT_H_FILE) >> 7) //
    }

    pub fn sWe(&self) -> Self {
        Self((self.0 & Self::NOT_A_FILE) >> 1) //
    }

    pub fn sEa(&self) -> Self {
        Self((self.0 & Self::NOT_H_FILE) << 1) //
    }

    pub fn sSoWe(&self) -> Self {
        Self((self.0 & Self::NOT_A_FILE) << 7) //
    }

    pub fn sSo(&self) -> Self {
        Self(self.0 << 8) //
    }
    pub fn sSoEa(&self) -> Self {
        Self((self.0 & Self::NOT_H_FILE) << 9) //
    }
}

impl std::ops::BitAnd<BitBoard> for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: BitBoard) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}
impl std::ops::BitAnd<u64> for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: u64) -> Self::Output {
        BitBoard(self.0 & rhs)
    }
}
