use std::ops::Not;

use crate::chess::{chess_move::MoveType, square::Square};

use super::{bitboard::BitBoard, chess_move::Move, zobrist_hash::ZHash};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub enum ChessPiece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub struct CastlingRights(pub u8);

impl CastlingRights {

    #[inline(always)]
    pub fn all() -> Self {
        Self(15)
    }

    #[inline(always)]
    pub fn none() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub fn white_queen_side(&self) -> bool {
        self.0 & 1 != 0
    }
    #[inline(always)]
    pub fn white_king_side(&self) -> bool {
        self.0 & 2 != 0
    }
    #[inline(always)]
    pub fn black_queen_side(&self) -> bool {
        self.0 & 4 != 0
    }
    #[inline(always)]
    pub fn black_king_side(&self) -> bool {
        self.0 & 8 != 0
    }

    #[inline(always)]
    pub fn set_white_queen_side(&mut self, val: bool) {
        if val {
            self.0 |= 1;
        } else {
            self.0 &= !1;
        }
    }

    #[inline(always)]
    pub fn set_white_king_side(&mut self, val: bool) {
        if val {
            self.0 |= 2;
        } else {
            self.0 &= !2;
        }
    }

    #[inline(always)]
    pub fn set_black_queen_side(&mut self, val: bool) {
        if val {
            self.0 |= 4;
        } else {
            self.0 &= !4;
        }
    }

    #[inline(always)]
    pub fn set_black_king_side(&mut self, val: bool) {
        if val {
            self.0 |= 8;
        } else {
            self.0 &= !8;
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub struct ChessBoard {
    pub white_pieces: [BitBoard; ChessPiece::PIECE_TYPE_COUNT],
    pub black_pieces: [BitBoard; ChessPiece::PIECE_TYPE_COUNT],

    pub all_white_pieces: BitBoard,
    pub all_black_pieces: BitBoard,

    piece_board: [Option<(ChessPiece, PieceColor)>; Square::NUM as usize],
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash)]
pub struct ChessBoardState {
    pub board: ChessBoard,
    pub side: PieceColor,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<u8>,
    pub half_moves: u8,
    pub full_moves: u8,
    pub zhash: ZHash,
}

impl From<usize> for ChessPiece {
    fn from(val: usize) -> Self {
        match val {
            0 => ChessPiece::Pawn,
            1 => ChessPiece::Knight,
            2 => ChessPiece::Bishop,
            3 => ChessPiece::Rook,
            4 => ChessPiece::Queen,
            5 => ChessPiece::King,
            _ => panic!("Non matching value!"),
        }
    }
}

impl Not for PieceColor {
    type Output = PieceColor;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

impl PieceColor {
    pub const PIECE_COLOR_COUNT: usize = 2;
    pub fn as_display_str(&self) -> String {
        match self {
            PieceColor::White => "White",
            PieceColor::Black => "Black",
        }
        .to_string()
    }
}

impl TryFrom<&str> for PieceColor {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 1 {
            return Err(());
        }
        match value.chars().nth(0).unwrap() {
            'w' => Ok(PieceColor::White),
            'b' => Ok(PieceColor::Black),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for CastlingRights {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut rights = CastlingRights(0);

        if value == "-" {
            return Ok(rights);
        }

        for chr in value.chars() {
            match chr {
                'Q' => {
                    rights.set_white_queen_side(true);
                }
                'q' => {
                    rights.set_black_queen_side(true);
                }
                'K' => {
                    rights.set_white_king_side(true);
                }
                'k' => {
                    rights.set_black_king_side(true);
                }
                _ => return Err(()),
            };
        }
        Ok(rights)
    }
}

impl ToString for CastlingRights {
    fn to_string(&self) -> String {
        let mut text = String::with_capacity(4);
        if self.white_king_side() {
            text.push('K');
        }
        if self.white_queen_side() {
            text.push('Q');
        }
        if self.black_king_side() {
            text.push('k');
        }
        if self.black_queen_side() {
            text.push('q');
        }

        if text.is_empty() {
            text.push('-');
        }
        text
    }
}

impl ToString for PieceColor {
    fn to_string(&self) -> String {
        match self {
            PieceColor::White => "w".to_string(),
            PieceColor::Black => "b".to_string(),
        }
    }
}

impl ChessPiece {
    pub const PIECE_TYPE_COUNT: usize = 6;
    pub fn eval_value(&self) -> u32 {
        match self {
            ChessPiece::Pawn => 100,
            ChessPiece::Knight => 300,
            ChessPiece::Bishop => 315,
            ChessPiece::Rook => 500,
            ChessPiece::Queen => 900,
            ChessPiece::King => 1200,
        }
    }
    pub fn is_slider(&self) -> bool {
        *self == ChessPiece::Rook || *self == ChessPiece::Bishop || *self == ChessPiece::Queen
    }
}

impl Default for ChessBoard {
    fn default() -> Self {
        Self {
            white_pieces: Default::default(),
            black_pieces: Default::default(),
            all_white_pieces: Default::default(),
            all_black_pieces: Default::default(),
            piece_board: [None; Square::NUM as usize],
        }
    }
}

impl ChessBoard {
    pub fn place_piece_of_color(
        &mut self,
        piece: ChessPiece,
        col: PieceColor,
        index_to_set: usize,
        zhash: &mut ZHash,
    ) {
        let piece_bitboard = if col == PieceColor::White {
            self.all_white_pieces = self.all_white_pieces.set_bit(index_to_set);
            &mut self.white_pieces[piece as usize]
        } else {
            self.all_black_pieces = self.all_black_pieces.set_bit(index_to_set);
            &mut self.black_pieces[piece as usize]
        };
        self.piece_board[index_to_set] = Some((piece, col));
        *piece_bitboard = piece_bitboard.set_bit(index_to_set);
        zhash.toggle_piece_at_pos(piece, col, index_to_set);
    }

    pub fn get_piece_at_pos(&self, index: usize) -> Option<(ChessPiece, PieceColor)> {
        self.piece_board[index]
    }

    pub fn remove_piece_at_pos(
        &mut self,
        piece: ChessPiece,
        col: PieceColor,
        index: usize,
        zhash: &mut ZHash,
    ) {
        let piece_bitboard = if col == PieceColor::White {
            self.all_white_pieces = self.all_white_pieces.clear_bit(index);
            &mut self.white_pieces[piece as usize]
        } else {
            self.all_black_pieces = self.all_black_pieces.clear_bit(index);
            &mut self.black_pieces[piece as usize]
        };
        self.piece_board[index] = None;
        *piece_bitboard = piece_bitboard.clear_bit(index);
        zhash.toggle_piece_at_pos(piece, col, index);
    }

    // only used for enpassant check reveal
    pub fn remove_any_piece_by_mask(&self, to_be_removed: BitBoard) -> Self {
        let mut new = self.clone();
        for bb in new.white_pieces.iter_mut() {
            *bb = *bb & !to_be_removed;
        }
        for bb in new.black_pieces.iter_mut() {
            *bb = *bb & !to_be_removed;
        }
        new.all_white_pieces = new.all_white_pieces & !to_be_removed;
        new.all_black_pieces = new.all_black_pieces & !to_be_removed;
        new
    }

    #[inline(always)]
    pub fn get_piece_bitboard(&self, piece: ChessPiece, col: PieceColor) -> BitBoard {
        if col == PieceColor::White {
            self.white_pieces[piece as usize]
        } else {
            self.black_pieces[piece as usize]
        }
    }

    #[inline(always)]
    pub fn get_opposing_pieces(&self, col: PieceColor) -> BitBoard {
        if col == PieceColor::White {
            self.all_black_pieces
        } else {
            self.all_white_pieces
        }
    }

    #[inline(always)]
    pub fn get_king_pos(&self, col: PieceColor) -> usize {
        self.get_piece_bitboard(ChessPiece::King, col)
            .into_iter()
            .nth(0)
            .unwrap()
    }

    pub fn from_fen_notation(fen: &str, zhash: &mut ZHash) -> Result<Self, ()> {
        let mut board = Self::default();
        let mut cur_index: usize = 0;

        for chr in fen.chars() {
            if chr.is_digit(10) {
                cur_index += chr.to_digit(10).unwrap() as usize;
                continue;
            }

            if chr == '/' {
                continue;
            }

            let piece_col = if chr.is_uppercase() {
                PieceColor::White
            } else {
                PieceColor::Black
            };

            match &chr.to_lowercase().to_string() as &str {
                "p" => board.place_piece_of_color(ChessPiece::Pawn, piece_col, cur_index, zhash),
                "n" => board.place_piece_of_color(ChessPiece::Knight, piece_col, cur_index, zhash),
                "b" => board.place_piece_of_color(ChessPiece::Bishop, piece_col, cur_index, zhash),
                "r" => board.place_piece_of_color(ChessPiece::Rook, piece_col, cur_index, zhash),
                "q" => board.place_piece_of_color(ChessPiece::Queen, piece_col, cur_index, zhash),
                "k" => board.place_piece_of_color(ChessPiece::King, piece_col, cur_index, zhash),
                _ => return Err(()),
            }
            cur_index += 1;
        }
        Ok(board)
    }

    pub fn empty_squares(&self) -> BitBoard {
        BitBoard(!(self.all_white_pieces.0 | self.all_black_pieces.0))
    }
}

impl ChessBoardState {
    pub fn starting_state() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0").unwrap()
    }

    pub fn piece_to_fen_notation(piece: ChessPiece, color: PieceColor) -> char {
        match (piece, color) {
            (ChessPiece::Pawn, PieceColor::Black) => 'p',
            (ChessPiece::Knight, PieceColor::Black) => 'n',
            (ChessPiece::Bishop, PieceColor::Black) => 'b',
            (ChessPiece::Rook, PieceColor::Black) => 'r',
            (ChessPiece::Queen, PieceColor::Black) => 'q',
            (ChessPiece::King, PieceColor::Black) => 'k',
            (ChessPiece::Pawn, PieceColor::White) => 'P',
            (ChessPiece::Knight, PieceColor::White) => 'N',
            (ChessPiece::Bishop, PieceColor::White) => 'B',
            (ChessPiece::Rook, PieceColor::White) => 'R',
            (ChessPiece::Queen, PieceColor::White) => 'Q',
            (ChessPiece::King, PieceColor::White) => 'K',
        }
    }

    fn update_enpassant_hash(&mut self, current_side: PieceColor, ep_target: Option<u8>) {
        // Only toggle the enpassant target if there are any pieces that attack the target square
        if let Some(ep_target) = ep_target {
            let enpassant_attackers = self.board.pawns_able_to_enpassant(current_side, ep_target);
            if enpassant_attackers.0 != 0 {
                self.zhash.toggle_enpassant(ep_target as usize);
            }
        }
    }

    pub fn from_fen(text: &str) -> Result<Self, ()> {
        let fen_parts: Vec<&str> = text.trim().split(" ").collect();
        if fen_parts.len() != 6 {
            return Err(());
        }

        let mut zhash = ZHash::default();
        let mut board = ChessBoardState {
            board: ChessBoard::from_fen_notation(fen_parts[0], &mut zhash)?,
            side: PieceColor::try_from(fen_parts[1])?,
            castling_rights: CastlingRights::try_from(fen_parts[2])?,
            en_passant_target: Square::from_square_name(fen_parts[3])?,
            half_moves: fen_parts[4].parse::<u8>().map_err(|_| ())?,
            full_moves: fen_parts[5].parse::<u8>().map_err(|_| ())?,
            zhash: zhash,
        };

        if board.side == PieceColor::White {
            board.zhash.toggle_side();
        }

        board
            .zhash
            .swap_castling_rights(&CastlingRights(0), &board.castling_rights);

        board.update_enpassant_hash(board.side, board.en_passant_target);

        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for y in 0..8 {
            let mut no_piece_count = 0;
            for x in 0..8 {
                let pos = x + 8 * y;
                if let Some((piece, color)) = self.board.get_piece_at_pos(pos) {
                    if no_piece_count != 0 {
                        fen.push_str(&no_piece_count.to_string());
                        no_piece_count = 0;
                    }
                    fen.push(Self::piece_to_fen_notation(piece, color));
                } else {
                    no_piece_count += 1;
                }
            }
            if no_piece_count != 0 {
                fen.push_str(&no_piece_count.to_string());
            }
            if y != 7 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push_str(&self.side.to_string());
        fen.push(' ');
        fen.push_str(&self.castling_rights.to_string());
        fen.push(' ');

        fen.push_str(&Square::to_square_name(self.en_passant_target));
        fen.push(' ');
        fen.push_str(&self.half_moves.to_string());
        fen.push(' ');
        fen.push_str(&self.full_moves.to_string());

        fen
    }

    pub fn revoke_castling_rights(
        &mut self,
        src_piece: ChessPiece,
        src_color: PieceColor,
        dst: Option<(ChessPiece, PieceColor)>,
        mv: &Move,
    ) {
        // Revoke Castling Rights, King has moved
        if src_piece == ChessPiece::King {
            if src_color == PieceColor::White {
                self.castling_rights.set_white_king_side(false);
                self.castling_rights.set_white_queen_side(false);
            } else {
                self.castling_rights.set_black_king_side(false);
                self.castling_rights.set_black_queen_side(false);
            }
        }
        let combinations: &[(
            PieceColor,
            fn(&CastlingRights) -> bool,
            fn(&mut CastlingRights, bool),
            u16,
        )] = &[
            (
                PieceColor::White,
                CastlingRights::white_queen_side,
                CastlingRights::set_white_queen_side,
                Square::A1,
            ),
            (
                PieceColor::White,
                CastlingRights::white_king_side,
                CastlingRights::set_white_king_side,
                Square::H1,
            ),
            (
                PieceColor::Black,
                CastlingRights::black_queen_side,
                CastlingRights::set_black_queen_side,
                Square::A8,
            ),
            (
                PieceColor::Black,
                CastlingRights::black_king_side,
                CastlingRights::set_black_king_side,
                Square::H8,
            ),
        ];

        // Revoke Castling Rights, Rook has moved
        if src_piece == ChessPiece::Rook {
            for (pc, getter, setter, square) in combinations {
                if src_color == *pc && getter(&self.castling_rights) && mv.get_src() == *square {
                    setter(&mut self.castling_rights, false);
                }
            }
        }
        // Revoke Castling Rights, Rook was captured
        if mv.is_capture() && !mv.is_en_passant() {
            let (dst_piece, dst_color) = match dst {
                None => panic!("No piece to capture"),
                Some(e) => e,
            };
            if dst_piece == ChessPiece::Rook {
                for (pc, getter, setter, square) in  combinations {
                    if *pc == dst_color && getter(&self.castling_rights) && mv.get_dst() == *square {
                        setter(&mut self.castling_rights, false);
                    }
                }
            }
        }
    }

    pub fn exec_move(&self, mv: Move) -> Self {
        let mut new = *self;

        if let Some(ep_target) = new.en_passant_target {
            new.zhash.toggle_enpassant(ep_target as usize);
        }
        new.en_passant_target = None;

        let (src_piece, src_color) = match self.board.get_piece_at_pos(mv.get_src() as usize) {
            None => panic!("No piece at src pos!"),
            Some(e) => e,
        };

        let dst_piece_col = self.board.get_piece_at_pos(mv.get_dst() as usize);

        assert!(
            src_color == self.side,
            "Moving piece which does not belong to current player!"
        );

        // Move is a capture
        if mv.is_capture() && !mv.is_en_passant() {
            let (dst_piece, dst_color) = dst_piece_col.unwrap();
            assert!(dst_color != src_color, "Can not capture own pieces");
            new.board.remove_piece_at_pos(
                dst_piece,
                dst_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );

            let new_piece = if mv.is_promotion() {
                mv.promotion_target()
            } else {
                src_piece
            };

            new.board.place_piece_of_color(
                new_piece,
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
        } else if mv.is_promotion() && !mv.is_capture() {
            // Promotion, non capture
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );
            new.board.place_piece_of_color(
                mv.promotion_target(),
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
        } else if mv.is_silent() {
            // Move is silent
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );
            new.board.place_piece_of_color(
                src_piece,
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
        } else if mv.is_double_push() {
            // Pawn double push
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );
            new.board.place_piece_of_color(
                src_piece,
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
            let new_ep_target = if src_color == PieceColor::White {
                Some(mv.get_dst() as u8 + 8)
            } else {
                Some(mv.get_dst() as u8 - 8)
            };
            // Side to move is not updated yet, so we need to toggle it
            new.update_enpassant_hash(!new.side, new_ep_target);
            new.en_passant_target = new_ep_target;
        } else if mv.is_en_passant() {
            // En passant
            let dst = if src_color == PieceColor::White {
                mv.get_dst() + 8
            } else {
                mv.get_dst() - 8
            };
            let (dst_piece, dst_color) = match self.board.get_piece_at_pos(dst as usize) {
                None => panic!("No piece to capture"),
                Some(e) => e,
            };

            new.board
                .remove_piece_at_pos(dst_piece, dst_color, dst as usize, &mut new.zhash);
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );
            new.board.place_piece_of_color(
                src_piece,
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
        } else if mv.get_type() == MoveType::CastleKingSide {
            // Castling King Side

            assert!(src_piece == ChessPiece::King);
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );
            new.board.place_piece_of_color(
                src_piece,
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
            if src_color == PieceColor::White {
                new.board.remove_piece_at_pos(
                    ChessPiece::Rook,
                    src_color,
                    Square::H1 as usize,
                    &mut new.zhash,
                );
                new.board.place_piece_of_color(
                    ChessPiece::Rook,
                    src_color,
                    Square::F1 as usize,
                    &mut new.zhash,
                );
            } else {
                new.board.remove_piece_at_pos(
                    ChessPiece::Rook,
                    src_color,
                    Square::H8 as usize,
                    &mut new.zhash,
                );
                new.board.place_piece_of_color(
                    ChessPiece::Rook,
                    src_color,
                    Square::F8 as usize,
                    &mut new.zhash,
                );
            }
        } else if mv.get_type() == MoveType::CastleQueenSide {
            // Castling Queen Side
            assert!(src_piece == ChessPiece::King);
            new.board.remove_piece_at_pos(
                src_piece,
                src_color,
                mv.get_src() as usize,
                &mut new.zhash,
            );
            new.board.place_piece_of_color(
                src_piece,
                src_color,
                mv.get_dst() as usize,
                &mut new.zhash,
            );
            if src_color == PieceColor::White {
                new.board.remove_piece_at_pos(
                    ChessPiece::Rook,
                    src_color,
                    Square::A1 as usize,
                    &mut new.zhash,
                );
                new.board.place_piece_of_color(
                    ChessPiece::Rook,
                    src_color,
                    Square::D1 as usize,
                    &mut new.zhash,
                );
            } else {
                new.board.remove_piece_at_pos(
                    ChessPiece::Rook,
                    src_color,
                    Square::A8 as usize,
                    &mut new.zhash,
                );
                new.board.place_piece_of_color(
                    ChessPiece::Rook,
                    src_color,
                    Square::D8 as usize,
                    &mut new.zhash,
                );
            }
        }

        new.revoke_castling_rights(src_piece, src_color, dst_piece_col, &mv);
        new.zhash
            .swap_castling_rights(&self.castling_rights, &new.castling_rights);

        if mv.is_capture() || src_piece == ChessPiece::Pawn {
            new.half_moves = 0;
        } else {
            new.half_moves += 1;
        }
        new.full_moves += 1;
        new.side = !new.side;
        new.zhash.toggle_side();
        new
    }

    pub fn is_in_check(&self) -> bool {
        let attacked_squares = self.board.squares_attacked_by_side(!self.side, false);
        attacked_squares.get_bit(self.board.get_king_pos(self.side))
    }

    pub fn total_piece_count(&self) -> u32 {
        self.board.all_black_pieces.0.count_ones() + self.board.all_white_pieces.0.count_ones()
    }
}

#[cfg(test)]
mod board_tests {

    use crate::bb;
    use crate::chess::board::BitBoard;
    use crate::chess::board::{CastlingRights, ChessBoard, ChessBoardState, PieceColor};
    use crate::chess::chess_move::{Move, MoveType};
    use crate::chess::square::Square;
    use crate::chess::zobrist_hash::ZHash;

    fn check_board_equality(state: &ChessBoardState, state_expected: &ChessBoardState) {
        for piece_type in 0..=5 {
            let board_white = state.board.white_pieces[piece_type];
            let exp_white = state_expected.board.white_pieces[piece_type];

            assert_eq!(
                board_white.0, exp_white.0,
                "Non matching for white piece_type {:?}",
                piece_type
            );

            let board_black = state.board.black_pieces[piece_type];
            let exp_black = state_expected.board.black_pieces[piece_type];

            assert_eq!(
                board_black.0, exp_black.0,
                "Non matching for black piece_type {:?}",
                piece_type
            );
        }
        assert_eq!(state.side, state_expected.side);
        assert_eq!(state.castling_rights, state_expected.castling_rights);
        assert_eq!(state.en_passant_target, state_expected.en_passant_target);
        assert_eq!(state.half_moves, state_expected.half_moves);
        assert_eq!(state.full_moves, state_expected.full_moves);
        assert_eq!(
            state.board.all_black_pieces,
            state_expected.board.all_black_pieces
        );
        assert_eq!(
            state.board.all_white_pieces,
            state_expected.board.all_white_pieces
        );
    }

    #[test]
    fn board_from_fen_simple() {
        let board =
            ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
        assert!(board.is_ok());

        let expected = ChessBoardState {
            board: ChessBoard {
                white_pieces: [
                    bb!(71776119061217280),
                    bb!(4755801206503243776),
                    bb!(2594073385365405696),
                    bb!(9295429630892703744),
                    bb!(576460752303423488),
                    bb!(1152921504606846976),
                ],
                black_pieces: [bb!(65280), bb!(66), bb!(36), bb!(129), bb!(8), bb!(16)],
                all_white_pieces: bb!(18446462598732840960),
                all_black_pieces: bb!(65535),
                piece_board: [None; Square::NUM as usize],
            },
            side: PieceColor::White,
            castling_rights: CastlingRights::all(),
            en_passant_target: None,
            half_moves: 0,
            full_moves: 0,
            zhash: ZHash::default(),
        };

        check_board_equality(&board.unwrap(), &expected);
    }

    #[test]
    fn board_from_fen_complex() {
        let board = ChessBoardState::from_fen(
            "2r2k1r/1pqn1p2/5P2/1p5p/1b1PQ3/PP5P/1BP3P1/2KRR3 b - - 0 21",
        );

        let expected = ChessBoardState {
            board: ChessBoard {
                white_pieces: [
                    bb!(19284368801398784),
                    bb!(0),
                    bb!(562949953421312),
                    bb!(1729382256910270464),
                    bb!(68719476736),
                    bb!(288230376151711744),
                ],
                black_pieces: [
                    bb!(2181046784),
                    bb!(2048),
                    bb!(8589934592),
                    bb!(132),
                    bb!(1024),
                    bb!(32),
                ],
                all_white_pieces: bb!(2037460020536279040),
                all_black_pieces: bb!(10770984612),
                piece_board: [None; Square::NUM as usize],
            },
            side: PieceColor::Black,
            castling_rights: CastlingRights::none(),
            en_passant_target: None,
            half_moves: 0,
            full_moves: 21,
            zhash: ZHash::default(),
        };

        check_board_equality(&board.unwrap(), &expected);
    }

    #[test]
    fn test_empty_squares() {
        let board_state =
            ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
        assert!(board_state.is_ok());
        let board_state = board_state.unwrap();
        assert_eq!(board_state.board.empty_squares(), BitBoard(281474976645120));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let board_strs = [
            "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2",
            "2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4",
            "r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2",
            "8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3",
            "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2",
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
            "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9",
            "4k3/4r3/4Q3/8/8/8/8/3K4 b - - 5 4",
            "8/8/8/3Qrk2/8/8/8/3K4 b - - 0 1",
            "6k1/5p2/8/3Q4/8/8/8/3K4 b - - 0 1",
            "6k1/5p2/8/3B4/8/8/8/3K4 b - - 0 1",
            "6k1/5n2/8/3B4/8/8/8/3K4 b - - 0 1",
            "6k1/5q2/8/3B4/8/8/8/3K4 b - - 0 1",
            "8/5k2/8/3B4/5R2/8/8/3K4 b - - 0 1",
            "8/5k2/4q3/3B4/5R2/8/8/3K4 b - - 0 1",
        ];

        for board in &board_strs {
            let board_state = ChessBoardState::from_fen(board);
            assert!(board_state.is_ok());
            let board_state = board_state.unwrap();

            let fen = board_state.to_fen();
            assert!(
                board.eq(&fen),
                "FENs are not equal, expected: {}, produced: {}",
                board,
                fen
            );
        }
    }

    #[test]
    fn test_castling_right_update() {
        let mut board_state =
            ChessBoardState::from_fen("r3k2r/8/8/R6R/r6r/8/8/R3K2R w KQkq - 0 12").unwrap();

        // Castle King Side White
        let castled_white_king_side =
            board_state.exec_move(Move::new(Square::E1, Square::G1, MoveType::CastleKingSide));
        assert_eq!(
            castled_white_king_side.castling_rights.white_king_side(), false,
            "Castle Kingside, all castling should be revoked"
        );
        assert_eq!(
            castled_white_king_side.castling_rights.white_queen_side(), false,
            "Castle Kingside, all castling should be revoked"
        );

        // Castle Queen Side White
        let castled_white_queen_side =
            board_state.exec_move(Move::new(Square::E1, Square::C1, MoveType::CastleQueenSide));
        assert_eq!(
            castled_white_queen_side.castling_rights.white_king_side(), false,
            "Castle Queenside, all castling should be revoked"
        );
        assert_eq!(
            castled_white_queen_side.castling_rights.white_queen_side(), false,
            "Castle Queenside, all castling should be revoked"
        );

        // Make black the current player
        board_state.side = PieceColor::Black;

        // Capture Queen Side Rook
        let capture_white_q_rook =
            board_state.exec_move(Move::new(Square::A4, Square::A1, MoveType::Capture));
        assert_eq!(
            capture_white_q_rook.castling_rights.white_queen_side(), false,
            "Captured queen side rook"
        );

        // Capture King Side Rook
        let capture_white_k_rook =
            board_state.exec_move(Move::new(Square::H4, Square::H1, MoveType::Capture));
        assert_eq!(
            capture_white_k_rook.castling_rights.white_king_side(), false,
            "Captured king side rook"
        );
    }
}
