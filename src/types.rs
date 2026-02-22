use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::database::AsStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DatabaseKey<'a> {
    pub e:           String,
    pub stored_type: String,
    pub _p:          PhantomData<&'a u8>
}

impl AsStr for DatabaseKey<'_> {
    fn as_str(&self) -> impl AsRef<str> {
        format!("{}:{}", self.e, self.stored_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoredType<'a> {
    pub entry: String,
    pub one:   u128,
    pub data:  Vec<RandomNestedStructure<'a>>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandomNestedStructure<'a> {
    pub val:   String,
    pub score: u64,
    pub _p:    PhantomData<&'a u8>
}
