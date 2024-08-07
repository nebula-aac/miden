use std::time::Duration;

use assembly::Assembler;
use criterion::{criterion_group, criterion_main, Criterion};
use stdlib::StdLibrary;

fn program_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("program_compilation");
    group.measurement_time(Duration::from_secs(10));

    let stdlib = StdLibrary::default();
    group.bench_function("sha256", |bench| {
        let source = "
            use.std::crypto::hashes::sha256

            begin
                exec.sha256::hash_2to1
            end";
        bench.iter(|| {
            let mut assembler = Assembler::default();
            assembler.add_library(&stdlib).expect("failed to load stdlib");
            assembler.assemble_program(source).expect("Failed to compile test source.")
        });
    });

    group.finish();
}

criterion_group!(sha256_group, program_compilation);
criterion_main!(sha256_group);
