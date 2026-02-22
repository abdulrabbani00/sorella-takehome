use std::{fmt::Debug, path::PathBuf};

pub trait AsStr {
    fn as_str(&self) -> impl AsRef<str>;
}

pub trait Db<'a>: DbReader<'a> + DbWriter<'a> {
    type InitArgs;

    fn new(path: PathBuf, args: Self::InitArgs) -> Self;
    fn args() -> Self::InitArgs;
}

pub trait DbReader<'a> {
    type Key: Debug;
    type Data: Debug;

    fn read(&self, key: &Self::Key) -> Option<Self::Data>;
}

pub trait DbWriter<'a> {
    type Key: Debug;
    type Data: Debug;

    fn write(&mut self, key: &Self::Key, data: &Self::Data);
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Data>;
}
