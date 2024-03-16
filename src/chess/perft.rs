use super::board::ChessBoardState;

pub fn perft(board_state: &ChessBoardState, depth: u32) -> u64 {
    if depth < 1 {
        1
    } else {
        let moves = board_state.generate_legal_moves_for_current_player::<false>();

        if depth == 1 {
            moves.len() as u64
        } else {
            moves
                .iter()
                .map(|m| {
                    let child: ChessBoardState = board_state.exec_move(*m);
                    perft(&child, depth - 1)
                })
                .sum()
        }
    }
}

#[cfg(test)]
mod perft_tests {
    use crate::chess::board::ChessBoardState;
    use crate::chess::perft::perft;

    #[test]
    fn base_perft() {
        let board_state = ChessBoardState::starting_state();
        assert_eq!(perft(&board_state, 0), 1);
        assert_eq!(perft(&board_state, 1), 20);
        assert_eq!(perft(&board_state, 2), 400);
        assert_eq!(perft(&board_state, 3), 8902);
        assert_eq!(perft(&board_state, 4), 197281);
        assert_eq!(perft(&board_state, 5), 4865609);
    }

    #[test]
    fn chess_wiki_position_4() {
        let board_state = ChessBoardState::from_fen(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        )
        .unwrap();
        assert_eq!(perft(&board_state, 1), 6);
        assert_eq!(perft(&board_state, 2), 264);
        assert_eq!(perft(&board_state, 3), 9467);
        assert_eq!(perft(&board_state, 4), 422333);
        assert_eq!(perft(&board_state, 5), 15833292);
        assert_eq!(perft(&board_state, 6), 706045033);
    }

    #[test]
    fn chess_wiki_position_5() {
        let board_state =
            ChessBoardState::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap();
        assert_eq!(perft(&board_state, 1), 44);
        assert_eq!(perft(&board_state, 2), 1486);
        assert_eq!(perft(&board_state, 3), 62379);
        assert_eq!(perft(&board_state, 4), 2103487);
        assert_eq!(perft(&board_state, 5), 89941194);
    }
}
