use crate::chess::chess_move::Move;
use crate::chess::zobrist_hash::ZHash;
use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpeningBookEntry {
    pub position: ZHash,
    pub moves: Vec<Move>,
}

impl PartialEq for OpeningBookEntry {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl PartialOrd for OpeningBookEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.position.0.cmp(&other.position.0))
    }
}

impl Eq for OpeningBookEntry {}

impl Ord for OpeningBookEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.position.0.cmp(&other.position.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningBook {
    pub entries: Vec<OpeningBookEntry>,
}

impl OpeningBook {
    pub fn lookup(&self, hash: ZHash, pick_random_opening: bool) -> Option<Move> {
        if let Ok(entry_index) = self
            .entries
            .binary_search_by(|e: &OpeningBookEntry| e.position.0.cmp(&hash.0))
        {
            let entry = &self.entries[entry_index];

            let mv_index = if pick_random_opening {
                let max = entry.moves.len().min(3);
                random::<usize>() % max
            } else {
                0
            };
            Some(entry.moves[mv_index])
        } else {
            None
        }
    }
}
