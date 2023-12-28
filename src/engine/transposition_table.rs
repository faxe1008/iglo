use crate::chess::{zobrist_hash::ZHash};

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Exact
    }
}


#[derive(Default, Copy, Clone)]
pub struct TranspositionEntry {
    pub zhash: ZHash,
    pub eval: i32,
    pub depth: u16,
    pub node_type: NodeType
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

    pub fn lookup(&self, hash: ZHash, depth: u16, alpha: i32, beta: i32) -> Option<i32> {
        let entry = &self.entries[hash.0 as usize % T];
        if entry.zhash == hash && entry.depth >= depth{
            if entry.node_type == NodeType::Exact {
                return Some(entry.eval);
            } else if entry.node_type == NodeType::UpperBound && entry.eval <= alpha {
                return Some(entry.eval)
            } else if entry.node_type == NodeType::LowerBound && entry.eval >= beta {
                return Some(entry.eval)
            }
        }
        None
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

    pub fn add_entry(&mut self, hash: ZHash,  eval: i32, depth: u16, node_type: NodeType) {
        let entry = &mut self.entries[hash.0 as usize % T];
        if entry.zhash.0 == 0 {
            entry.zhash = hash;
            entry.eval = eval;
            entry.depth = depth;
            entry.node_type = node_type;
            self.occupancy += 1;
        } else if entry.zhash == hash && entry.depth < depth {
            entry.depth = depth;
            entry.eval = eval;
            entry.node_type = node_type;
        }
    }

}