use std::marker::PhantomData;

use rand::{
    Rng,
    distr::{Alphanumeric, SampleString}
};

use crate::{
    type_generator::{RANGE_SIZE, TypeGenerator},
    types::{DatabaseKey, RandomNestedStructure, StoredType}
};

pub struct DefaultGenerator;

impl<'a> TypeGenerator<'a> for DefaultGenerator {
    type Data = StoredType<'a>;
    type InitArgs = u8;
    type Key = DatabaseKey<'a>;

    fn new(_: Self::InitArgs) -> Self {
        DefaultGenerator
    }

    fn args() -> Self::InitArgs {
        1
    }

    fn generate_key(&mut self) -> Self::Key {
        let mut rng = rand::rng();
        let string_size = rng.random_range(RANGE_SIZE);

        let s1 = Alphanumeric.sample_string(&mut rng, string_size);

        let string_size = rng.random_range(RANGE_SIZE);
        let s2 = Alphanumeric.sample_string(&mut rng, string_size);

        DatabaseKey { e: s1, stored_type: s2, _p: PhantomData }
    }

    fn generate_data(&mut self) -> Self::Data {
        let mut rng = rand::rng();
        let string_size = rng.random_range(RANGE_SIZE);
        let entry = Alphanumeric.sample_string(&mut rng, string_size);

        let data = (0usize..rng.random_range(RANGE_SIZE))
            .map(|_| RandomNestedStructure {
                val:   {
                    let string_size = rng.random_range(RANGE_SIZE);
                    Alphanumeric.sample_string(&mut rng, string_size)
                },
                score: rng.random(),
                _p:    PhantomData
            })
            .collect();

        StoredType { entry, one: rng.random(), data }
    }
}
