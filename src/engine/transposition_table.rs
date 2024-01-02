use std::sync::{atomic::AtomicBool, Arc};

use crate::chess::{board::ChessBoardState, zobrist_hash::ZHash};

use super::search::MATE_DISTANCE;

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
    age: u8,
}

pub struct TranspositionTable<const T: usize> {
    entries: [TranspositionEntry; T],
    occupancy: usize,
    age: u8,
}

impl<const T: usize> Default for TranspositionTable<T> {
    fn default() -> Self {
        Self {
            entries: [TranspositionEntry::default(); T],
            occupancy: 0,
            age: 0,
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

    pub fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
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

        let slot_is_empty = entry.zhash.0 == 0;
        let slot_matches = entry.zhash == board_state.zhash;
        let slot_depth_smaller = entry.depth < depth;
        let slot_has_different_age = entry.age != self.age;

        if slot_is_empty {
            entry.zhash = board_state.zhash;
            entry.eval = Self::correct_eval_for_storage(eval, ply_from_root);
            entry.depth = depth;
            entry.age = self.age;
            entry.node_type = node_type;
            self.occupancy += 1;
        } else if slot_matches && (slot_depth_smaller || slot_has_different_age) {
            entry.zhash = board_state.zhash;
            entry.eval = Self::correct_eval_for_storage(eval, ply_from_root);
            entry.depth = depth;
            entry.age = self.age;
            entry.node_type = node_type;
        }
    }
}
