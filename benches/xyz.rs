use criterion::{criterion_group, criterion_main};

#[inline(always)]
fn for_tripples(f: impl Fn([f32; 3]) -> [f32; 3]) {
    for i in 0..(1 << 12) {
        let r = (i >> 8) as f32 / 15.0;
        let g = ((i >> 4) & 15) as f32 / 15.0;
        let b = (i & 15) as f32 / 15.0;
        criterion::black_box(&f([r, g, b]));
    }
}

fn xyz_from_linear(c: &mut criterion::Criterion) {
    c.bench_function("Linear → XYZ", move |b| {
        b.iter(|| for_tripples(srgb::xyz::xyz_from_linear))
    });
}

fn linear_from_xyz(c: &mut criterion::Criterion) {
    c.bench_function("XYZ → Linear", move |b| {
        b.iter(|| for_tripples(srgb::xyz::xyz_from_linear))
    });
}

criterion_group!(benches, xyz_from_linear, linear_from_xyz,);
criterion_main!(benches);
