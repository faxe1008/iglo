use std::{env, fs::File, io::Write};

use chessica::engine::{bitboard::BitBoard, square::Square};

const KNIGHT_OFFSETS: [(i32, i32); 8] = [
    (-1, -2),
    (1, 2),
    (-2, -1),
    (2, 1),
    (1, -2),
    (-1, 2),
    (2, -1),
    (-2, 1),
];

const KING_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

fn generate_jump_piece_lookup(offset: &[(i32, i32)]) -> Vec<BitBoard> {
    let mut lookup = Vec::new();
    for y in 0..8 {
        for x in 0..8 {
            let mut jump_map = BitBoard::EMPTY;
            for (x_off, y_off) in offset {
                let new_pos_x = x + x_off;
                let new_pos_y = y + y_off;

                if new_pos_x < 0 || new_pos_x >= 8 || new_pos_y < 0 || new_pos_y >= 8 {
                    continue;
                }
                jump_map = jump_map
                    .set_bit(Square::square_from_pos(new_pos_x as u16, new_pos_y as u16) as usize);
            }
            lookup.push(jump_map);
        }
    }
    lookup
}

fn write_bitboards_to_file(path: &str, boards: &Vec<BitBoard>) {
    let mut file = File::create(path).unwrap();
    for bb in boards {
        file.write_all(unsafe { &std::mem::transmute::<BitBoard, [u8; 8]>(*bb) })
            .unwrap();
    }
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Provide piece to gen table(s) for [knight,bishop,rook,queen,king]");
        return Err(());
    }

    match args[1].as_ref() {
        "knight" => {
            write_bitboards_to_file(
                "knight_lookup.bin",
                &generate_jump_piece_lookup(&KNIGHT_OFFSETS),
            );
        }
        "king" => {
            write_bitboards_to_file(
                "king_lookup.bin",
                &generate_jump_piece_lookup(&KING_OFFSETS),
            );
        }
        _ => {
            panic!("Unknown piece to generate for")
        }
    };

    Ok(())
}
