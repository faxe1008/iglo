use std::{env, fs::File, io::Write};

use chessica::engine::{
    bitboard::{BitBoard, MagicEntry},
    square::Square,
};
use rand::random;

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


const NON_EDGE_BOARD : BitBoard = BitBoard(0x7e7e7e7e7e7e00);

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

const ROOK_OFFSETS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
const BISHOP_OFFSETS: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

fn generate_sliding_piece_moves(
    square: u16,
    mut blockers: BitBoard,
    offsets: &[(i32, i32)],
) -> BitBoard {
    let mut moves = BitBoard::EMPTY;
    let mut i = 0;
    while i < offsets.len() {
        let (dx, dy) = offsets[i];
        let mut square = square;
        while !blockers.get_bit(square as usize) {
            if let Some(sq) = Square::add_offset(square, dx, dy) {
                square = sq;
                moves = moves.set_bit(square as usize);
            } else {
                break;
            }
        }
        i += 1;
    }
    moves
}

fn rook_blocker_mask(pos: u16) -> BitBoard {
    let mut blocker_map = BitBoard::EMPTY;

    let rank = pos / 8;
    let file = pos % 8;

    for x in 1..7 {
        blocker_map = blocker_map.set_bit(Square::square_from_pos(x, rank) as usize);
    }
    for y in 1..7 {
        blocker_map = blocker_map.set_bit(Square::square_from_pos(file, y) as usize);
    }
    blocker_map = blocker_map.clear_bit(pos as usize);
    blocker_map
}

fn bishop_blocker_mask(pos: u16) -> BitBoard {
    let mut blocker_map = BitBoard::EMPTY;

    let rank = pos / 8;
    let file = pos % 8;


    let mut upper_left = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if upper_left.0 == 0 {
            break;
        }
        upper_left = upper_left.sNoWe();
        blocker_map = blocker_map | upper_left;
    }


    let mut upper_right = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if upper_right.0 == 0 {
            break;
        }
        upper_right = upper_right.sNoEa();
        blocker_map = blocker_map | upper_right;
    }

    let mut lower_left = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if lower_left.0 == 0 {
            break;
        }
        lower_left = lower_left.sSoEa();
        blocker_map = blocker_map | lower_left;
    }

    let mut lower_right = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if lower_right.0 == 0 {
            break;
        }
        lower_right = lower_right.sSoWe();
        blocker_map = blocker_map | lower_right;
    }
    

    blocker_map = blocker_map.clear_bit(pos as usize);
    blocker_map = blocker_map & NON_EDGE_BOARD;
    blocker_map
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

    dbg!(bishop_blocker_mask(20));

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
