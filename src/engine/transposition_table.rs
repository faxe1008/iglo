use crate::chess::{zobrist_hash::ZHash, chess_move::Move};


#[derive(Default, Copy, Clone)]
pub struct TranspositionEntry {
    pub zhash: ZHash,
    pub eval: i32
}

pub struct TranspositionTable<const T: usize> {
    entries: [TranspositionEntry; T],
    occupancy: usize
}

impl<const T: usize>  Default for TranspositionTable<T> {
    fn default() -> Self {
        Self { entries: [TranspositionEntry::default();T], occupancy: 0 }
    }
}

impl<const T: usize> TranspositionTable<T> {

    pub fn lookup(&self, hash: ZHash) -> Option<&TranspositionEntry> {
        let entry = &self.entries[hash.0 as usize % T];
        if entry.zhash == hash {
            Some(entry)
        } else {
            None
        }
    }

    pub fn capacity(&self) -> usize {
        T
    }

    pub fn clear(&mut self) {
        self.entries.iter_mut().for_each(|m| {
            m.zhash.0 = 0;
            m.eval = 0;
        });
        self.occupancy = 0;
    }

    pub fn size(&self) -> usize {
        self.occupancy
    }

    pub fn add_entry(&mut self, hash: ZHash,  eval: i32) {
        let entry = &mut self.entries[hash.0 as usize % T];
        if entry.zhash.0 == 0 {
            entry.zhash = hash;
            entry.eval = eval;
            self.occupancy += 1;
        }
    }

}