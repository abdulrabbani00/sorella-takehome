use std::path::PathBuf;

use crate::{
    database::{Db, DbReader, DbWriter},
    types::{DatabaseKey, StoredType}
};

pub struct CandidateDatabase {}

impl Db<'_> for CandidateDatabase {
    type InitArgs = u8;

    fn new(_path: PathBuf, _args: Self::InitArgs) -> Self {
        todo!()
    }

    fn args() -> Self::InitArgs {
        1
    }
}

impl DbReader<'_> for CandidateDatabase {
    type Data = StoredType<'static>;
    type Key = DatabaseKey<'static>;

    fn read(&self, _key: &Self::Key) -> Option<Self::Data> {
        todo!()
    }
}

impl DbWriter<'_> for CandidateDatabase {
    type Data = StoredType<'static>;
    type Key = DatabaseKey<'static>;

    fn write(&mut self, _key: &Self::Key, _data: &Self::Data) {
        todo!()
    }

    fn remove(&mut self, _key: &Self::Key) -> Option<Self::Data> {
        todo!()
    }
}
