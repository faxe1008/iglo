

pub struct Move (pub u16);


impl Move {

    pub fn get_src(&self) -> u16 {
        self.0 & 0b111111
    }

    pub fn get_dst(&self) -> u16 {
        (self.0 >> 6) & 0b111111
    }

}
