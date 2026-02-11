//! Benchmarks for backtracking-related functions.
//!
//! This benchmark suite measures the performance of backtracking operations,
//! particularly the `find_best_assumption` function which selects the next cell
//! to try when backtracking is needed.
//!
//! # Test Data
//!
//! All puzzles are generated from the same solution (seed: `c1d44bd6afaf8af64f126546884e19298acbdc33c3924a28136715de946ef3f1`)
//! with varying numbers of given cells:
//!
//! - **empty** (0 given): Completely empty grid
//! - **sparse** (23 given): Minimal puzzle with few clues
//! - **mid** (40 given): Medium difficulty with moderate clues
//! - **dense** (60 given): Nearly solved puzzle with many clues
//!
//! Each puzzle is preprocessed with `NakedSingle` constraint propagation to simulate
//! realistic solver state.
//!
//! # Running
//!
//! ```sh
//! cargo bench --bench backtrack
//! ```

use std::{hint, str::FromStr as _};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use numelace_core::{CandidateGrid, DigitGrid};
use numelace_solver::{
    backtrack,
    technique::{NakedSingle, Technique as _, TechniqueGrid},
};

// Problems generated from seed: c1d44bd6afaf8af64f126546884e19298acbdc33c3924a28136715de946ef3f1
// using PuzzleGenerator with fundamental techniques (NakedSingle + HiddenSingle).
// empty: 0 given
const EMPTY_PROBLEM: &str =
    ".................................................................................";
// sparse: 23 given
const SPARSE_PROBLEM: &str =
    "...36..4..9.....2...67..1..5....987..31..............4..8...65.67.....3......3..2";
// mid: 63 given
const MID_PROBLEM: &str =
    "...36.94..9....526.467..1..5..2.98719318.......75...94.18.2.65.67..51.3...9..3.12";
// dense: 60 given
const DENSE_PROBLEM: &str =
    "18536294779.148.26.4679518.5.4239871.31..42658..51.3.4..8.2.65967.9..438.59683.12";
// solution 81 given
const _SOLUTION: &str =
    "185362947793148526246795183564239871931874265827516394318427659672951438459683712";

fn bench_find_best_assumption(c: &mut Criterion) {
    let puzzles = [
        ("empty", EMPTY_PROBLEM),
        ("sparse", SPARSE_PROBLEM),
        ("mid", MID_PROBLEM),
        ("dense", DENSE_PROBLEM),
        // do not include solution here, find_best_assumption will panic on a fully solved grid
        // ("solution", SOLUTION),
    ];

    for (param, grid) in puzzles {
        let grid = DigitGrid::from_str(grid).unwrap();
        let given = grid.iter().filter(|o| o.is_some()).count();

        let mut grid = TechniqueGrid::from(CandidateGrid::from(grid));
        NakedSingle::new().apply(&mut grid).unwrap();
        c.bench_with_input(
            BenchmarkId::new("find_best_assumption", format!("{param}_{given}")),
            &grid,
            |b, grid| {
                b.iter_batched_ref(
                    || hint::black_box(grid.clone()),
                    |grid| {
                        let result = backtrack::find_best_assumption(grid);
                        hint::black_box(result)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(benches, bench_find_best_assumption);
criterion_main!(benches);
