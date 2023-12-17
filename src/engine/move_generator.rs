use super::{
    bitboard::BitBoard,
    board::{ChessBoard, ChessBoardState, ChessPiece, PieceColor},
    chess_move::{Move, MoveType},
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
        board_state.board.white_pieces
    } else {
        board_state.board.black_pieces
    }[ChessPiece::Pawn as usize];

    if side_pawn_board == BitBoard::EMPTY {
        return moves;
    }

    let push_dir: i32 = if color == PieceColor::White { -1 } else { 1 };

    for pushable_pawn in board_state.board.pawns_able_to_push(color).into_iter() {
        let target = pushable_pawn as i32 + 8 * push_dir;
        if target >= 0 && target <= 63 {
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
        if target >= 0 && target <= 63 {
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
        if target >= 0 && target <= 63 {
            moves.push(Move::new(
                west_atacking_pawn as u16,
                target as u16,
                MoveType::Capture,
            ));
        }
    }

    moves
}

#[cfg(test)]
mod move_gen_tests {
    use crate::engine::{
        board::{ChessBoardState, PieceColor},
        chess_move::{Move, MoveType},
        move_generator::generate_pawn_moves,
    };

    fn compare_moves(generated: &[Move], expected: &[Move]) {
        assert_eq!(generated.len(), expected.len());
        for expected_move in expected {
            assert!(generated.contains(&expected_move));
        }
    }

    #[test]
    fn pawns_moves_from_fen_simple() {
        let board_state = ChessBoardState::starting_state();

        let expected_moves_white = [
            Move::new(55, 47, MoveType::Silent),
            Move::new(54, 46, MoveType::Silent),
            Move::new(53, 45, MoveType::Silent),
            Move::new(52, 44, MoveType::Silent),
            Move::new(51, 43, MoveType::Silent),
            Move::new(50, 42, MoveType::Silent),
            Move::new(49, 41, MoveType::Silent),
            Move::new(48, 40, MoveType::Silent),
            Move::new(55, 39, MoveType::DoublePush),
            Move::new(54, 38, MoveType::DoublePush),
            Move::new(53, 37, MoveType::DoublePush),
            Move::new(52, 36, MoveType::DoublePush),
            Move::new(51, 35, MoveType::DoublePush),
            Move::new(50, 34, MoveType::DoublePush),
            Move::new(49, 33, MoveType::DoublePush),
            Move::new(48, 32, MoveType::DoublePush),
        ];

        let white_pawn_moves = generate_pawn_moves(&board_state, PieceColor::White);
        compare_moves(&white_pawn_moves, &expected_moves_white);

        let expected_moves_black = [
            Move::new(15, 23, MoveType::Silent),
            Move::new(14, 22, MoveType::Silent),
            Move::new(13, 21, MoveType::Silent),
            Move::new(12, 20, MoveType::Silent),
            Move::new(11, 19, MoveType::Silent),
            Move::new(10, 18, MoveType::Silent),
            Move::new(9, 17, MoveType::Silent),
            Move::new(8, 16, MoveType::Silent),
            Move::new(15, 31, MoveType::DoublePush),
            Move::new(14, 30, MoveType::DoublePush),
            Move::new(13, 29, MoveType::DoublePush),
            Move::new(12, 28, MoveType::DoublePush),
            Move::new(11, 27, MoveType::DoublePush),
            Move::new(10, 26, MoveType::DoublePush),
            Move::new(9, 25, MoveType::DoublePush),
            Move::new(8, 24, MoveType::DoublePush),
        ];
        let black_pawn_moves = generate_pawn_moves(&board_state, PieceColor::Black);
        compare_moves(&black_pawn_moves, &expected_moves_black);
    }

    #[test]
    fn pawns_attacks() {
        let board_state = ChessBoardState::from_fen("8/2r5/3P4/4p3/2nP1P2/1P3P2/8/8 w - - 0 1");
        assert!(board_state.is_ok());
        let board_state = board_state.unwrap();

        let white_pawn_moves = generate_pawn_moves(&board_state, PieceColor::White);
        println!("{:?}", white_pawn_moves);
    }
}
