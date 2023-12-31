use std::{cmp::{min, max}, sync::{Arc, atomic::AtomicBool}};

use crate::chess::{board::ChessBoardState, zobrist_hash::ZHash};

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
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
    pub node_type: NodeType,
}

pub struct TranspositionTable<const T: usize> {
    entries: [TranspositionEntry; T],
    occupancy: usize,
}

impl<const T: usize> Default for TranspositionTable<T> {
    fn default() -> Self {
        Self {
            entries: [TranspositionEntry::default(); T],
            occupancy: 0,
        }
    }
}

impl<const T: usize> TranspositionTable<T> {
    pub fn lookup(&self, hash: ZHash, depth: u16, alpha: &mut i32, beta: &mut i32) -> Option<i32> {
        let entry = &self.entries[hash.0 as usize % T];
        if entry.zhash == hash && entry.depth >= depth {
            if entry.node_type == NodeType::Exact {
                return Some(entry.eval);
            } else if entry.node_type == NodeType::UpperBound {
                *beta = min(*beta, entry.eval);
            } else if entry.node_type == NodeType::LowerBound  {
                *alpha = max(*alpha, entry.eval);
            }
            
            if alpha >= beta {
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

    pub fn add_entry(
        &mut self,
        board_state: &ChessBoardState,
        eval: i32,
        depth: u16,
        node_type: NodeType,
        stop: &Arc<AtomicBool>
    ) {
        if stop.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let entry = &mut self.entries[board_state.zhash.0 as usize % T];
        if entry.zhash.0 == 0 {
            entry.zhash = board_state.zhash;
            entry.eval = eval;
            entry.depth = depth;
            entry.node_type = node_type;
            self.occupancy += 1;
        } else if entry.zhash == board_state.zhash && entry.depth < depth {
            entry.depth = depth;
            entry.eval = eval;
            entry.node_type = node_type;
        }
    }
}
