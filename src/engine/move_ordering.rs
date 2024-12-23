use crate::chess::{
    board::{ChessBoardState, ChessPiece},
    chess_move::Move,
};

use super::search::{SearchInfo, MAX_KILLER_MOVES};

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

const MVV_LVA_OFFSET: u32 = u32::MAX - 256;
const KILLER_VALUE: u32 = 10;

pub fn order_moves(
    moves: &mut Vec<Move>,
    board_state: &ChessBoardState,
    search_info: &SearchInfo,
    ply_from_root: u16,
) {
    let ply = ply_from_root as usize;

    // Cache the evaluations to avoid repeated calculations
    let mut move_evals: Vec<(Move, u32)> = moves.iter()
        .map(|&mv| (mv, move_order_eval(mv, board_state, search_info, ply)))
        .collect();

    // Sort the moves based on their evaluations
    move_evals.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    // Update the original moves vector with the sorted moves
    for (i, (mv, _)) in move_evals.into_iter().enumerate() {
        moves[i] = mv;
    }
}

#[inline(always)]
fn move_order_eval(
    mv: Move,
    board_state: &ChessBoardState,
    search_info: &SearchInfo,
    ply: usize,
) -> u32 {
    let src_piece = mv.get_moved_piece(board_state) as usize;

    if let Some(captured_piece) = mv.get_captured_piece(board_state) {
        MVV_LVA_OFFSET + MVV_LVA[captured_piece as usize][src_piece] as u32
    } else {
        search_info.killer_moves.iter().enumerate().find_map(|(i, killers)| {
            if mv == killers[ply] {
                Some(MVV_LVA_OFFSET - ((i as u32 + 1) * KILLER_VALUE))
            } else {
                None
            }
        }).unwrap_or(0)
    }
}

#[cfg(test)]
mod move_ordering_tests {
    use crate::{
        chess::{
            board::ChessBoardState,
            chess_move::{Move, MoveType},
            square::Square,
        },
        engine::search::SearchInfo,
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

        order_moves(&mut moves, &board_state, &SearchInfo::default(), 4);

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
