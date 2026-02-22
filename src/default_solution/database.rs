use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    marker::PhantomData,
    path::PathBuf
};

use serde::{Deserialize, Serialize};

use crate::database::{AsStr, Db, DbReader, DbWriter};

pub struct DefaultDatabase<K, T> {
    db_path: PathBuf,
    _p:      PhantomData<(K, T)>
}

impl<K, T> Db<'_> for DefaultDatabase<K, T>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug,
    K: AsStr + Debug
{
    type InitArgs = u8;

    fn new(path: PathBuf, _: Self::InitArgs) -> Self {
        Self { db_path: path, _p: PhantomData }
    }

    fn args() -> Self::InitArgs {
        1
    }
}

impl<K, T> DefaultDatabase<K, T> {
    fn get_db_handle(&self) -> File {
        OpenOptions::new()
            .truncate(false)
            .append(true)
            .create(true)
            .read(true)
            .open(self.db_path.clone())
            .unwrap()
    }
}

impl<K, T> DbWriter<'_> for DefaultDatabase<K, T>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug,
    K: AsStr + Debug
{
    type Data = T;
    type Key = K;

    fn write(&mut self, key: &Self::Key, data: &Self::Data) {
        let format =
            format!("{} => {};\n", key.as_str().as_ref(), serde_json::to_string(&data).unwrap());

        let mut file_handle = self.get_db_handle();

        write!(&mut file_handle, "{}", format).unwrap();
        file_handle.flush().unwrap();
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Data> {
        let mut file_handle = self.get_db_handle();
        file_handle.seek(std::io::SeekFrom::Start(0)).unwrap();

        let mut buffer = Vec::new();
        file_handle.read_to_end(&mut buffer).unwrap();

        let string = String::from_utf8(buffer).unwrap();

        let key_prefix = format!("{} => ", key.as_str().as_ref());

        let start_index = string.find(&key_prefix)?;
        let data_start = start_index + key_prefix.len();
        let end_index = string[start_index..].find(';')?;

        let parsed_data = serde_json::from_str(&string[data_start..start_index + end_index]).ok();

        // Remove the entire line including the trailing newline
        let line_end = start_index + end_index + 2; // +2 for ";\n"
        let mut new_data = String::new();
        new_data.push_str(&string[..start_index]);
        if line_end <= string.len() {
            new_data.push_str(&string[line_end..]);
        }

        file_handle.set_len(0).unwrap();
        file_handle.seek(std::io::SeekFrom::Start(0)).unwrap();
        write!(file_handle, "{}", new_data).unwrap();
        file_handle.flush().unwrap();

        parsed_data
    }
}

impl<K, T> DbReader<'_> for DefaultDatabase<K, T>
where
    T: Serialize + for<'a> Deserialize<'a> + Debug,
    K: AsStr + Debug
{
    type Data = T;
    type Key = K;

    fn read(&self, key: &Self::Key) -> Option<Self::Data> {
        let mut file_handle = self.get_db_handle();
        file_handle.seek(std::io::SeekFrom::Start(0)).unwrap();

        let mut buffer = Vec::new();
        file_handle.read_to_end(&mut buffer).unwrap();

        let string = String::from_utf8(buffer).unwrap();

        let key_prefix = format!("{} => ", key.as_str().as_ref());

        let start_index = string.find(&key_prefix)?;
        let data_start = start_index + key_prefix.len();
        let end_index = string[start_index..].find(';')?;

        serde_json::from_str(&string[data_start..start_index + end_index]).ok()
    }
}
