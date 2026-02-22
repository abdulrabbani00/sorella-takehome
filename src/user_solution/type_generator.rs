use crate::{
    type_generator::TypeGenerator,
    types::{DatabaseKey, StoredType}
};

pub struct CandidateTypeGen;

impl<'a> TypeGenerator<'a> for CandidateTypeGen {
    type Data = StoredType<'static>;
    type InitArgs = u8;
    type Key = DatabaseKey<'static>;

    fn new(_: Self::InitArgs) -> Self {
        todo!()
    }

    fn args() -> Self::InitArgs {
        1
    }

    fn generate_key(&mut self) -> Self::Key {
        todo!()
    }

    fn generate_data(&mut self) -> Self::Data {
        todo!()
    }
}
