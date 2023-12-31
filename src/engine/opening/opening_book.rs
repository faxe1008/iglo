
use serde::{Serialize, Deserialize};
use crate::chess::chess_move::Move;
use crate::chess::zobrist_hash::ZHash;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpeningBookEntry {
    pub position: ZHash,
    pub moves: Vec<Move>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningBook {
    pub entries: Vec<OpeningBookEntry>
}

impl OpeningBook {

    fn lookup(&self, hash: ZHash, pick_random_opening: bool) -> Option<Move> {
        None
   }
}