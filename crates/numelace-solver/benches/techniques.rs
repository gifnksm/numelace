//! Micro-benchmarks for individual technique applications.
//!
//! This benchmark suite measures the cost of calling `apply` for each technique
//! on representative puzzle states.
//!
//! # Running
//!
//! ```sh
//! cargo bench --bench techniques
//! ```

use std::hint;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use numelace_core::{CandidateGrid, Digit, Position};
use numelace_solver::technique::{
    HiddenSingle, LockedCandidates, NakedSingle, Technique as _, TechniqueGrid,
};

fn naked_single_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let target = Position::new(0, 0);
    for digit in Digit::ALL {
        if digit != Digit::D1 {
            grid.remove_candidate(target, digit);
        }
    }
    TechniqueGrid::from(grid)
}

fn hidden_single_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let target = Position::new(1, 0);
    for pos in Position::ROWS[0] {
        if pos != target {
            grid.remove_candidate(pos, Digit::D2);
        }
    }
    TechniqueGrid::from(grid)
}

fn locked_candidates_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    for pos in Position::BOXES[0] {
        if pos.y() != 0 {
            grid.remove_candidate(pos, Digit::D5);
        }
    }
    TechniqueGrid::from(grid)
}

fn bench_naked_single_apply(c: &mut Criterion) {
    let puzzles = [
        ("naked_single", naked_single_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = NakedSingle::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("naked_single_apply", param),
            &grid,
            |b, grid| {
                b.iter_batched_ref(
                    || hint::black_box(grid.clone()),
                    |grid| {
                        let changed = technique.apply(grid).unwrap();
                        hint::black_box(changed)
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_hidden_single_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_single", hidden_single_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = HiddenSingle::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("hidden_single_apply", param),
            &grid,
            |b, grid| {
                b.iter_batched_ref(
                    || hint::black_box(grid.clone()),
                    |grid| {
                        let changed = technique.apply(grid).unwrap();
                        hint::black_box(changed)
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_locked_candidates_apply(c: &mut Criterion) {
    let puzzles = [
        ("locked_candidates", locked_candidates_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = LockedCandidates::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("locked_candidates_apply", param),
            &grid,
            |b, grid| {
                b.iter_batched_ref(
                    || hint::black_box(grid.clone()),
                    |grid| {
                        let changed = technique.apply(grid).unwrap();
                        hint::black_box(changed)
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(
    benches,
    bench_naked_single_apply,
    bench_hidden_single_apply,
    bench_locked_candidates_apply,
);
criterion_main!(benches);
