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
use numelace_solver::{
    TechniqueGrid,
    technique::{
        HiddenPair, HiddenSingle, HiddenTriple, LockedCandidates, NakedPair, NakedSingle,
        NakedTriple, Technique as _,
    },
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

fn naked_pair_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(1, 0);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 {
            grid.remove_candidate(pos1, digit);
            grid.remove_candidate(pos2, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn hidden_pair_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(3, 0);

    for pos in Position::ROWS[0] {
        if pos != pos1 && pos != pos2 {
            grid.remove_candidate(pos, Digit::D1);
            grid.remove_candidate(pos, Digit::D2);
        }
    }

    TechniqueGrid::from(grid)
}

fn naked_triple_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(3, 0);
    let pos3 = Position::new(6, 0);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
            grid.remove_candidate(pos1, digit);
            grid.remove_candidate(pos2, digit);
            grid.remove_candidate(pos3, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn hidden_triple_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(3, 0);
    let pos3 = Position::new(6, 0);

    for pos in Position::ROWS[0] {
        if pos != pos1 && pos != pos2 && pos != pos3 {
            grid.remove_candidate(pos, Digit::D1);
            grid.remove_candidate(pos, Digit::D2);
            grid.remove_candidate(pos, Digit::D3);
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

fn bench_naked_pair_apply(c: &mut Criterion) {
    let puzzles = [
        ("naked_pair", naked_pair_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = NakedPair::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("naked_pair_apply", param),
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

fn bench_hidden_pair_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_pair", hidden_pair_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = HiddenPair::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("hidden_pair_apply", param),
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

fn bench_naked_triple_apply(c: &mut Criterion) {
    let puzzles = [
        ("naked_triple", naked_triple_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = NakedTriple::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("naked_triple_apply", param),
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

fn bench_hidden_triple_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_triple", hidden_triple_grid()),
        ("empty", TechniqueGrid::new()),
    ];

    let technique = HiddenTriple::new();

    for (param, grid) in puzzles {
        c.bench_with_input(
            BenchmarkId::new("hidden_triple_apply", param),
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
    bench_naked_pair_apply,
    bench_hidden_pair_apply,
    bench_naked_triple_apply,
    bench_hidden_triple_apply,
);
criterion_main!(benches);
