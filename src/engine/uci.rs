use std::{sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc}, thread, io::{stdin, BufRead}};

use crate::chess::{board::ChessBoardState, chess_move::Move, perft::perft};

const ENGINE_NAME: &str = env!("CARGO_PKG_NAME");
const ENGINE_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq)]
enum UCICommand {
    UCI,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    UCINewGame,
    Position(ChessBoardState),
    Peft(u32),
    Quit,
    Stop
}

fn board_exec_movelist(board_state: &mut ChessBoardState, moves: &Vec<&str>)  {
    for move_str in moves {
        if let Ok(mv) = Move::try_from(((*move_str).trim(), &*board_state)) {
            *board_state = board_state.exec_move(mv);
        }
    }
}

impl TryFrom<&str> for UCICommand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut tokens = value.split_whitespace();

        match tokens.next() {
            Some("uci") => Ok(UCICommand::UCI),
            Some("isready") => Ok(UCICommand::IsReady),
            Some("debug") => match tokens.next() {
                Some("on") => Ok(UCICommand::Debug(true)),
                Some("off") => Ok(UCICommand::Debug(false)),
                _ => Err(()),
            },
            Some("setoption") => {
                let opt_name = tokens.next();
                let opt_val = tokens.next();
                if opt_name.is_none() || opt_val.is_none() {
                    Err(())
                } else {
                    Ok(UCICommand::SetOption(
                        opt_name.unwrap().to_string(),
                        opt_val.unwrap().to_string(),
                    ))
                }
            }
            Some("ucinewgame") => Ok(UCICommand::UCINewGame),
            Some("position") => {
                let mut chessboard_state = match tokens.next() {
                    Some("startpos") => ChessBoardState::starting_state(),
                    Some("fen") => {
                        let fen_str = tokens.by_ref().take(6).collect::<Vec<&str>>().join(" ");
                        ChessBoardState::from_fen(dbg!(&fen_str))?
                    }
                    Some(_) | None => return Err(()),
                };

                let move_list = if let Some("moves") = tokens.next() {
                    tokens.collect()
                } else {
                    vec![]
                };

                board_exec_movelist(&mut chessboard_state, &move_list);

                Ok(UCICommand::Position(chessboard_state))
            },
            Some("quit") => Ok(UCICommand::Quit),
            Some("stop") => Ok(UCICommand::Stop),
            Some("perft") => {
                let depth = tokens.next().unwrap_or("1").parse::<u32>();
                if let Ok(d) = depth {
                    Ok(UCICommand::Peft(d))
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

struct UCIController();
pub struct UCIReader {
    stop: Arc<AtomicBool>,
    controller_tx: mpsc::Sender<UCICommand>,
}

impl Default for UCIReader {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<UCICommand>();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = stop.clone();
        thread::spawn(move || UCIController::run(rx, thread_stop));

        Self {
            stop,
            controller_tx: tx,
        }
    }
}

impl UCIReader {
    /// Start UCI I/O loop
    pub fn run(&self) {
        println!("{ENGINE_NAME} v{ENGINE_VERSION} by {ENGINE_AUTHOR}");

        let stream = stdin().lock();

        for line in stream.lines().map(|l| l.expect("Parsing error!")) {
            match UCICommand::try_from(line.as_ref()) {
                Ok(command) => {
                    match command {
                        UCICommand::UCI => {
                            println!("id name {ENGINE_NAME} {ENGINE_VERSION}");
                            println!("id author {ENGINE_AUTHOR}");
                            println!("uciok");
                        }
                        UCICommand::IsReady => {
                            println!("readyok");
                        }
                        UCICommand::Stop => self.stop.store(true, Ordering::SeqCst), // strict ordering
                        UCICommand::Quit => return,
                        _ => self.controller_tx.send(command).unwrap(),
                    }
                }
                Err(e) => println!("Error parsing {line}"),
            };
        }
    }
}

impl UCIController {
    fn run(rx: mpsc::Receiver<UCICommand>, stop: Arc<AtomicBool>) {
        let mut board_state = ChessBoardState::starting_state();

        for command in &rx {
            match command {
                UCICommand::UCINewGame => {
                    board_state = ChessBoardState::starting_state();
                },
                UCICommand::SetOption(name, value) => {
                },

                UCICommand::Position(new_state) => {
                    board_state = new_state;
                },
                UCICommand::Peft(depth) => {
                    let nodes = perft(&board_state, depth);
                    println!("Nodes searched: {}", nodes);
                }
                _ => eprintln!("Unexpected UCI command!"),
            }
        }
    }
}


#[cfg(test)]
mod uci_tests {
    use crate::chess::{board::ChessBoardState, chess_move::{Move, MoveType}, square::Square};

    use super::UCICommand;

    #[test]
    fn test_simple_commands() {
        assert_eq!(UCICommand::try_from("uci").unwrap(), UCICommand::UCI);
        assert_eq!(
            UCICommand::try_from("isready").unwrap(),
            UCICommand::IsReady
        );
        assert_eq!(
            UCICommand::try_from("debug on").unwrap(),
            UCICommand::Debug(true)
        );
        assert_eq!(
            UCICommand::try_from("debug off").unwrap(),
            UCICommand::Debug(false)
        );
        assert_eq!(
            UCICommand::try_from("setoption foo bar").unwrap(),
            UCICommand::SetOption("foo".into(), "bar".into())
        );
        assert_eq!(
            UCICommand::try_from("ucinewgame").unwrap(),
            UCICommand::UCINewGame
        );
    }

    #[test]
    fn test_position_start() {
        assert_eq!(
            UCICommand::try_from("position startpos").unwrap(),
            UCICommand::Position(ChessBoardState::starting_state())
        )
    }

    #[test]
    fn test_position_start_moves() {
        let board_after_moves = UCICommand::try_from("position startpos moves c2c4 g8f6 d1a4 g7g6 g1f3 f8h6 a4a3 e8g8").unwrap();
        let expected_board = UCICommand::Position(ChessBoardState::from_fen("rnbq1rk1/pppppp1p/5npb/8/2P5/Q4N2/PP1PPPPP/RNB1KB1R w KQ - 4 8").unwrap());
        assert_eq!(board_after_moves, expected_board);
    }

    #[test]
    fn test_position_fen() {
        let expected_state = ChessBoardState::from_fen(
            "r1bqk1nr/pp1p1ppp/2nb4/2p1p3/Q1P5/2N2P2/PP1PP1PP/R1B1KBNR w KQkq - 0 1",
        )
        .unwrap();
        assert_eq!(
            UCICommand::try_from("position fen r1bqk1nr/pp1p1ppp/2nb4/2p1p3/Q1P5/2N2P2/PP1PP1PP/R1B1KBNR w KQkq - 0 1").unwrap(),
            UCICommand::Position(expected_state)
        )
    }

    #[test]
    fn check_move_deserialization(){
        let board = ChessBoardState::from_fen("r3k2r/pPppp2p/8/2Rr1pP1/8/8/PPPPP1PP/R3K2R w KQkq f6 0 1").unwrap();

        let assert_mv = |str, src, dst, ty| {
            let mv = Move::try_from((str, &board)).unwrap();
            assert_eq!(mv.get_src(), src);
            assert_eq!(mv.get_dst(), dst);
            assert_eq!(mv.get_type(), ty);
        };

        assert_mv("g5f6", Square::G5, Square::F6, MoveType::EnPassant);
        assert_mv("e1g1", Square::E1, Square::G1, MoveType::CastleKingSide);
        assert_mv("e1c1", Square::E1, Square::C1, MoveType::CastleQueenSide);
        assert_mv("b2b4", Square::B2, Square::B4, MoveType::DoublePush);
        assert_mv("c5d5", Square::C5, Square::D5, MoveType::Capture);
        assert_mv("c5b5", Square::C5, Square::B5, MoveType::Silent);

        assert_mv("b7b8r", Square::B7, Square::B8, MoveType::RookPromotion);
        assert_mv("b7b8b", Square::B7, Square::B8, MoveType::BishopPromotion);
        assert_mv("b7b8n", Square::B7, Square::B8, MoveType::KnightPromotion);
        assert_mv("b7b8q", Square::B7, Square::B8, MoveType::QueenPromotion);
      
        assert_mv("b7a8r", Square::B7, Square::A8, MoveType::RookCapPromotion);
        assert_mv("b7a8b", Square::B7, Square::A8, MoveType::BishopCapPromotion);
        assert_mv("b7a8n", Square::B7, Square::A8, MoveType::KnightCapPromotion);
        assert_mv("b7a8q", Square::B7, Square::A8, MoveType::QueenCapPromotion);
    }
}
