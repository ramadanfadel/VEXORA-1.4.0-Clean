use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    let dir = std::env::temp_dir().join("vexora_bench_hyperscan");
    let _ = fs::create_dir_all(&dir);
    let payload = vec![7u8; 64 * 1024];
    for i in 0..40 {
        let _ = fs::write(dir.join(format!("file-{i}.bin")), &payload);
    }
    dir
}

fn bench_hash(c: &mut Criterion) {
    let dir = fixture();
    let first = dir.join("file-0.bin");
    c.bench_function("blake3_hash_64kb", |b| {
        b.iter(|| black_box(vexora_lib::hyperscan::hash_file(black_box(&first))))
    });
}

criterion_group!(benches, bench_hash);
criterion_main!(benches);
