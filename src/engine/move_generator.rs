use super::{
    bitboard::{BitBoard, MagicEntry},
    board::{ChessBoard, ChessBoardState, ChessPiece, PieceColor},
    chess_move::{Move, MoveType, PROMOTION_CAPTURE_TARGETS, PROMOTION_TARGETS},
};

impl ChessBoard {
    #[inline(always)]
    fn pawns_able_to_push(&self, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            self.empty_squares().s_so() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            self.empty_squares().s_no() & self.black_pieces[ChessPiece::Pawn as usize]
        }
    }

    #[inline(always)]
    fn pawns_able_to_double_push(&self, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            let empty_rank_3 =
                (self.empty_squares() & BitBoard::RANK_4).s_so() & self.empty_squares();
            empty_rank_3.s_so() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            let empty_rank_6 =
                (self.empty_squares() & BitBoard::RANK_5).s_no() & self.empty_squares();
            empty_rank_6.s_no() & self.black_pieces[ChessPiece::Pawn as usize]
        }
    }

    #[inline(always)]
    fn pawns_able_to_attack_east(&self, pawn_board: BitBoard, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            self.all_black_pieces.s_so_we() & pawn_board
        } else {
            self.all_white_pieces.s_no_we() & pawn_board
        }
    }

    #[inline(always)]
    fn pawns_able_to_attack_west(&self, pawn_board: BitBoard, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            self.all_black_pieces.s_so_ea() & pawn_board
        } else {
            self.all_white_pieces.s_no_ea() & pawn_board
        }
    }

    #[inline(always)]
    fn pawns_able_to_enpassant(&self, color: PieceColor, en_passant_target: u8) -> BitBoard {
        let target_bb = BitBoard::EMPTY.set_bit(en_passant_target as usize);
        if color == PieceColor::White {
            (target_bb.s_so_ea() | target_bb.s_so_we())
                & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            (target_bb.s_no_ea() | target_bb.s_no_we())
                & self.black_pieces[ChessPiece::Pawn as usize]
        }
    }

    pub fn squares_attacked_by_side(&self, color: PieceColor) -> BitBoard {
        let mut attacked_map = BitBoard::EMPTY;

        let side_pieces = if color == PieceColor::White {
            attacked_map = attacked_map
                | self.white_pieces[ChessPiece::Pawn as usize].s_no_we()
                | self.white_pieces[ChessPiece::Pawn as usize].s_no_ea();
            &self.white_pieces
        } else {
            attacked_map = attacked_map
                | self.black_pieces[ChessPiece::Pawn as usize].s_so_we()
                | self.black_pieces[ChessPiece::Pawn as usize].s_so_ea();
            &self.black_pieces
        };

        let blockers = self.all_black_pieces | self.all_white_pieces;

        for knight_pos in side_pieces[ChessPiece::Knight as usize] {
            attacked_map = attacked_map | KNIGHT_MOVE_LOOKUP[knight_pos];
        }
        for bishop in side_pieces[ChessPiece::Bishop as usize] {
            let magic = BISHOP_MAGICS[bishop].magic_index(blockers);
            attacked_map = attacked_map | (&BISHOP_MOVES[bishop])[magic];
        }
        for rook in side_pieces[ChessPiece::Rook as usize] {
            let magic = ROOK_MAGICS[rook].magic_index(blockers);
            attacked_map = attacked_map | (&ROOK_MOVES[rook])[magic];
        }
        for queen in side_pieces[ChessPiece::Queen as usize] {
            let r_magic = ROOK_MAGICS[queen].magic_index(blockers);
            let b_magic = BISHOP_MAGICS[queen].magic_index(blockers);
            attacked_map = attacked_map | (&ROOK_MOVES[queen])[r_magic];
            attacked_map = attacked_map | (&BISHOP_MOVES[queen])[b_magic];
        }

        attacked_map
    }

    #[inline(always)]
    pub fn king_attackers(&self, color: PieceColor) -> BitBoard {
        let mut attacker_map = BitBoard::EMPTY;

        let king_pos = if color == PieceColor::White {
            self.white_pieces[ChessPiece::King as usize]
        } else {
            self.black_pieces[ChessPiece::King as usize]
        }
        .into_iter()
        .nth(0)
        .unwrap();

        let opposing_pieces = if color == PieceColor::White {
            &self.black_pieces
        } else {
            &self.white_pieces
        };

        let blockers = self.all_black_pieces | self.all_white_pieces;

        // Check for knights
        attacker_map = attacker_map
            | (KNIGHT_MOVE_LOOKUP[king_pos] & opposing_pieces[ChessPiece::Knight as usize]);

        // Check for bishops and queens attack as bishops
        let bishop_attacks =
            (&BISHOP_MOVES[king_pos])[BISHOP_MAGICS[king_pos].magic_index(blockers)];
        attacker_map = attacker_map
            | (bishop_attacks
                & (opposing_pieces[ChessPiece::Bishop as usize]
                    | opposing_pieces[ChessPiece::Queen as usize]));

        // Check for rooks and queens attack as rooks
        let rook_attacks = (&ROOK_MOVES[king_pos])[ROOK_MAGICS[king_pos].magic_index(blockers)];
        attacker_map = attacker_map
            | (rook_attacks
                & (opposing_pieces[ChessPiece::Rook as usize]
                    | opposing_pieces[ChessPiece::Queen as usize]));

        // Check for pawns
        let king_board = BitBoard(1 << king_pos);
        let attackers = if color == PieceColor::White {
            (king_board.s_no_we() | king_board.s_no_ea())
                & opposing_pieces[ChessPiece::Pawn as usize]
        } else {
            (king_board.s_so_we() | king_board.s_so_ea())
                & opposing_pieces[ChessPiece::Pawn as usize]
        };
        attacker_map = attacker_map | attackers;

        attacker_map
    }
}

fn generate_pawn_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::with_capacity(16);

    let side_pawn_board = if color == PieceColor::White {
        board_state.board.white_pieces[ChessPiece::Pawn as usize]
    } else {
        board_state.board.black_pieces[ChessPiece::Pawn as usize]
    };

    if side_pawn_board == BitBoard::EMPTY {
        return moves;
    }

    let push_dir: i32 = if color == PieceColor::White { -1 } else { 1 };
    let promotion_range = if color == PieceColor::White {
        0..8
    } else {
        56..64
    };

    for pushable_pawn in board_state.board.pawns_able_to_push(color).into_iter() {
        let target = pushable_pawn as i32 + 8 * push_dir;
        if target < 0 || target > 63 {
            continue;
        }

        if promotion_range.contains(&target) {
            // Promote Pawn
            for p in PROMOTION_TARGETS {
                moves.push(Move::new(pushable_pawn as u16, target as u16, p));
            }
        } else {
            moves.push(Move::new(
                pushable_pawn as u16,
                target as u16,
                MoveType::Silent,
            ));
        }
    }

    for double_pushable_pawn in board_state
        .board
        .pawns_able_to_double_push(color)
        .into_iter()
    {
        let target = double_pushable_pawn as i32 + 16 * push_dir;
        if target >= 0 && target <= 63 {
            moves.push(Move::new(
                double_pushable_pawn as u16,
                target as u16,
                MoveType::DoublePush,
            ));
        }
    }

    let east_attack_dir = if color == PieceColor::White { -7 } else { 9 };
    for east_attacking_pawn in board_state
        .board
        .pawns_able_to_attack_east(side_pawn_board, color)
        .into_iter()
    {
        let target = east_attacking_pawn as i32 + east_attack_dir;
        if target < 0 || target > 63 {
            continue;
        }

        if promotion_range.contains(&target) {
            for p in PROMOTION_CAPTURE_TARGETS {
                moves.push(Move::new(east_attacking_pawn as u16, target as u16, p));
            }
        } else {
            moves.push(Move::new(
                east_attacking_pawn as u16,
                target as u16,
                MoveType::Capture,
            ));
        }
    }

    let west_attack_dir = if color == PieceColor::White { -9 } else { 7 };
    for west_atacking_pawn in board_state
        .board
        .pawns_able_to_attack_west(side_pawn_board, color)
        .into_iter()
    {
        let target = west_atacking_pawn as i32 + west_attack_dir;
        if target < 0 || target > 63 {
            continue;
        }

        if promotion_range.contains(&target) {
            for p in PROMOTION_CAPTURE_TARGETS {
                moves.push(Move::new(west_atacking_pawn as u16, target as u16, p));
            }
        } else {
            moves.push(Move::new(
                west_atacking_pawn as u16,
                target as u16,
                MoveType::Capture,
            ));
        }
    }

    if let Some(en_passant_target) = board_state.en_passant_target {
        for en_passant_pawns in board_state
            .board
            .pawns_able_to_enpassant(color, en_passant_target)
        {
            moves.push(Move::new(
                en_passant_pawns as u16,
                en_passant_target as u16,
                MoveType::EnPassant,
            ));
        }
    }

    moves
}

const KNIGHT_MOVE_LOOKUP: [BitBoard; 64] =
    unsafe { std::mem::transmute(*include_bytes!("lookup_gens/knight_lookup.bin")) };
fn generate_knight_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::with_capacity(16);
    let side_knight_board = if color == PieceColor::White {
        board_state.board.white_pieces[ChessPiece::Knight as usize]
    } else {
        board_state.board.black_pieces[ChessPiece::Knight as usize]
    };

    if side_knight_board == BitBoard::EMPTY {
        return moves;
    }

    let empty_squares = board_state.board.empty_squares();
    let opposite_board = if color == PieceColor::White {
        board_state.board.all_black_pieces
    } else {
        board_state.board.all_white_pieces
    };

    for knight_pos in side_knight_board {
        let attack_map = KNIGHT_MOVE_LOOKUP[knight_pos];

        for silent_jump_target in attack_map & empty_squares {
            moves.push(Move::new(
                knight_pos as u16,
                silent_jump_target as u16,
                MoveType::Silent,
            ));
        }

        for capture_jump in attack_map & opposite_board {
            moves.push(Move::new(
                knight_pos as u16,
                capture_jump as u16,
                MoveType::Capture,
            ));
        }
    }

    moves
}

const KING_MOVE_LOOKUP: [BitBoard; 64] =
    unsafe { std::mem::transmute(*include_bytes!("lookup_gens/king_lookup.bin")) };
fn generate_king_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::with_capacity(16);
    let side_king_board = if color == PieceColor::White {
        board_state.board.white_pieces[ChessPiece::King as usize]
    } else {
        board_state.board.black_pieces[ChessPiece::King as usize]
    };

    if side_king_board == BitBoard::EMPTY {
        return moves;
    }

    let empty_squares = board_state.board.empty_squares();
    let opposite_board = if color == PieceColor::White {
        board_state.board.all_black_pieces
    } else {
        board_state.board.all_white_pieces
    };

    let attacked_by_enemy = board_state.board.squares_attacked_by_side(!color);

    for king_pos in side_king_board {
        let attack_map = KING_MOVE_LOOKUP[king_pos] & !attacked_by_enemy;

        for silent_move_target in attack_map & empty_squares {
            moves.push(Move::new(
                king_pos as u16,
                silent_move_target as u16,
                MoveType::Silent,
            ));
        }
        for capture_move in attack_map & opposite_board {
            moves.push(Move::new(
                king_pos as u16,
                capture_move as u16,
                MoveType::Capture,
            ));
        }
    }

    moves
}

const ROOK_MAGICS: [MagicEntry; 64] =
    unsafe { std::mem::transmute(*include_bytes!("lookup_gens/rook_magics.bin")) };
const ROOK_MOVES: [[BitBoard; 4096]; 64] =
    unsafe { std::mem::transmute(*include_bytes!("lookup_gens/rook_moves.bin")) };

fn generate_rook_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::with_capacity(16);
    let side_rook_board = if color == PieceColor::White {
        board_state.board.white_pieces[ChessPiece::Rook as usize]
    } else {
        board_state.board.black_pieces[ChessPiece::Rook as usize]
    };

    if side_rook_board == BitBoard::EMPTY {
        return moves;
    }

    let opposing_pieces = if color == PieceColor::White {
        board_state.board.all_black_pieces
    } else {
        board_state.board.all_white_pieces
    };

    let blockers = board_state.board.all_white_pieces | board_state.board.all_black_pieces;
    let empty_squares = board_state.board.empty_squares();

    for rook_pos in side_rook_board {
        let magic_index = &ROOK_MAGICS[rook_pos].magic_index(blockers);
        let move_bitboard = ROOK_MOVES[rook_pos][*magic_index];
        for mv_dst in move_bitboard {
            if opposing_pieces.get_bit(mv_dst) {
                moves.push(Move::new(rook_pos as u16, mv_dst as u16, MoveType::Capture));
            } else if empty_squares.get_bit(mv_dst) {
                moves.push(Move::new(rook_pos as u16, mv_dst as u16, MoveType::Silent));
            }
        }
    }

    moves
}

const BISHOP_MAGICS: [MagicEntry; 64] =
    unsafe { std::mem::transmute(*include_bytes!("lookup_gens/bishop_magics.bin")) };
const BISHOP_MOVES: [[BitBoard; 4096]; 64] =
    unsafe { std::mem::transmute(*include_bytes!("lookup_gens/bishop_moves.bin")) };

fn generate_bishop_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::with_capacity(16);
    let side_bishop_board = if color == PieceColor::White {
        board_state.board.white_pieces[ChessPiece::Bishop as usize]
    } else {
        board_state.board.black_pieces[ChessPiece::Bishop as usize]
    };

    if side_bishop_board == BitBoard::EMPTY {
        return moves;
    }

    let opposing_pieces = if color == PieceColor::White {
        board_state.board.all_black_pieces
    } else {
        board_state.board.all_white_pieces
    };

    let blockers = board_state.board.all_white_pieces | board_state.board.all_black_pieces;
    let empty_squares = board_state.board.empty_squares();

    for bishop_pos in side_bishop_board {
        let magic_index = &BISHOP_MAGICS[bishop_pos].magic_index(blockers);
        let move_bitboard = BISHOP_MOVES[bishop_pos][*magic_index];
        for mv_dst in move_bitboard {
            if opposing_pieces.get_bit(mv_dst) {
                moves.push(Move::new(
                    bishop_pos as u16,
                    mv_dst as u16,
                    MoveType::Capture,
                ));
            } else if empty_squares.get_bit(mv_dst) {
                moves.push(Move::new(
                    bishop_pos as u16,
                    mv_dst as u16,
                    MoveType::Silent,
                ));
            }
        }
    }

    moves
}

fn generate_queen_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut moves = Vec::with_capacity(16);

    let side_queen_board = if color == PieceColor::White {
        board_state.board.white_pieces[ChessPiece::Queen as usize]
    } else {
        board_state.board.black_pieces[ChessPiece::Queen as usize]
    };

    if side_queen_board == BitBoard::EMPTY {
        return moves;
    }

    let opposing_pieces = if color == PieceColor::White {
        board_state.board.all_black_pieces
    } else {
        board_state.board.all_white_pieces
    };
    let blockers = board_state.board.all_white_pieces | board_state.board.all_black_pieces;
    let empty_squares = board_state.board.empty_squares();

    for queen_pos in side_queen_board {
        let bishop_magic = &BISHOP_MAGICS[queen_pos].magic_index(blockers);
        let bishop_move_bitboard = BISHOP_MOVES[queen_pos][*bishop_magic];

        let rook_magic = &ROOK_MAGICS[queen_pos].magic_index(blockers);
        let rook_move_bitboard = ROOK_MOVES[queen_pos][*rook_magic];

        let queen_move_bitboard = bishop_move_bitboard | rook_move_bitboard;

        for queen_dst in queen_move_bitboard {
            if opposing_pieces.get_bit(queen_dst) {
                moves.push(Move::new(
                    queen_pos as u16,
                    queen_dst as u16,
                    MoveType::Capture,
                ));
            } else if empty_squares.get_bit(queen_dst) {
                moves.push(Move::new(
                    queen_pos as u16,
                    queen_dst as u16,
                    MoveType::Silent,
                ));
            }
        }
    }

    moves
}

pub fn generate_pseudo_legal_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut vec = Vec::with_capacity(32);
    vec.append(&mut generate_knight_moves(board_state, color));
    vec.append(&mut generate_pawn_moves(board_state, color));
    vec.append(&mut generate_king_moves(board_state, color));
    vec.append(&mut generate_rook_moves(board_state, color));
    vec.append(&mut generate_bishop_moves(board_state, color));
    vec.append(&mut generate_queen_moves(board_state, color));
    vec
}

#[cfg(test)]
mod move_gen_tests {
    use crate::engine::{
        board::{ChessBoardState, PieceColor},
        chess_move::{Move, MoveType},
        move_generator::{generate_knight_moves, generate_pawn_moves, KNIGHT_MOVE_LOOKUP},
        square::Square,
    };

    fn compare_moves(generated: &[Move], expected: &[Move]) {
        //assert_eq!(generated.len(), expected.len());
        for gen_move in generated {
            //println!("GEN: {:?}", &gen_move);
            assert!(expected.contains(dbg!(&gen_move)));
        }
    }

    #[test]
    fn pawns_moves_from_fen_simple() {
        dbg!(KNIGHT_MOVE_LOOKUP);
        let board_state = ChessBoardState::starting_state();

        let expected_moves_white = [
            Move::new(Square::H2, Square::H3, MoveType::Silent),
            Move::new(Square::G2, Square::G3, MoveType::Silent),
            Move::new(Square::F2, Square::F3, MoveType::Silent),
            Move::new(Square::E2, Square::E3, MoveType::Silent),
            Move::new(Square::D2, Square::D3, MoveType::Silent),
            Move::new(Square::C2, Square::C3, MoveType::Silent),
            Move::new(Square::B2, Square::B3, MoveType::Silent),
            Move::new(Square::A2, Square::A3, MoveType::Silent),
            Move::new(Square::H2, Square::H4, MoveType::DoublePush),
            Move::new(Square::G2, Square::G4, MoveType::DoublePush),
            Move::new(Square::F2, Square::F4, MoveType::DoublePush),
            Move::new(Square::E2, Square::E4, MoveType::DoublePush),
            Move::new(Square::D2, Square::D4, MoveType::DoublePush),
            Move::new(Square::C2, Square::C4, MoveType::DoublePush),
            Move::new(Square::B2, Square::B4, MoveType::DoublePush),
            Move::new(Square::A2, Square::A4, MoveType::DoublePush),
        ];

        let white_pawn_moves = generate_pawn_moves(&board_state, PieceColor::White);
        compare_moves(&white_pawn_moves, &expected_moves_white);

        let expected_moves_black = [
            Move::new(Square::H7, Square::H6, MoveType::Silent),
            Move::new(Square::G7, Square::G6, MoveType::Silent),
            Move::new(Square::F7, Square::F6, MoveType::Silent),
            Move::new(Square::E7, Square::E6, MoveType::Silent),
            Move::new(Square::D7, Square::D6, MoveType::Silent),
            Move::new(Square::C7, Square::C6, MoveType::Silent),
            Move::new(Square::B7, Square::B6, MoveType::Silent),
            Move::new(Square::A7, Square::A6, MoveType::Silent),
            Move::new(Square::H7, Square::H5, MoveType::DoublePush),
            Move::new(Square::G7, Square::G5, MoveType::DoublePush),
            Move::new(Square::F7, Square::F5, MoveType::DoublePush),
            Move::new(Square::E7, Square::E5, MoveType::DoublePush),
            Move::new(Square::D7, Square::D5, MoveType::DoublePush),
            Move::new(Square::C7, Square::C5, MoveType::DoublePush),
            Move::new(Square::B7, Square::B5, MoveType::DoublePush),
            Move::new(Square::A7, Square::A5, MoveType::DoublePush),
        ];
        let black_pawn_moves = generate_pawn_moves(&board_state, PieceColor::Black);
        compare_moves(&black_pawn_moves, &expected_moves_black);
    }

    #[test]
    fn pawns_attacks() {
        let board_state =
            ChessBoardState::from_fen("k6p/6P1/2r5/p1qP4/1P3p2/5P2/P2p4/7K w QKqk - 0 0");
        assert!(board_state.is_ok());
        let board_state = board_state.unwrap();

        let white_pawn_moves = generate_pawn_moves(&board_state, PieceColor::White);
        let expected_moves_white = [
            Move::new(Square::A2, Square::A3, MoveType::Silent),
            Move::new(Square::A2, Square::A4, MoveType::DoublePush),
            Move::new(Square::B4, Square::C5, MoveType::Capture),
            Move::new(Square::B4, Square::A5, MoveType::Capture),
            Move::new(Square::B4, Square::B5, MoveType::Silent),
            Move::new(Square::D5, Square::C6, MoveType::Capture),
            Move::new(Square::D5, Square::D6, MoveType::Silent),
            Move::new(Square::G7, Square::G8, MoveType::KnightPromotion),
            Move::new(Square::G7, Square::G8, MoveType::BishopPromotion),
            Move::new(Square::G7, Square::G8, MoveType::RookPromotion),
            Move::new(Square::G7, Square::G8, MoveType::QueenPromotion),
            Move::new(Square::G7, Square::H8, MoveType::KnightCapPromotion),
            Move::new(Square::G7, Square::H8, MoveType::BishopCapPromotion),
            Move::new(Square::G7, Square::H8, MoveType::RookCapPromotion),
            Move::new(Square::G7, Square::H8, MoveType::QueenCapPromotion),
        ];
        compare_moves(&white_pawn_moves, &expected_moves_white);

        let black_pawn_moves = generate_pawn_moves(&board_state, PieceColor::Black);
        let expected_black_moves = [
            Move::new(Square::A5, Square::A4, MoveType::Silent),
            Move::new(Square::A5, Square::B4, MoveType::Capture),
            Move::new(Square::D2, Square::D1, MoveType::KnightPromotion),
            Move::new(Square::D2, Square::D1, MoveType::BishopPromotion),
            Move::new(Square::D2, Square::D1, MoveType::RookPromotion),
            Move::new(Square::D2, Square::D1, MoveType::QueenPromotion),
            Move::new(Square::H8, Square::G7, MoveType::Capture),
            Move::new(Square::H8, Square::H7, MoveType::Silent),
        ];
        compare_moves(&black_pawn_moves, &expected_black_moves);
    }

    #[test]
    fn knight_attacks() {
        let board_state = ChessBoardState::from_fen("3R4/5n1k/7N/R3B3/3q4/1N6/K7/2b5 w QKqk - 0 0");
        assert!(board_state.is_ok());
        let board_state = board_state.unwrap();

        let white_knight_moves = generate_knight_moves(&board_state, PieceColor::White);
        let expected_white_knight_moves = [
            Move::new(Square::B3, Square::C5, MoveType::Silent),
            Move::new(Square::B3, Square::D4, MoveType::Capture),
            Move::new(Square::B3, Square::D2, MoveType::Silent),
            Move::new(Square::B3, Square::C1, MoveType::Capture),
            Move::new(Square::B3, Square::A1, MoveType::Silent),
            Move::new(Square::H6, Square::G8, MoveType::Silent),
            Move::new(Square::H6, Square::G4, MoveType::Silent),
            Move::new(Square::H6, Square::F5, MoveType::Silent),
            Move::new(Square::H6, Square::F7, MoveType::Capture),
        ];
        compare_moves(&white_knight_moves, &expected_white_knight_moves);

        let black_knight_moves = generate_knight_moves(&board_state, PieceColor::Black);
        let expected_black_moves = [
            Move::new(Square::F7, Square::H8, MoveType::Silent),
            Move::new(Square::F7, Square::H6, MoveType::Capture),
            Move::new(Square::F7, Square::G5, MoveType::Silent),
            Move::new(Square::F7, Square::E5, MoveType::Capture),
            Move::new(Square::F7, Square::D6, MoveType::Silent),
            Move::new(Square::F7, Square::D8, MoveType::Capture),
        ];
        compare_moves(&black_knight_moves, &expected_black_moves);
    }
}
