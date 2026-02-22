use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, SamplingMode, criterion_group, criterion_main};
use takehome_rust_optimize::{
    database::Db,
    runner::{run_bench, setup_bench, teardown_bench},
    type_generator::TypeGenerator,
    user_solution::{database::CandidateDatabase, type_generator::CandidateTypeGen}
};

// (items, removes, sample_size)
const SIZES: &[(usize, usize, usize)] =
    &[(100, 50, 100), (500, 250, 50), (1000, 500, 25), (10_000, 5_000, 10)];

fn candidate_db_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("candidate_db");
    group.sampling_mode(SamplingMode::Flat);

    for &(items, removes, sample_size) in SIZES {
        group.sample_size(sample_size);
        group.bench_with_input(
            BenchmarkId::new("items", items),
            &(items, removes),
            |b, &(items, removes)| {
                b.iter_custom(|iters| {
                    let mut total = std::time::Duration::ZERO;

                    for _ in 0..iters {
                        let path = PathBuf::from(format!("./bench_testdb"));
                        let args = CandidateTypeGen::args();
                        let mut input =
                            setup_bench(CandidateTypeGen::new(args), path, items, removes);

                        let args = CandidateDatabase::args();
                        let start = std::time::Instant::now();
                        run_bench(|p| CandidateDatabase::new(p, args), &mut input);
                        total += start.elapsed();

                        teardown_bench(&input);
                    }

                    total
                });
            }
        );
    }

    group.finish();
}

// NOTE: default db isn't run  due to excessive runtime.
criterion_group!(benches, candidate_db_benchmark);
criterion_main!(benches);
