use std::ops::Range;

pub const RANGE_SIZE: Range<usize> = 10usize..30usize;

pub trait TypeGenerator<'a> {
    type Key: 'a;
    type Data: 'a;
    type InitArgs;

    fn new(args: Self::InitArgs) -> Self;
    fn args() -> Self::InitArgs;

    fn generate_key(&mut self) -> Self::Key;
    fn generate_data(&mut self) -> Self::Data;
}
