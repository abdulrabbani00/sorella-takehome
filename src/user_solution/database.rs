use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf
};

use ahash::AHashMap;
use memmap2::Mmap;

use crate::database::{AsStr, Db, DbReader, DbWriter};
use crate::types::{DatabaseKey, StoredType};

const MAGIC: &[u8; 4] = b"SDB\0";
const HEADER_SIZE: u64 = 16;
const BUF_SIZE: usize = 64 * 1024;

fn index_path(path: &PathBuf) -> PathBuf {
    PathBuf::from(path.to_string_lossy().to_string() + ".index")
}

fn key_str(key: &DatabaseKey<'_>) -> String {
    key.as_str().as_ref().to_string()
}

pub struct CandidateDatabase {
    path:       PathBuf,
    index_path: PathBuf,
    writer:     RefCell<BufWriter<File>>,
    mmap:       RefCell<Option<Mmap>>,
    index:      AHashMap<String, (u64, u32)>,
    data_end:   u64,
    dirty:      RefCell<bool>
}

impl CandidateDatabase {
    fn open_data_file(path: &PathBuf) -> std::io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
    }

    fn load_index(path: &PathBuf, index_path: &PathBuf) -> (AHashMap<String, (u64, u32)>, u64) {
        let mut index = AHashMap::new();
        let mut data_end = HEADER_SIZE;

        let index_bytes = match std::fs::read(index_path) {
            Ok(b) => b,
            Err(_) => return (index, data_end)
        };

        if index_bytes.len() < 4 {
            return (index, data_end);
        }

        let mut pos = 0;
        let count = u32::from_le_bytes([index_bytes[0], index_bytes[1], index_bytes[2], index_bytes[3]]) as usize;
        pos += 4;

        for _ in 0..count {
            if pos + 4 > index_bytes.len() {
                break;
            }
            let key_len = u32::from_le_bytes(index_bytes[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if pos + key_len + 8 + 4 > index_bytes.len() {
                break;
            }
            let key = String::from_utf8_lossy(&index_bytes[pos..pos + key_len]).to_string();
            pos += key_len;
            let offset = u64::from_le_bytes(index_bytes[pos..pos + 8].try_into().unwrap());
            pos += 8;
            let length = u32::from_le_bytes(index_bytes[pos..pos + 4].try_into().unwrap());
            pos += 4;
            index.insert(key, (offset, length));
        }

        if let Ok(meta) = std::fs::metadata(path) {
            data_end = meta.len().max(HEADER_SIZE);
        }

        (index, data_end)
    }

    fn write_binary_index(index: &AHashMap<String, (u64, u32)>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(index.len() * 64 + 4);
        buf.extend_from_slice(&(index.len() as u32).to_le_bytes());
        for (k, (off, len)) in index {
            buf.extend_from_slice(&(k.len() as u32).to_le_bytes());
            buf.extend_from_slice(k.as_bytes());
            buf.extend_from_slice(&off.to_le_bytes());
            buf.extend_from_slice(&len.to_le_bytes());
        }
        buf
    }
}

impl<'a> Db<'a> for CandidateDatabase {
    type InitArgs = u8;

    fn new(path: PathBuf, _args: Self::InitArgs) -> Self {
        let index_path = index_path(&path);
        let mut file = Self::open_data_file(&path).expect("Failed to open database file");
        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);

        let (index, mut data_end) = if file_len >= HEADER_SIZE {
            Self::load_index(&path, &index_path)
        } else {
            let _ = std::fs::remove_file(&index_path);
            (AHashMap::new(), HEADER_SIZE)
        };

        if file_len < HEADER_SIZE {
            file.write_all(MAGIC).expect("Failed to write magic");
            file.write_all(&0u64.to_le_bytes()).expect("Failed to write placeholder");
            file.write_all(&[0u8; 4]).expect("Failed to write reserved");
        } else if index.is_empty() {
            data_end = file_len;
        }

        file.seek(SeekFrom::End(0)).expect("Failed to seek");
        let writer = BufWriter::with_capacity(BUF_SIZE, file);

        Self {
            path: path.clone(),
            index_path,
            writer: RefCell::new(writer),
            mmap: RefCell::new(None),
            index,
            data_end,
            dirty: RefCell::new(false)
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

        let mut w = self.writer.borrow_mut();
        w.seek(SeekFrom::Start(self.data_end))
            .expect("Failed to seek");
        w.write_all(&key_len.to_le_bytes())
            .expect("Failed to write key len");
        w.write_all(key_bytes).expect("Failed to write key");
        w.write_all(&json_len.to_le_bytes())
            .expect("Failed to write json len");
        w.write_all(&json).expect("Failed to write json");

        let json_offset = self.data_end + 4 + (key_len as u64) + 4;
        self.index.insert(key_str, (json_offset, json_len));
        self.data_end = json_offset + (json_len as u64);
        *self.dirty.borrow_mut() = true;
    }

    fn remove(&mut self, key: &Self::Key) -> Option<Self::Data> {
        self.writer.borrow_mut().flush().expect("Failed to flush before read");
        let key_str = key_str(key);
        let &(json_offset, json_len) = self.index.get(&key_str)?;
        let mut buf = vec![0u8; json_len as usize];
        let mut file = File::open(&self.path).ok()?;
        file.seek(SeekFrom::Start(json_offset)).ok()?;
        file.read_exact(&mut buf).ok()?;
        let value: StoredType = serde_json::from_slice(&buf).ok()?;

        let key_bytes = key_str.as_bytes();
        let key_len = key_bytes.len() as u32;
        let json_len = 0u32;

        let mut w = self.writer.borrow_mut();
        w.seek(SeekFrom::Start(self.data_end))
            .expect("Failed to seek");
        w.write_all(&key_len.to_le_bytes())
            .expect("Failed to write key len");
        w.write_all(key_bytes).expect("Failed to write key");
        w.write_all(&json_len.to_le_bytes())
            .expect("Failed to write tombstone");

        self.index.remove(&key_str);
        self.data_end += 4 + (key_len as u64) + 4;
        *self.dirty.borrow_mut() = true;

        Some(value)
    }
}

impl<'a> DbReader<'a> for CandidateDatabase {
    type Data = StoredType<'a>;
    type Key = DatabaseKey<'a>;

    fn read(&self, key: &Self::Key) -> Option<Self::Data> {
        let _ = self.writer.borrow_mut().flush();
        let key_str = key_str(key);
        let &(json_offset, json_len) = self.index.get(&key_str)?;
        if json_len == 0 {
            return None;
        }
        let json_offset = json_offset as usize;
        let json_len = json_len as usize;

        let mut mmap_guard = self.mmap.borrow_mut();
        if *self.dirty.borrow() {
            *mmap_guard = None;
        }
        if mmap_guard.is_none() {
            let file = File::open(&self.path).ok()?;
            let mmap = unsafe { Mmap::map(&file).ok()? };
            if json_offset + json_len <= mmap.len() {
                *mmap_guard = Some(mmap);
                *self.dirty.borrow_mut() = false;
            }
        }

        if let Some(ref mmap) = *mmap_guard {
            if json_offset + json_len <= mmap.len() {
                return serde_json::from_slice(&mmap[json_offset..json_offset + json_len]).ok();
            }
        }

        let mut buf = vec![0u8; json_len];
        let mut file = File::open(&self.path).ok()?;
        file.seek(SeekFrom::Start(json_offset as u64)).ok()?;
        file.read_exact(&mut buf).ok()?;
        serde_json::from_slice(&buf).ok()
    }
}

impl Drop for CandidateDatabase {
    fn drop(&mut self) {
        self.writer.borrow_mut().flush().expect("Failed to flush");
        let index_bytes = Self::write_binary_index(&self.index);
        std::fs::write(&self.index_path, &index_bytes).expect("Failed to write index");
    }
}
