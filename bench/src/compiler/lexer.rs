use anyhow::Result;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use raccoon_compiler::Lexer;

fn tokenize(c: &mut Criterion) {
    c.bench_function("tokenize", |b| {
        b.iter_batched(
            || {
                // TODO(appcypher): Use lexer proptest strategies.
                "0x1234_5678__9abc_def0"
            },
            |code| Lexer::tokenize(code).collect::<Result<Vec<_>>>(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, tokenize);

criterion_main!(benches);
