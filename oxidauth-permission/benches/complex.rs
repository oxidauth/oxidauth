use criterion::{black_box, criterion_group, criterion_main, Criterion};

use oxidauth_permission::tokens::parse::parse;

pub fn parse_complex(c: &mut Criterion) {
    let input = "oxidauth:users.34645u485464j64u46u4.profile:read";

    c.bench_function("parse:complex", |b| {
        b.iter(|| {
            parse(black_box(input)).unwrap();
        })
    });
}

criterion_group!(complex, parse_complex);

criterion_main!(complex);
