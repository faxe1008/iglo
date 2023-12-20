pub struct Square;

impl Square {
    pub const A8: u16 = 0;
    pub const B8: u16 = 1;
    pub const C8: u16 = 2;
    pub const D8: u16 = 3;
    pub const E8: u16 = 4;
    pub const F8: u16 = 5;
    pub const G8: u16 = 6;
    pub const H8: u16 = 7;
    pub const A7: u16 = 8;
    pub const B7: u16 = 9;
    pub const C7: u16 = 10;
    pub const D7: u16 = 11;
    pub const E7: u16 = 12;
    pub const F7: u16 = 13;
    pub const G7: u16 = 14;
    pub const H7: u16 = 15;
    pub const A6: u16 = 16;
    pub const B6: u16 = 17;
    pub const C6: u16 = 18;
    pub const D6: u16 = 19;
    pub const E6: u16 = 20;
    pub const F6: u16 = 21;
    pub const G6: u16 = 22;
    pub const H6: u16 = 23;
    pub const A5: u16 = 24;
    pub const B5: u16 = 25;
    pub const C5: u16 = 26;
    pub const D5: u16 = 27;
    pub const E5: u16 = 28;
    pub const F5: u16 = 29;
    pub const G5: u16 = 30;
    pub const H5: u16 = 31;
    pub const A4: u16 = 32;
    pub const B4: u16 = 33;
    pub const C4: u16 = 34;
    pub const D4: u16 = 35;
    pub const E4: u16 = 36;
    pub const F4: u16 = 37;
    pub const G4: u16 = 38;
    pub const H4: u16 = 39;
    pub const A3: u16 = 40;
    pub const B3: u16 = 41;
    pub const C3: u16 = 42;
    pub const D3: u16 = 43;
    pub const E3: u16 = 44;
    pub const F3: u16 = 45;
    pub const G3: u16 = 46;
    pub const H3: u16 = 47;
    pub const A2: u16 = 48;
    pub const B2: u16 = 49;
    pub const C2: u16 = 50;
    pub const D2: u16 = 51;
    pub const E2: u16 = 52;
    pub const F2: u16 = 53;
    pub const G2: u16 = 54;
    pub const H2: u16 = 55;
    pub const A1: u16 = 56;
    pub const B1: u16 = 57;
    pub const C1: u16 = 58;
    pub const D1: u16 = 59;
    pub const E1: u16 = 60;
    pub const F1: u16 = 61;
    pub const G1: u16 = 62;
    pub const H1: u16 = 63;

    pub const NUM: u16 = 64;


    pub fn designator_str_from_index(index: u16) -> String {
        let file = index % 8;
        let rank = 7 - (index / 8);

        let file_char = char::from_u32('A' as u32 + file as u32).unwrap();
        let rank_char =  char::from_u32('1' as u32 + rank as u32).unwrap();
        [file_char, rank_char].iter().collect()
    }

    pub fn square_from_pos(x: u16, y: u16) -> u16 {
        x + y * 8
    }

    pub fn add_offset(square: u16, x: i32, y: i32) -> Option<u16> {
        let rank = (square as i32 / 8_i32) + y;
        let file = (square as i32 % 8_i32) + x;

        if rank < 0 || rank > 7 || file < 0 || file > 7 {
            None
        } else {
            Some(file as u16 + rank as u16 * 8)
        }
    }

}
