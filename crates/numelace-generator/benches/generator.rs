//! Benchmarks for Sudoku puzzle generation.
//!
//! This benchmark suite measures the performance of puzzle generation using
//! `PuzzleGenerator` with fundamental and basic techniques.
//!
//! # Benchmarks
//!
//! - **`generator_fundamental`**: Generates puzzles using fundamental techniques
//!   (`NakedSingle` + `HiddenSingle`). Measures the complete generation process
//!   including solution generation and cell removal.
//! - **`generator_basic`**: Generates puzzles using basic techniques
//!   (fundamental + `Locked Candidates`).
//!
//! # Test Data
//!
//! Uses three fixed seeds to ensure reproducibility while testing multiple cases:
//!
//! - **`seed_0`**: `c1d44bd6afaf8af64f126546884e19298acbdc33c3924a28136715de946ef3f1`
//! - **`seed_1`**: `a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3`
//! - **`seed_2`**: `1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef`
//!
//! Each seed produces a different puzzle, allowing measurement across various cases
//! while maintaining reproducibility.
//!
//! # Running
//!
//! ```sh
//! cargo bench --bench generator
//! ```

use std::{hint, str::FromStr as _, time::Duration};

use criterion::{
    BatchSize, BenchmarkId, Criterion, PlottingBackend, criterion_group, criterion_main,
};
use numelace_generator::{PuzzleGenerator, PuzzleSeed};
use numelace_solver::{TechniqueSolver, technique};

const SEEDS: [&str; 3] = [
    "c1d44bd6afaf8af64f126546884e19298acbdc33c3924a28136715de946ef3f1",
    "a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3",
    "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
];

fn bench_generator_fundamental(c: &mut Criterion) {
    let solver = TechniqueSolver::new(technique::fundamental_techniques());
    let generator = PuzzleGenerator::new(&solver);

    for (i, seed) in SEEDS.into_iter().enumerate() {
        let seed = PuzzleSeed::from_str(seed).unwrap();
        c.bench_with_input(
            BenchmarkId::new("generator_fundamental", format!("seed_{i}")),
            &seed,
            |b, seed| {
                b.iter_batched(
                    || hint::black_box(*seed),
                    |seed| generator.generate_with_seed(seed),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_generator_basic(c: &mut Criterion) {
    let solver = TechniqueSolver::new(technique::basic_techniques());
    let generator = PuzzleGenerator::new(&solver);

    for (i, seed) in SEEDS.into_iter().enumerate() {
        let seed = PuzzleSeed::from_str(seed).unwrap();
        c.bench_with_input(
            BenchmarkId::new("generator_basic", format!("seed_{i}")),
            &seed,
            |b, seed| {
                b.iter_batched(
                    || hint::black_box(*seed),
                    |seed| generator.generate_with_seed(seed),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(
    name = benches;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters)
            .measurement_time(Duration::from_secs(12));
    targets =
        bench_generator_fundamental,
        bench_generator_basic
);
criterion_main!(benches);
