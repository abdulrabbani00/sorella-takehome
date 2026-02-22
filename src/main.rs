use takehome_rust_optimize::{
    database::Db,
    default_solution::{database::DefaultDatabase, type_generator::DefaultGenerator},
    runner::runner,
    type_generator::TypeGenerator,
    types::{DatabaseKey, StoredType},
    user_solution::{database::CandidateDatabase, type_generator::CandidateTypeGen}
};

fn main() {
    println!("running default");
    runner(
        |path| {
            let init = DefaultDatabase::<DatabaseKey, StoredType>::args();
            DefaultDatabase::new(path, init)
        },
        DefaultGenerator
    );

    println!("running candidate");
    runner(
        |path| {
            let init = CandidateDatabase::args();
            CandidateDatabase::new(path, init)
        },
        {
            let args = CandidateTypeGen::args();
            CandidateTypeGen::new(args)
        }
    );
}
