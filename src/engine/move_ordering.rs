use crate::chess::{
    board::{ChessBoardState, ChessPiece},
    chess_move::Move,
};

// MVV_VLA[victim][attacker]
pub const MVV_LVA: [[u8; ChessPiece::PIECE_TYPE_COUNT + 1]; ChessPiece::PIECE_TYPE_COUNT + 1] = [
    [15, 14, 13, 12, 11, 10, 0], // victim P, attacker K, Q, R, B, N, P, None
    [25, 24, 23, 22, 21, 20, 0], // victim N, attacker K, Q, R, B, N, P, None
    [35, 34, 33, 32, 31, 30, 0], // victim B, attacker K, Q, R, B, N, P, None
    [45, 44, 43, 42, 41, 40, 0], // victim R, attacker K, Q, R, B, N, P, None
    [55, 54, 53, 52, 51, 50, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim King, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

pub fn order_moves(moves: &mut Vec<Move>, board_state: &ChessBoardState) {
    moves.sort_by(|a, b| {
        let a_src = a.get_moved_piece(board_state) as usize;
        let a_dst = a
            .get_captured_piece(board_state)
            .map(|c| c as usize)
            .unwrap_or(ChessPiece::PIECE_TYPE_COUNT);

        let b_src = b.get_moved_piece(board_state) as usize;
        let b_dst = b
            .get_captured_piece(board_state)
            .map(|c| c as usize)
            .unwrap_or(ChessPiece::PIECE_TYPE_COUNT);

        MVV_LVA[b_dst][b_src].cmp(&MVV_LVA[a_dst][a_src])
    });
}

#[cfg(test)]
mod move_ordering_tests {
    use crate::chess::{
        board::ChessBoardState,
        chess_move::{Move, MoveType},
        square::Square,
    };

    use super::order_moves;

    #[test]
    fn check_mvv_lva() {
        let board_state = ChessBoardState::from_fen(
            "rnb1kbn1/pp1p1ppp/2p1p3/8/2q1P3/3P1r2/PPPN1PPP/R1BQKBNR b KQq - 1 5",
        )
        .unwrap();

        let mut moves = vec![
            Move::new(Square::D2, Square::C4, MoveType::Capture),
            Move::new(Square::D3, Square::C4, MoveType::Capture),
            Move::new(Square::E4, Square::E5, MoveType::Silent),
            Move::new(Square::G1, Square::F3, MoveType::Capture),
        ];

        order_moves(&mut moves, &board_state);

        assert_eq!(
            moves[0],
            Move::new(Square::D3, Square::C4, MoveType::Capture),
            "Pawn capture should be first"
        );
        assert_eq!(
            moves[1],
            Move::new(Square::D2, Square::C4, MoveType::Capture),
            "Knight capture should be second"
        );
        assert_eq!(
            moves[2],
            Move::new(Square::G1, Square::F3, MoveType::Capture),
            "Rook capture should be third"
        );
        assert_eq!(
            moves[3],
            Move::new(Square::E4, Square::E5, MoveType::Silent),
            "Silent Move should be last"
        );
    }
}
