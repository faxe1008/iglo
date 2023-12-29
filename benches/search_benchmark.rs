use std::sync::{atomic::AtomicBool, Arc};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iglo::{
    chess::{board::ChessBoardState, chess_move::Move},
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
        TimeControl::FixedDepth(depth as u32),
        &stop,
    )
}

fn search_benchmark(c: &mut Criterion) {
    let mut bot = NPlyTranspoBot::default();
    let mut board_state = ChessBoardState::from_fen(
        "rnbqkbnr/pp1p1ppp/2p1p3/8/4P3/3P4/PPPN1PPP/R1BQKBNR b KQkq - 1 5",
    )
    .unwrap();

    let mut group = c.benchmark_group("sample-size-example");
    group.sample_size(10);
    group.bench_function("Search 6ply deep", |b| {
        b.iter(|| run_move_search(black_box(&mut bot), &mut board_state, 6))
    });
    group.finish();
}

criterion_group!(benches, search_benchmark);
criterion_main!(benches);
