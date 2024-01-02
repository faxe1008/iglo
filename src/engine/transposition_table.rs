use std::{
    cmp::{max, min},
    sync::{atomic::AtomicBool, Arc},
};

use crate::chess::{board::ChessBoardState, zobrist_hash::ZHash};

use super::search::{Searcher, CHECKMATE, MATE_DISTANCE};

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
    pub fn lookup(
        &self,
        hash: ZHash,
        depth: u16,
        ply_from_root: u16,
        alpha: i32,
        beta: i32,
    ) -> Option<i32> {
        let entry = &self.entries[hash.0 as usize % T];
        if entry.zhash == hash && entry.depth >= depth {
            let eval = Self::correct_fetched_score(entry.eval, ply_from_root);

            if entry.node_type == NodeType::Exact {
                return Some(eval);
            } else if entry.node_type == NodeType::UpperBound && entry.eval <= alpha {
                return Some(alpha);
            } else if entry.node_type == NodeType::LowerBound && entry.eval >= beta {
                return Some(beta);
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

    pub fn hashfull(&self) -> usize {
        (1000 * self.occupancy) / T
    }

    pub fn correct_eval_for_storage(eval: i32, ply_from_root: u16) -> i32 {
        if eval >= MATE_DISTANCE {
            eval + ply_from_root as i32
        } else if eval <= -MATE_DISTANCE {
            eval - ply_from_root as i32
        } else {
            eval
        }
    }

    pub fn correct_fetched_score(eval: i32, ply_from_root: u16) -> i32 {
        if eval >= MATE_DISTANCE {
            eval - ply_from_root as i32
        } else if eval <= -MATE_DISTANCE {
            eval + ply_from_root as i32
        } else {
            eval
        }
    }

    pub fn add_entry(
        &mut self,
        board_state: &ChessBoardState,
        eval: i32,
        depth: u16,
        ply_from_root: u16,
        node_type: NodeType,
        stop: &Arc<AtomicBool>,
    ) {
        if stop.load(std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let entry = &mut self.entries[board_state.zhash.0 as usize % T];
        if entry.zhash.0 == 0 {
            entry.zhash = board_state.zhash;
            entry.eval = Self::correct_eval_for_storage(eval, ply_from_root);
            entry.depth = depth;
            entry.node_type = node_type;
            self.occupancy += 1;
        } else if entry.zhash == board_state.zhash && entry.depth < depth {
            entry.depth = depth;
            entry.eval = Self::correct_eval_for_storage(eval, ply_from_root);
            entry.node_type = node_type;
        }
    }
}
