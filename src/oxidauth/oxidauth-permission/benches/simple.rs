use criterion::{black_box, criterion_group, criterion_main, Criterion};

use oxidauth_permission::tokens::parse::parse;

pub fn parse_simple(c: &mut Criterion) {
    let input = "*:*:*";

    c.bench_function("parse:simple", |b| {
        b.iter(|| {
            parse(black_box(input)).unwrap();
        })
    });
}

criterion_group!(simple, parse_simple);

criterion_main!(simple);
