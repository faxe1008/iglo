use crate::chess::{zobrist_hash::ZHash};


#[derive(Default, Copy, Clone)]
pub struct TranspositionEntry {
    pub zhash: ZHash,
    pub eval: i32,
    pub depth: u32
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

    pub fn lookup(&self, hash: ZHash, depth: u32) -> Option<&TranspositionEntry> {
        let entry = &self.entries[hash.0 as usize % T];
        if entry.zhash == hash && entry.depth >= depth{
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

    pub fn add_entry(&mut self, hash: ZHash,  eval: i32, depth: u32) {
        let entry = &mut self.entries[hash.0 as usize % T];
        if entry.zhash.0 == 0 {
            entry.zhash = hash;
            entry.eval = eval;
            entry.depth = depth;
            self.occupancy += 1;
        } else if entry.zhash == hash && entry.depth < depth {
            entry.depth = depth;
            entry.eval = eval;
        }
    }

}