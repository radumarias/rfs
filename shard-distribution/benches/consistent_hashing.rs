use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};

use shard_distribution::consistent_hashing::ConsistentHashing;
use shard_distribution::shard_distribution::ShardDistribution;

fn consistent_hashing(c: &mut Criterion) {
    let mut nodes = vec![];
    for i in 0..1_000 {
        nodes.push((format!("node{}", i), i as f64));
    }
    let nodes: HashMap<String, f64> = nodes.into_iter().collect();
    c.bench_function("create", |b| {
        b.iter(|| black_box(ConsistentHashing::new(nodes.clone(), 5)))
    });

    let mut hasher = ConsistentHashing::new(nodes, 5);
    c.bench_function("get_nodes", |b| {
        b.iter(|| {
            let nodes = hasher.distribute(
                format!("key{}", thread_rng().gen_range(0..10_000)),
                Some(10),
                Some(10_f64),
            );
            black_box(nodes)
        })
    });

    c.bench_function("add", |b| {
        b.iter(|| {
            let node = format!("node{}", thread_rng().gen_range(1_000..2_000));
            hasher.add(node.clone(), 10_f64);
            black_box(node)
        })
    });

    c.bench_function("update", |b| {
        b.iter(|| {
            let node = format!("node{}", thread_rng().gen_range(0..1_000));
            hasher.update(node.clone(), 10_f64);
            black_box(node)
        })
    });

    c.bench_function("remove", |b| {
        b.iter(|| {
            let node = format!("node{}", thread_rng().gen_range(0..2_000));
            hasher.remove(node.clone());
            black_box(node)
        })
    });
}

criterion_group!(benches, consistent_hashing);
criterion_main!(benches);
