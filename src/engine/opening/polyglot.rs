use std::collections::HashMap;

use crate::chess::{board::{ChessBoardState, ChessPiece, PieceColor}, chess_move::{Move, MoveType}, square::Square};

pub trait OpeningBook {
    fn get(&self, board_state: &ChessBoardState) -> Vec<Move>;
}

pub struct PolyglotEntry {
    pub key: u64,
    pub move_: u16,
    pub weight: u16,
    pub learn: u32,
}


impl PolyglotEntry {
    fn to_move(&self, board_state: &ChessBoardState) -> Move {
        let to_file = (self.move_ & 0b111) as u8;
        let to_row = 7 - ((self.move_ >> 3) & 0b111) as u8;
        let from_file = ((self.move_ >> 6) & 0b111) as u8;
        let from_row = 7 - ((self.move_ >> 9) & 0b111) as u8;
        let promotion_piece = ((self.move_ >> 12) & 0b111) as u8;

        let mut to_square = Square::square_from_pos(to_file.into(), to_row.into());
        let from_square = Square::square_from_pos(from_file.into(), from_row.into());

        // check if the move is capture
        let is_capture = board_state.board.get_piece_at_pos(to_square as usize ).is_some();

        // check if the move is a double push
        let piece_at_source_pos = board_state.board.get_piece_at_pos(from_square as usize).unwrap();
        if piece_at_source_pos.0 == ChessPiece::Pawn && ((to_row == 3 && from_row == 1) || (to_row == 4 && from_row == 6)){
            return Move::new(from_square, to_square, MoveType::DoublePush);
        }

        // check if the move is a castle
        if piece_at_source_pos.0 == ChessPiece::King && from_file == 4 && to_file == 7 {
            if piece_at_source_pos.1 == PieceColor::White {
                to_square = Square::G1;
            } else {
                to_square = Square::G8;
            }
            return Move::new(from_square, to_square, MoveType::CastleKingSide);
        }
        if piece_at_source_pos.0 == ChessPiece::King && from_file == 4 && to_file == 0 {
            if piece_at_source_pos.1 == PieceColor::White {
                to_square = Square::C1;
            } else {
                to_square = Square::C8;
            }
            return Move::new(from_square, to_square, MoveType::CastleQueenSide);
        }
      
        // check if the move is a en passant capture
        if piece_at_source_pos.0 == ChessPiece::Pawn && board_state.en_passant_target.is_some() && to_square as u8 == board_state.en_passant_target.unwrap() {
            return Move::new(from_square, to_square, MoveType::EnPassant);
        };

        // check if the move is a promotion
        if self.move_ & (0b111 << 12) != 0 {
            let move_type: MoveType = match (promotion_piece, is_capture) {
                (0, false) => MoveType::KnightPromotion,
                (0, true) => MoveType::KnightCapPromotion,
                (1, false) => MoveType::BishopPromotion,
                (1, true) => MoveType::BishopCapPromotion,
                (2, false) => MoveType::RookPromotion,
                (2, true) => MoveType::RookCapPromotion,
                (3, false) => MoveType::QueenPromotion,
                (3, true) => MoveType::QueenCapPromotion,
                _ => panic!("Invalid promotion piece"),
            };
            return Move::new(from_square, to_square, move_type);
        }

        if is_capture {
            return Move::new(from_square, to_square, MoveType::Capture);
        }

        return Move::new(from_square, to_square, MoveType::Silent);
    }
}

pub struct PolyglotOpeningBook {
    entries: HashMap<u64, Vec<PolyglotEntry>>,
}

impl PolyglotOpeningBook {
    pub fn new(entries: HashMap<u64, Vec<PolyglotEntry>>) -> Self {
        Self { entries }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut entries = HashMap::new();
        for i in 0..bytes.len() / 16 {
            let key = u64::from_be_bytes([
                bytes[i * 16],
                bytes[i * 16 + 1],
                bytes[i * 16 + 2],
                bytes[i * 16 + 3],
                bytes[i * 16 + 4],
                bytes[i * 16 + 5],
                bytes[i * 16 + 6],
                bytes[i * 16 + 7],
            ]);
            let move_ = u16::from_be_bytes([
                bytes[i * 16 + 8],
                bytes[i * 16 + 9],
            ]);
            let weight = u16::from_be_bytes([
                bytes[i * 16 + 10],
                bytes[i * 16 + 11],
            ]);
            let learn = u32::from_be_bytes([
                bytes[i * 16 + 12],
                bytes[i * 16 + 13],
                bytes[i * 16 + 14],
                bytes[i * 16 + 15],
            ]);

            entries.entry(key).or_insert(Vec::new()).push(PolyglotEntry {
                key,
                move_,
                weight,
                learn,
            });
        }

        // Sort descending by weight
        for (_, entry) in entries.iter_mut() {
            entry.sort_by(|a, b| b.weight.cmp(&a.weight));
        }

        Self::new(entries)
    }


   
}

impl OpeningBook for PolyglotOpeningBook {
    fn get(&self, board_state: &ChessBoardState) -> Vec<Move> {
        let mut moves = Vec::new();
        let key = board_state.zhash.0;
       
        if let Some(entries) = self.entries.get(&key) {
            for entry in entries {
                moves.push(entry.to_move(board_state));
            }
        }

        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::{chess::{board::ChessBoardState, chess_move::MoveType, square::Square}, engine::opening::polyglot::PolyglotOpeningBook};

    use super::OpeningBook;

    #[test]
    fn test_polyglot_deserialization() {
        let bytes: Vec<u8> = vec![
            0x00, 0x00, 0x29, 0x13, 0x39, 0x5F, 0x74, 0x7C, 0x06, 0xE3, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x39, 0x14, 0x43, 0x33, 0x39, 0x92, 0x07, 0x95, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
        ];

        let book = PolyglotOpeningBook::from_bytes(&bytes);

        assert_eq!(book.entries.len(), 2);
    }

    #[test]
    fn test_polyglot_find_silent_move() {
        let bytes: Vec<u8> = vec![
            0x06, 0x64, 0x9B, 0xA6, 0x9B, 0x8C, 0x9F, 0xF8, 0x00, 0xA6, 0x00, 0xC2, 0x00, 0x00, 0x00, 0x00
        ];
        let board_state = ChessBoardState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let book = PolyglotOpeningBook::from_bytes(&bytes);

        let moves = book.get(&board_state);
        assert_eq!(moves.len() , 1);
        assert_eq!(moves[0].get_src() , Square::C1);
        assert_eq!(moves[0].get_dst() , Square::G5);
        assert_eq!(moves[0].get_type() , MoveType::Silent);
    }

    #[test]
    fn test_polyglot_find_double_push() {
        let bytes = vec![0x46, 0x3B, 0x96, 0x18, 0x16, 0x91, 0xFC, 0x9C, 0x03, 0x1C, 0x3F, 0x95, 0x00, 0x00, 0x00, 0x00];
        let board_state = ChessBoardState::starting_state();

        let book = PolyglotOpeningBook::from_bytes(&bytes);
        let moves = book.get(&board_state);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].get_src(), Square::E2);
        assert_eq!(moves[0].get_dst(), Square::E4);
        assert_eq!(moves[0].get_type(), MoveType::DoublePush);
    }

    #[test]
    fn test_capture() {
        let bytes = vec![0xC2, 0x63, 0x96, 0xEE, 0x70, 0x0C, 0x22, 0xF2, 0x08, 0xDC, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00];
        let board_state = ChessBoardState::from_fen("rnbqkbnr/ppp1pppp/8/3p4/3PP3/8/PPP2PPP/RNBQKBNR b KQkq e3 0 1").unwrap();

        let book = PolyglotOpeningBook::from_bytes(&bytes);
        let moves = book.get(&board_state);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].get_src(), Square::D5);
        assert_eq!(moves[0].get_dst(), Square::E4);
        assert_eq!(moves[0].get_type(), MoveType::Capture);
    }


}
