use std::sync::{atomic::AtomicBool, Arc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iglo::{
    chess::{board::ChessBoardState, chess_move::{Move, MoveType}, perft::perft, square::Square},
    engine::{bot::ChessBot, bots::nplytranspo_bot::NPlyTranspoBot, move_ordering::order_moves, search::SearchInfo, time_control::TimeControl},
};

fn run_move_search(
    bot: &mut NPlyTranspoBot,
    mut board_state: &mut ChessBoardState,
    depth: u16,
) -> Move {
    let stop: Arc<AtomicBool> = Arc::new(false.into());
    bot.search_best_move(
        &mut board_state,
        TimeControl::FixedDepth(depth as u64),
        &stop,
    )
}

fn benchmark_order_moves(c: &mut Criterion) {
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
    let search_info = SearchInfo::default();

    c.bench_function("order_moves", |b| {
        b.iter(|| order_moves(black_box(&mut moves), black_box(&board_state), black_box(&search_info), black_box(4)))
    });
}

fn search_benchmark(c: &mut Criterion) {
    let mut bot = NPlyTranspoBot::default();
    let mut board_state = ChessBoardState::from_fen(
        "rnbqkbnr/pp1p1ppp/2p1p3/8/4P3/3P4/PPPN1PPP/R1BQKBNR b KQkq - 9 9",
    )
    .unwrap();

    let mut group = c.benchmark_group("Search Move");
    group.sample_size(10);
    group.bench_function("Search 6ply deep", |b| {
        b.iter(|| run_move_search(black_box(&mut bot), &mut board_state, 7))
    });
    group.finish();
}

fn perft_benchmark(c: &mut Criterion) {
    let board_fens = [
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    ];

    let mut group = c.benchmark_group("Peft");
    group.sample_size(10);

    for fen in &board_fens {
        let board_state = ChessBoardState::from_fen(fen).unwrap();

        group.bench_function(format!("Perft {}", fen), |b| {
            b.iter(|| perft(black_box(&board_state), 6))
        });
    }
    group.finish()
}

criterion_group!(benches, search_benchmark, perft_benchmark, benchmark_order_moves);
criterion_main!(benches);
