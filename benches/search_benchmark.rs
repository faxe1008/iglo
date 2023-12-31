use std::sync::{atomic::AtomicBool, Arc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iglo::{
    chess::{board::ChessBoardState, chess_move::Move, perft::perft},
    engine::{
        bot::{ChessBot},
        bots::nplytranspo_bot::NPlyTranspoBot, time_control::TimeControl,
    },
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

fn search_benchmark(c: &mut Criterion) {
    let mut bot = NPlyTranspoBot::default();
    let mut board_state = ChessBoardState::from_fen(
        "rnbqkbnr/pp1p1ppp/2p1p3/8/4P3/3P4/PPPN1PPP/R1BQKBNR b KQkq - 1 5",
    )
    .unwrap();

    let mut group = c.benchmark_group("Search Move");
    group.sample_size(10);
    group.bench_function("Search 6ply deep", |b| {
        b.iter(|| run_move_search(black_box(&mut bot), &mut board_state, 6))
    });
    group.finish();
}

fn perft_benchmark(c: &mut Criterion) {

    let board_fens = ["r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8" ];

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


criterion_group!(benches, search_benchmark, perft_benchmark);
criterion_main!(benches);
