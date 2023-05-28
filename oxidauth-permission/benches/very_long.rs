use criterion::{black_box, criterion_group, criterion_main, Criterion};

use oxidauth_permission::tokens::parse::parse;

pub fn parse_very_very_long(c: &mut Criterion) {
    let input = "oxidauth.admin_web.super_admin.tenant.special:tenant.c1fd240c-4160-459e-a184-084ddba63a94.users.c2fd240c-4160-459e-a184-084ddba63a94.relationship.c2fd240c-4160-459e-a184-084ddba63a94:read_manage_write_all.*";

    c.bench_function("parse:very_long", |b| {
        b.iter(|| {
            parse(black_box(input)).unwrap();
        })
    });
}

criterion_group!(very_long, parse_very_very_long);

criterion_main!(very_long);
