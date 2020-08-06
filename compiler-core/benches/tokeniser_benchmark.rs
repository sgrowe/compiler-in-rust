use compiler_core::tokeniser::tokenise;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

pub fn criterion_benchmark(c: &mut Criterion) -> std::io::Result<()> {
    let contents = fs::read_to_string("src/fixtures/example_program.lang")?;

    c.bench_function("tokenise example program", |b| {
        b.iter(|| {
            let tokeniser = tokenise(&contents);

            for token in tokeniser {
                black_box(token.unwrap());
            }
        })
    });

    Ok(())
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
