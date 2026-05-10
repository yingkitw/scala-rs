//! Throughput smoke bench: typecheck + interpret a small recursive program.
//!
//! Run: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_fib_interp(c: &mut Criterion) {
    let src = r#"def fib(n: Int): Int = if (n <= 1) n else fib(n - 1) + fib(n - 2)
fib(14)
"#;

    c.bench_function("fib_14_typecheck_then_run", |b| {
        b.iter(|| {
            let _ = scala::typecheck_then_run(black_box(src)).unwrap();
        });
    });
}

criterion_group!(benches, bench_fib_interp);
criterion_main!(benches);
