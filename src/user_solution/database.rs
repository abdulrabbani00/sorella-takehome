use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf
};

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::database::{AsStr, Db, DbReader, DbWriter};
use crate::types::{DatabaseKey, StoredType};

const MAGIC: &[u8; 4] = b"SDB\0";
const HEADER_SIZE: u64 = 16;
const INDEX_START_OFFSET: u64 = 4;

#[derive(Serialize, Deserialize)]
struct IndexEntry {
    offset: u64,
    length: u32
}

#[derive(Serialize, Deserialize)]
struct PersistedIndex {
    entries: HashMap<String, IndexEntry>
}

pub struct CandidateDatabase {
    path:     PathBuf,
    file:     File,
    index:   AHashMap<String, (u64, u32)>,
    /// Offset where the next record will be written (end of data section)
    data_end: u64
}

fn key_str(key: &DatabaseKey<'_>) -> String {
    key.as_str().as_ref().to_string()
}

impl CandidateDatabase {
    fn open_for_read_write(path: &PathBuf) -> std::io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
    }
}

impl<'a> Db<'a> for CandidateDatabase {
    type InitArgs = u8;

    fn new(path: PathBuf, _args: Self::InitArgs) -> Self {
        let mut file = Self::open_for_read_write(&path).expect("Failed to open database file");
        let mut index = AHashMap::new();
        let mut data_end = HEADER_SIZE;

        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if file_len >= HEADER_SIZE {
            let mut magic = [0u8; 4];
            file.read_exact(&mut magic).ok();
            if magic == *MAGIC {
                let mut index_start_bytes = [0u8; 8];
                file.seek(SeekFrom::Start(INDEX_START_OFFSET)).ok();
                file.read_exact(&mut index_start_bytes).ok();
                let index_start = u64::from_le_bytes(index_start_bytes);
                if index_start <= file_len && index_start >= HEADER_SIZE {
                    let index_len = (file_len - index_start) as usize;
                    let mut index_buf = vec![0u8; index_len];
                    file.seek(SeekFrom::Start(index_start)).ok();
                    file.read_exact(&mut index_buf).ok();
                    if let Ok(parsed) = serde_json::from_slice::<PersistedIndex>(&index_buf) {
                        for (k, e) in parsed.entries {
                            index.insert(k, (e.offset, e.length));
                        }
                        data_end = index_start;
                    }
                }
            }
            if index.is_empty() {
                file.seek(SeekFrom::End(0)).ok();
                data_end = file.stream_position().unwrap_or(HEADER_SIZE).max(HEADER_SIZE);
            }
            file.seek(SeekFrom::Start(0)).ok();
        } else {
            file.write_all(MAGIC).expect("Failed to write magic");
            file.write_all(&0u64.to_le_bytes()).expect("Failed to write placeholder");
            file.write_all(&[0u8; 4]).expect("Failed to write reserved");
            data_end = HEADER_SIZE;
        }

        Self {
            path,
            file,
            index,
            data_end
        }
    }

    fn args() -> Self::InitArgs {
        1
    }
}

impl<'a> DbWriter<'a> for CandidateDatabase {
    type Data = StoredType<'a>;
    type Key = DatabaseKey<'a>;

    fn write(&mut self, key: &Self::Key, data: &Self::Data) {
        let key_str = key_str(key);
        let key_bytes = key_str.as_bytes();
        let json = serde_json::to_vec(data).expect("Failed to serialize");
        let key_len = key_bytes.len() as u32;
        let json_len = json.len() as u32;

        self.file
            .seek(SeekFrom::Start(self.data_end))
            .expect("Failed to seek");
        self.file
            .write_all(&key_len.to_le_bytes())
            .expect("Failed to write key len");
        self.file.write_all(key_bytes).expect("Failed to write key");
        self.file
            .write_all(&json_len.to_le_bytes())
            .expect("Failed to write json len");
        self.file.write_all(&json).expect("Failed to write json");

        let json_offset = self.data_end + 4 + (key_len as u64) + 4;
        self.index.insert(key_str, (json_offset, json_len));
        self.data_end = json_offset + (json_len as u64);
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Data> {
        let key_str = key_str(key);
        let &(json_offset, json_len) = self.index.get(&key_str)?;
        let mut buf = vec![0u8; json_len as usize];
        self.file.seek(SeekFrom::Start(json_offset)).ok()?;
        self.file.read_exact(&mut buf).ok()?;
        let value: StoredType = serde_json::from_slice(&buf).ok()?;

        let key_bytes = key_str.as_bytes();
        let key_len = key_bytes.len() as u32;
        let json_len = 0u32;

        self.file
            .seek(SeekFrom::Start(self.data_end))
            .expect("Failed to seek");
        self.file
            .write_all(&key_len.to_le_bytes())
            .expect("Failed to write key len");
        self.file.write_all(key_bytes).expect("Failed to write key");
        self.file
            .write_all(&json_len.to_le_bytes())
            .expect("Failed to write tombstone");

        self.index.remove(&key_str);
        self.data_end += 4 + (key_len as u64) + 4;

        Some(value)
    }
}

impl<'a> DbReader<'a> for CandidateDatabase {
    type Data = StoredType<'a>;
    type Key = DatabaseKey<'a>;

    fn read(&self, key: &Self::Key) -> Option<Self::Data> {
        let key_str = key_str(key);
        let &(json_offset, json_len) = self.index.get(&key_str)?;
        if json_len == 0 {
            return None;
        }
        let mut buf = vec![0u8; json_len as usize];
        let mut file = File::open(&self.path).ok()?;
        file.seek(SeekFrom::Start(json_offset)).ok()?;
        file.read_exact(&mut buf).ok()?;
        serde_json::from_slice(&buf).ok()
    }
}

impl Drop for CandidateDatabase {
    fn drop(&mut self) {
        let index_json = {
            let entries: HashMap<String, IndexEntry> = self
                .index
                .iter()
                .map(|(k, (off, len))| {
                    (
                        k.clone(),
                        IndexEntry {
                            offset: *off,
                            length: *len
                        }
                    )
                })
                .collect();
            serde_json::to_string(&PersistedIndex { entries }).expect("Failed to serialize index")
        };

        self.file
            .set_len(self.data_end)
            .expect("Failed to truncate before index write");
        self.file
            .seek(SeekFrom::Start(self.data_end))
            .expect("Failed to seek for index write");
        self.file
            .write_all(index_json.as_bytes())
            .expect("Failed to write index");
        self.file
            .seek(SeekFrom::Start(INDEX_START_OFFSET))
            .expect("Failed to seek for header update");
        self.file
            .write_all(&self.data_end.to_le_bytes())
            .expect("Failed to write index offset");
        self.file.flush().expect("Failed to flush");
    }
}
