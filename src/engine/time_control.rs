#[derive(PartialEq, Debug)]
pub enum TimeControl {
    Infinite,
    FixedDepth(u32)
}