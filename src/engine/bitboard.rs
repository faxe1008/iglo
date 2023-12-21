
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
#[repr(packed)]
pub struct MagicEntry {
    pub blocker_mask: BitBoard,
    pub magic: u64,
    pub index_bits: u8,
}
pub struct BitBoardSubsetIter {
    set: BitBoard,
    subset: BitBoard,
    finished: bool
}

#[macro_export]
macro_rules! bb {
    ($expr: expr) => {
        BitBoard($expr)
    };
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(0xFFFFFFFFFFFFFFFF);

    pub const NOT_A_FILE: u64 = 0xfefefefefefefefe;
    pub const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;
    pub const RANK_4: u64 = 1095216660480;
    pub const RANK_5: u64 = 4278190080;

    #[must_use]
    pub fn get_bit(&self, pos: usize) -> bool {
        (self.0 & (1u64 << pos)) != 0
    }

    #[must_use]
    pub fn set_bit(&self, pos: usize) -> Self {
        Self(self.0 | (1u64 << pos))
    }

    #[must_use]
    pub fn clear_bit(&self, pos: usize) -> Self {
        Self(self.0 & !(1u64 << pos))
    }

    pub fn bit_count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn iter_subsets(self) -> BitBoardSubsetIter {
        BitBoardSubsetIter {
            set: self,
            subset: Self::EMPTY,
            finished: false
        }
    }

    #[must_use]
    pub fn s_no_we(&self) -> Self {
        Self((self.0 & Self::NOT_A_FILE) >> 9) //
    }
    #[must_use]
    pub fn s_no(&self) -> Self {
        Self(self.0 >> 8) //
    }

    #[must_use]
    pub fn s_no_ea(&self) -> Self {
        Self((self.0 & Self::NOT_H_FILE) >> 7) //
    }

    #[must_use]
    pub fn s_we(&self) -> Self {
        Self((self.0 & Self::NOT_A_FILE) >> 1) //
    }

    #[must_use]
    pub fn s_ea(&self) -> Self {
        Self((self.0 & Self::NOT_H_FILE) << 1) //
    }

    #[must_use]
    pub fn s_so_we(&self) -> Self {
        Self((self.0 & Self::NOT_A_FILE) << 7) //
    }

    #[must_use]
    pub fn s_so(&self) -> Self {
        Self(self.0 << 8) //
    }

    #[must_use]
    pub fn s_so_ea(&self) -> Self {
        Self((self.0 & Self::NOT_H_FILE) << 9) //
    }
}

impl MagicEntry {
    pub fn magic_index(&self, blockers: BitBoard) -> usize {
        let blockers = blockers & self.blocker_mask;
        let hash = blockers.0.wrapping_mul(self.magic);
        let index = (hash >> (64 - self.index_bits)) as usize;
        index
    }
}

pub struct BitBoardIterator {
    value: BitBoard,
}

impl BitBoardIterator {
    fn new(bitboard: BitBoard) -> Self {
        Self {
            value: bitboard,
        }
    }
}

impl Iterator for BitBoardIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value.0 == 0 {
            return None;
        }
        let lsb = self.value.0.trailing_zeros() as usize;
        self.value = self.value.clear_bit(lsb);
        Some(lsb)
    }
}

impl IntoIterator for BitBoard {
    type Item = usize;

    type IntoIter = BitBoardIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIterator::new(self)
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

impl std::ops::BitOr<BitBoard> for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: BitBoard) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl std::ops::Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl Iterator for BitBoardSubsetIter {
    type Item = BitBoard;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let current = self.subset;
        self.subset.0 = self.subset.0.wrapping_sub(self.set.0) & self.set.0;
        self.finished = self.subset.0 == 0;
        Some(current)
    }
}



#[cfg(test)]
mod bitboard_tests {
    use super::{BitBoard, BitBoardIterator};

    #[test]
    fn test_iterator() {
        let bitboard = BitBoard(0b0001);

        let mut iter = BitBoardIterator::new(bitboard);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iterator_complex() {
        let bitboard = BitBoard(0b11001 | (1 << 63));

        let mut iter = BitBoardIterator::new(bitboard);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(63));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_subset_iterator() {
        let mut bitboard = BitBoard::EMPTY;

        bitboard = bitboard.set_bit(10);
        bitboard = bitboard.set_bit(5);
        bitboard = bitboard.set_bit(1);

        let subsets : Vec<BitBoard> = bitboard.iter_subsets().collect();
        assert!(subsets.len() == 8);
        assert!(subsets.contains(&BitBoard(0 << 1 | 0 << 5 | 0 << 10)));
        assert!(subsets.contains(&BitBoard(1 << 1 | 0 << 5 | 0 << 10)));
        assert!(subsets.contains(&BitBoard(0 << 1 | 1 << 5 | 0 << 10)));
        assert!(subsets.contains(&BitBoard(1 << 1 | 1 << 5 | 0 << 10)));
        assert!(subsets.contains(&BitBoard(0 << 1 | 0 << 5 | 1 << 10)));
        assert!(subsets.contains(&BitBoard(1 << 1 | 0 << 5 | 1 << 10)));
        assert!(subsets.contains(&BitBoard(0 << 1 | 1 << 5 | 1 << 10)));
        assert!(subsets.contains(&BitBoard(1 << 1 | 1 << 5 | 1 << 10)));
    }
}
