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

fn compress_u8_precise(c: &mut criterion::Criterion) {
    c.bench_function("compress 8-bit precise", move |b| {
        b.iter(|| {
            for s in 0..=255 {
                criterion::black_box(srgb::gamma::compress_u8_precise(
                    s as f32 / 255.0,
                ));
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

fn expand_rec709_8bit(c: &mut criterion::Criterion) {
    c.bench_function("expand 8-bit Rec.709", move |b| {
        b.iter(|| {
            for e in 16..=235 {
                criterion::black_box(srgb::gamma::expand_rec709_8bit(e));
            }
        });
    });
}

fn compress_rec709_8bit(c: &mut criterion::Criterion) {
    c.bench_function("compress 8-bit Rec.709", move |b| {
        b.iter(|| {
            for s in 0..=219 {
                criterion::black_box(srgb::gamma::compress_rec709_8bit(
                    s as f32 / 219.0,
                ));
            }
        });
    });
}

fn expand_rec709_10bit(c: &mut criterion::Criterion) {
    c.bench_function("expand 10-bit Rec.709", move |b| {
        b.iter(|| {
            for e in 64..=940 {
                criterion::black_box(srgb::gamma::expand_rec709_10bit(e));
            }
        });
    });
}

fn compress_rec709_10bit(c: &mut criterion::Criterion) {
    c.bench_function("compress 10-bit Rec.709", move |b| {
        b.iter(|| {
            for s in 0..=876 {
                criterion::black_box(srgb::gamma::compress_rec709_10bit(
                    s as f32 / 876.0,
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
    compress_u8_precise,
    compress_u8,
    expand_rec709_8bit,
    compress_rec709_8bit,
    expand_rec709_10bit,
    compress_rec709_10bit,
    expand_normalised,
    compress_normalised,
);
criterion_main!(benches);
