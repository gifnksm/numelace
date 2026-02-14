//! End-to-end benchmarks for Sudoku solvers.
//!
//! This benchmark suite measures the performance of complete puzzle solving,
//! including both `TechniqueSolver` (technique-based solving) and `BacktrackSolver`
//! (backtracking with techniques).
//!
//! # Benchmarks
//!
//! - **`technique_solver_fundamental`**: Solves puzzles using only fundamental techniques
//!   (`NakedSingle` + `HiddenSingle`). Only includes puzzles solvable by these techniques.
//! - **`backtrack_solver_fundamental`**: Solves puzzles using backtracking with fundamental
//!   techniques. Includes all puzzles, even those requiring backtracking. For puzzles
//!   with multiple solutions (`empty`, `ultra_sparse`), limits to first 100 solutions.
//!
//! # Test Data
//!
//! All puzzles are generated from the same solution (seed: `c1d44bd6afaf8af64f126546884e19298acbdc33c3924a28136715de946ef3f1`)
//! with varying numbers of given cells:
//!
//! - **`empty`** (0 given): Completely empty grid (multiple solutions)
//! - **`ultra_sparse`** (16 given): Very few clues (multiple solutions)
//! - **`sparse`** (23 given): Minimal puzzle with few clues (unique solution)
//! - **`mid`** (40 given): Medium difficulty with moderate clues (unique solution)
//! - **`dense`** (60 given): Nearly solved puzzle with many clues (unique solution)
//! - **`solution`** (81 given): Fully solved grid
//!
//! # Running
//!
//! ```sh
//! cargo bench --bench solver
//! ```

use std::{hint, str::FromStr as _};

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use numelace_core::DigitGrid;
use numelace_solver::{
    BacktrackSolver, TechniqueSolver,
    technique::{TechniqueGrid, basic_techniques},
};

// Problems generated from seed: c1d44bd6afaf8af64f126546884e19298acbdc33c3924a28136715de946ef3f1
// using PuzzleGenerator with fundamental techniques (NakedSingle + HiddenSingle).
// empty: 0 given (need backtracking)
const EMPTY_PROBLEM: &str =
    ".................................................................................";
// ultra-sparse: 16 given (need backtracking)
const ULTRA_SPARSE_PROBLEM: &str =
    "....6..4..9.....2...67.....5....987..................4..8....5.67................";
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
const SOLUTION: &str =
    "185362947793148526246795183564239871931874265827516394318427659672951438459683712";
// locked-candidates: 25 given (uses Locked Candidates 3 times)
// seed: 723b41d8a9fa67daa2622c9ca4137222a39fbbbe9908cc8d2cd544ec0a8f6826
const LOCKED_CANDIDATES_PROBLEM: &str =
    "7...134...8...2...63.5.....5..........2..814....3....6.7....61...6...2548.....7..";
const LOCKED_CANDIDATES_SOLUTION: &str =
    "729613485485972361631584927597146832362758149148329576273495618916837254854261793";

fn bench_technique_solver_fundamental(c: &mut Criterion) {
    let puzzles = [
        ("sparse", SPARSE_PROBLEM),
        ("mid", MID_PROBLEM),
        ("dense", DENSE_PROBLEM),
        ("solution", SOLUTION),
    ];

    let solver = TechniqueSolver::with_fundamental_techniques();

    for (param, grid) in puzzles {
        let grid = DigitGrid::from_str(grid).unwrap();
        let given = grid.iter().filter(|o| o.is_some()).count();
        let grid = TechniqueGrid::from(grid);
        c.bench_with_input(
            BenchmarkId::new("technique_solver_fundamental", format!("{param}_{given}")),
            &grid,
            |b, grid| {
                // ensure that the puzzle is solvable by fundamental techniques
                let mut test_grid = grid.clone();
                let (puzzle_solved, _stats) = solver.solve(&mut test_grid).unwrap();
                assert!(
                    puzzle_solved,
                    "puzzle should be solvable by fundamental techniques"
                );
                assert_eq!(test_grid.to_digit_grid().to_string(), SOLUTION);

                b.iter_batched_ref(
                    || hint::black_box(grid.clone()),
                    |grid| solver.solve(grid).unwrap(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_technique_solver_basic(c: &mut Criterion) {
    let puzzles = [(
        "locked_candidates",
        LOCKED_CANDIDATES_PROBLEM,
        LOCKED_CANDIDATES_SOLUTION,
    )];

    let solver = TechniqueSolver::new(basic_techniques());

    for (param, grid, expected_solution) in puzzles {
        let grid = DigitGrid::from_str(grid).unwrap();
        let given = grid.iter().filter(|o| o.is_some()).count();
        let grid = TechniqueGrid::from(grid);
        c.bench_with_input(
            BenchmarkId::new("technique_solver_basic", format!("{param}_{given}")),
            &grid,
            |b, grid| {
                // ensure that the puzzle is solvable by basic techniques
                let mut test_grid = grid.clone();
                let (puzzle_solved, _stats) = solver.solve(&mut test_grid).unwrap();
                assert!(
                    puzzle_solved,
                    "puzzle should be solvable by basic techniques"
                );
                assert_eq!(test_grid.to_digit_grid().to_string(), expected_solution);

                b.iter_batched_ref(
                    || hint::black_box(grid.clone()),
                    |grid| solver.solve(grid).unwrap(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn bench_backtrack_solver_fundamental(c: &mut Criterion) {
    let puzzles = [
        ("empty", 100, EMPTY_PROBLEM),
        ("ultra_sparse", 100, ULTRA_SPARSE_PROBLEM),
        ("sparse", 1, SPARSE_PROBLEM),
        ("mid", 1, MID_PROBLEM),
        ("dense", 1, DENSE_PROBLEM),
        ("solution", 1, SOLUTION),
    ];

    let solver = BacktrackSolver::with_fundamental_techniques();

    for (param, expected_solutions, grid) in puzzles {
        let grid = DigitGrid::from_str(grid).unwrap();
        let given = grid.iter().filter(|o| o.is_some()).count();
        let grid = TechniqueGrid::from(grid);
        c.bench_with_input(
            BenchmarkId::new("backtrack_solver_fundamental", format!("{param}_{given}")),
            &grid,
            |b, grid| {
                // ensure that the puzzle has the expected number of solutions
                let test_grid = grid.clone();
                let solutions = solver
                    .solve(test_grid)
                    .unwrap()
                    .take(100) // iterate up to first 100 solutions
                    .collect::<Vec<_>>();
                assert_eq!(
                    solutions.len(),
                    expected_solutions,
                    "{param} should have {expected_solutions} solutions"
                );
                // if unique solution, ensure that it matches the known solution
                if solutions.len() == 1 {
                    assert_eq!(
                        solutions[0].0.candidates().to_digit_grid().to_string(),
                        SOLUTION
                    );
                }

                b.iter_batched(
                    || hint::black_box(grid.clone()),
                    |grid| solver.solve(grid).unwrap().take(100).collect::<Vec<_>>(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(
    benches,
    bench_technique_solver_fundamental,
    bench_technique_solver_basic,
    bench_backtrack_solver_fundamental,
);
criterion_main!(benches);
