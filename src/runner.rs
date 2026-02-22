use std::path::PathBuf;

use itertools::Itertools;

use crate::{
    database::{DbReader, DbWriter},
    type_generator::TypeGenerator,
    types::{DatabaseKey, StoredType}
};

pub const ITEMS: usize = 1_000;
pub const REMOVES: usize = 500;
const RELOADS: usize = 100;

pub struct BenchInput<'a, TG> {
    #[allow(dead_code)]
    tg:                 TG,
    pub indexed_values: Vec<(DatabaseKey<'a>, StoredType<'a>)>,
    pub path:           PathBuf,
    pub removes:        Vec<(DatabaseKey<'a>, StoredType<'a>)>,
    pub removes_len:    usize
}

pub fn setup_bench<'a, TG>(
    mut tg: TG,
    path: PathBuf,
    items: usize,
    removes: usize
) -> BenchInput<'a, TG>
where
    TG: TypeGenerator<'a, Key = DatabaseKey<'a>, Data = StoredType<'a>> + 'a
{
    let _ = std::fs::remove_file(&path);
    let mut indexed_values = Vec::with_capacity(items);

    for _ in 0..items {
        let key = tg.generate_key();
        let data = tg.generate_data();

        if indexed_values.iter().any(|(k, _)| k == &key) {
            continue;
        }

        indexed_values.push((key, data));
    }

    BenchInput {
        tg,
        indexed_values,
        path,
        removes: Vec::with_capacity(removes),
        removes_len: removes
    }
}

pub fn run_bench<'a, DB, TG>(db: impl Fn(PathBuf) -> DB, input: &mut BenchInput<'a, TG>)
where
    DB: 'a,
    DB: DbWriter<'a, Key = DatabaseKey<'a>, Data = StoredType<'a>>,
    DB: DbReader<'a, Key = DatabaseKey<'a>, Data = StoredType<'a>>
{
    let mut current_db = db(input.path.clone());

    for (key, data) in input.indexed_values.iter() {
        current_db.write(key, data);
    }

    for (k, expected) in input.indexed_values.iter() {
        let stored = current_db.read(k).expect("missing entry from db");
        assert_eq!(&stored, expected);
    }
    drop(current_db);

    for _ in 0..RELOADS {
        let current_db = db(input.path.clone());
        for (k, expected) in input.indexed_values.iter() {
            let stored = current_db.read(k).expect("missing entry from db");
            assert_eq!(&stored, expected);
        }
        drop(current_db);
    }

    let mut current_db = db(input.path.clone());
    let items = input.indexed_values.len();
    let removed = &mut input.removes;

    for remove_idx in (0..)
        .map(|_| rand::random_range(0..items))
        .unique()
        .take(input.removes_len)
        .sorted_by(|a, b| Ord::cmp(&b, &a))
    {
        let (k, v) = input.indexed_values.remove(remove_idx);
        let remove_v = current_db.remove(&k).unwrap();

        assert_eq!(v, remove_v);

        removed.push((k, v));
    }

    drop(current_db);

    let current_db = db(input.path.clone());

    for (k, expected) in input.indexed_values.iter() {
        let stored = current_db.read(k).expect("missing entry from db");
        assert_eq!(&stored, expected);
    }

    for (k, _) in removed.iter() {
        assert!(current_db.read(k).is_none());
    }

    drop(current_db);

    // File corruption test: verify data is actually read from file, not static
    // storage
    std::fs::write(&input.path, "CORRUPTED").expect("Failed to corrupt test file");

    let current_db = db(input.path.clone());

    for (k, _) in input.indexed_values.iter() {
        assert!(
            current_db.read(k).is_none(),
            "Read succeeded after file corruption - static/global storage detected!"
        );
    }
}

pub fn teardown_bench<TG>(input: &BenchInput<'_, TG>) {
    let _ = std::fs::remove_file(&input.path);
}

pub fn runner<'a, DB, TG>(db: impl Fn(PathBuf) -> DB, tg: TG)
where
    DB: 'a,
    DB: DbWriter<'a, Key = DatabaseKey<'a>, Data = StoredType<'a>>,
    DB: DbReader<'a, Key = DatabaseKey<'a>, Data = StoredType<'a>>,
    TG: TypeGenerator<'a, Key = DatabaseKey<'a>, Data = StoredType<'a>> + 'a
{
    let path = PathBuf::from("./testdb");
    let mut input = setup_bench(tg, path, ITEMS, REMOVES);

    run_bench(&db, &mut input);

    teardown_bench(&input);
}
