use criterion::{criterion_group, criterion_main};

fn expand_u8(c: &mut criterion::Criterion) {
    c.bench_function("expand 8-bit", move |b| {
        b.iter(|| {
            for e in 0..=255 {
                criterion::black_box(srgb::gamma::expand_u8(e));
            }
        });
    });
}

fn compress_u8(c: &mut criterion::Criterion) {
    c.bench_function("compress 8-bit", move |b| {
        b.iter(|| {
            for s in 0..=255 {
                criterion::black_box(srgb::gamma::compress_u8(
                    s as f32 / 255.0,
                ));
            }
        });
    });
}

fn expand_normalised(c: &mut criterion::Criterion) {
    c.bench_function("expand normalised", move |b| {
        b.iter(|| {
            for e in 0..=255 {
                criterion::black_box(srgb::gamma::expand_normalised(
                    e as f32 / 255.0,
                ));
            }
        });
    });
}

fn compress_normalised(c: &mut criterion::Criterion) {
    c.bench_function("compress normalised", move |b| {
        b.iter(|| {
            for s in 0..=255 {
                criterion::black_box(srgb::gamma::compress_normalised(
                    s as f32 / 255.0,
                ));
            }
        });
    });
}

criterion_group!(
    benches,
    expand_u8,
    compress_u8,
    expand_normalised,
    compress_normalised,
);
criterion_main!(benches);
