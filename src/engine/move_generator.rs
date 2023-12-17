use crate::c_move;

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
            let empty_rank_3 = (self.empty_squares() & BitBoard::RANK_4).sSo() & self.empty_squares();
            empty_rank_3.sSo() & self.white_pieces[ChessPiece::Pawn as usize]
        } else {
            let empty_rank_6 = (self.empty_squares() & BitBoard::RANK_5).sNo() & self.empty_squares();
            empty_rank_6.sNo() & self.black_pieces[ChessPiece::Pawn as usize]
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

    let single_push_pawns = board_state.board.pawns_able_to_push(color);
    if single_push_pawns != BitBoard::EMPTY {
        for i in 0..64 {
            if single_push_pawns.get_bit(i) {
                let target = i as i32 + 8 * push_dir;
                if target >= 0 && target <= 63 {
                    moves.push(c_move!(i, target, MoveType::Silent));
                }
            }
        }
    }

    let double_push_pawns = board_state.board.pawns_able_to_double_push(color);
    if double_push_pawns != BitBoard::EMPTY {
        for i in 0..64 {
            if double_push_pawns.get_bit(i) {
                let target = i as i32 + 16 * push_dir;
                if target >= 0 && target <= 63 {
                    moves.push(c_move!(i, target, MoveType::Silent));
                }
            }
        }
    }

    moves
}

#[cfg(test)]
mod move_gen_tests {
    use crate::engine::{
        board::{ChessBoardState, PieceColor},
        move_generator::generate_pawn_moves,
    };

    #[test]
    fn pawns_moves_from_fen_simple() {
        let board_state =
            ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0");
        assert!(board_state.is_ok());
        let board_state = board_state.unwrap();

        println!("ASD {:?}", generate_pawn_moves(&board_state, PieceColor::White));
    }
}
