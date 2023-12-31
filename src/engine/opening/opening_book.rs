use rand::random;

use crate::chess::chess_move::Move;
use crate::chess::zobrist_hash::ZHash;

#[derive(Debug, Clone, Default)]
pub struct OpeningBookEntry {
    pub position: ZHash,
    pub moves: Vec<Move>
}

#[derive(Debug, Clone)]
pub struct OpeningBook {
    pub entries: Vec<OpeningBookEntry>
}

impl OpeningBook {

    fn lookup(&self, hash: ZHash, pick_random_opening: bool) -> Option<Move> {
        None
   }
}