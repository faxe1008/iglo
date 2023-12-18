use super::{
    bitboard::BitBoard,
    board::{ChessBoard, ChessBoardState, ChessPiece, PieceColor},
    chess_move::{Move, MoveType, PROMOTION_CAPTURE_TARGETS, PROMOTION_TARGETS},
};

impl ChessBoard {
    #[inline(always)]
    fn pawns_able_to_push(&self, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            self.empty_squares().sSo() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            self.empty_squares().sNo() & self.black_pieces[ChessPiece::Pawn as usize]
        }
    }

    #[inline(always)]
    fn pawns_able_to_double_push(&self, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            let empty_rank_3 =
                (self.empty_squares() & BitBoard::RANK_4).sSo() & self.empty_squares();
            empty_rank_3.sSo() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            let empty_rank_6 =
                (self.empty_squares() & BitBoard::RANK_5).sNo() & self.empty_squares();
            empty_rank_6.sNo() & self.black_pieces[ChessPiece::Pawn as usize]
        }
    }

    #[inline(always)]
    fn pawns_able_to_attack_east(&self, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            self.all_black_pieces.sSoWe() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            self.all_white_pieces.sNoWe() & self.black_pieces[ChessPiece::Pawn as usize]
        }
    }

    #[inline(always)]
    fn pawns_able_to_attack_west(&self, color: PieceColor) -> BitBoard {
        if color == PieceColor::White {
            self.all_black_pieces.sSoEa() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            self.all_white_pieces.sNoEa() & self.black_pieces[ChessPiece::Pawn as usize]
        }
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
        .pawns_able_to_attack_east(color)
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
        .pawns_able_to_attack_west(color)
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

    moves
}


const KNIGHT_MOVE_LOOKUP : [BitBoard; 64] = unsafe { std::mem::transmute(*include_bytes!("lookup_gens/knight_lookup.bin")) };
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
    }else{
        board_state.board.all_white_pieces
    };
    
    for knight_pos  in side_knight_board {
        let attack_map = KNIGHT_MOVE_LOOKUP[knight_pos];

        for silent_jump_target in attack_map & empty_squares {
            moves.push(Move::new(knight_pos as u16, silent_jump_target as u16, MoveType::Silent));
        }

        for capture_jump in attack_map & opposite_board {
            moves.push(Move::new(knight_pos as u16, capture_jump as u16, MoveType::Capture));
        }
    }

    moves
}


pub fn generate_pseudo_legal_moves(board_state: &ChessBoardState, color: PieceColor) -> Vec<Move> {
    let mut vec = Vec::with_capacity(32);

    vec.append(&mut generate_knight_moves(board_state, color));
    vec.append(&mut generate_pawn_moves(board_state, color));

    vec
}


#[cfg(test)]
mod move_gen_tests {
    use crate::engine::{
        board::{ChessBoardState, PieceColor},
        chess_move::{Move, MoveType},
        move_generator::{generate_pawn_moves, KNIGHT_MOVE_LOOKUP, generate_knight_moves},
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
    fn knight_attacks() 
    {
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
