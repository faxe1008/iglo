use std::{fs::File, io::Write};

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

const ROOK_OFFSETS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
const BISHOP_OFFSETS: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];

const ROOK_INDEX_BITS: u8 = 12;
const BISHOP_INDEX_BITS: u8 = 12;

const NON_EDGE_BOARD: BitBoard = BitBoard(0x7e7e7e7e7e7e00);

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

fn generate_sliding_piece_moves(
    square: u16,
    blockers: BitBoard,
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

    let mut upper_left = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if upper_left.0 == 0 {
            break;
        }
        upper_left = upper_left.s_no_we();
        blocker_map = blocker_map | upper_left;
    }

    let mut upper_right = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if upper_right.0 == 0 {
            break;
        }
        upper_right = upper_right.s_no_ea();
        blocker_map = blocker_map | upper_right;
    }

    let mut lower_left = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if lower_left.0 == 0 {
            break;
        }
        lower_left = lower_left.s_so_ea();
        blocker_map = blocker_map | lower_left;
    }

    let mut lower_right = BitBoard::EMPTY.set_bit(pos as usize);
    loop {
        if lower_right.0 == 0 {
            break;
        }
        lower_right = lower_right.s_so_we();
        blocker_map = blocker_map | lower_right;
    }

    blocker_map = blocker_map.clear_bit(pos as usize);
    blocker_map = blocker_map & NON_EDGE_BOARD;
    blocker_map
}

fn rook_moves(pos: u16, blockers: BitBoard) -> BitBoard {
    generate_sliding_piece_moves(pos, blockers, &ROOK_OFFSETS)
}

fn bishop_moves(pos: u16, blockers: BitBoard) -> BitBoard {
    generate_sliding_piece_moves(pos, blockers, &BISHOP_OFFSETS)
}

fn find_magic(
    move_gen_fn: fn(u16, BitBoard) -> BitBoard,
    blocker_mask_fn: fn(u16) -> BitBoard,
    square: u16,
    index_bits: u8,
) -> (MagicEntry, Vec<BitBoard>) {
    let blocker_mask = blocker_mask_fn(square);
    loop {
        // Magics require a low number of active bits, so we AND
        // by two more random values to cut down on the bits set.
        let magic = random::<u64>() & random::<u64>();
        let magic_entry = MagicEntry {
            blocker_mask,
            magic,
            index_bits,
        };
        if let Ok(table) = try_make_table(move_gen_fn, square, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

struct TableFillError;

fn try_make_table(
    move_gen_fn: fn(u16, BitBoard) -> BitBoard,
    square: u16,
    magic_entry: &MagicEntry,
) -> Result<Vec<BitBoard>, TableFillError> {
    let mut table = vec![BitBoard::EMPTY; 1 << magic_entry.index_bits];
    // Iterate all configurations of blockers
    for blockers in magic_entry.blocker_mask.iter_subsets() {
        let moves = move_gen_fn(square, blockers);
        let table_entry = &mut table[magic_entry.magic_index(blockers)];
        if table_entry.0 == 0 {
            // Write to empty slot
            *table_entry = moves;
        } else if *table_entry != moves {
            // Having two different move sets in the same slot is a hash collision
            return Err(TableFillError);
        }
    }
    Ok(table)
}

fn generate_magic_entries(
    move_gen_fn: fn(u16, BitBoard) -> BitBoard,
    blocker_mask_fn: fn(u16) -> BitBoard,
    index_bits: u8,
) -> ([MagicEntry; 64], [Vec<BitBoard>; 64]) {

    let mut magic_array = [MagicEntry { blocker_mask:BitBoard::EMPTY, magic: 0, index_bits: 0}; 64];
    let mut moves_array: [Vec<BitBoard>; 64] = vec![Vec::new(); 64].try_into().expect("static");

    for square in 0..Square::NUM {
        let (magic, moves)  = find_magic(move_gen_fn, blocker_mask_fn, square, index_bits);
        magic_array[square as usize] = magic;
        dbg!(&moves.len());
        moves_array[square as usize] = moves;
    }
    (magic_array, moves_array)
}

fn write_bitboards_to_file(path: &str, boards: &[BitBoard]) {
    let mut file = File::create(path).unwrap();
    for bb in boards {
        file.write_all(unsafe { &std::mem::transmute::<BitBoard, [u8; 8]>(*bb) })
            .unwrap();
    }
}

fn write_magic_entries_to_file(path: &str, magics: &[MagicEntry]){
    let mut file = File::create(path).unwrap();
    for me in magics {
        file.write_all(unsafe { &std::mem::transmute::<MagicEntry, [u8; 17]>(*me) })
            .unwrap();
    }
}

fn write_moves_entries_to_file(path: &str, moves: &[Vec<BitBoard>]) {
    let mut file = File::create(path).unwrap();
    assert!(moves.len() == 64);
    for mv in moves {
        for single_mv in mv {
            file.write_all(unsafe { &std::mem::transmute::<BitBoard, [u8; 8]>(*single_mv) }).unwrap();
        }
    }
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Provide piece to gen table(s) for [knight,bishop,rook,queen,king]");
        return Err(());
    }

    generate_magic_entries(
        rook_moves,
        rook_blocker_mask,
        ROOK_INDEX_BITS
    );

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
        },
        "rook" => {
            let (magics, moves) = generate_magic_entries(rook_moves, rook_blocker_mask, ROOK_INDEX_BITS);
            write_magic_entries_to_file("rook_magics.bin", &magics);
            write_moves_entries_to_file("rook_moves.bin", &moves);
        },
        "bishop" => {
            let (magics, moves) = generate_magic_entries(bishop_moves, bishop_blocker_mask, BISHOP_INDEX_BITS);
            write_magic_entries_to_file("bishop_magics.bin", &magics);
            write_moves_entries_to_file("bishop_moves.bin", &moves);
        }
        _ => {
            panic!("Unknown piece to generate for")
        }
    };

    Ok(())
}
