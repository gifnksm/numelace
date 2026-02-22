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

use criterion::{
    BatchSize, BenchmarkId, Criterion, PlottingBackend, criterion_group, criterion_main,
};
use numelace_core::{CandidateGrid, Digit, Position};
use numelace_solver::{
    Technique, TechniqueGrid,
    technique::{
        HiddenPair, HiddenQuad, HiddenSingle, HiddenTriple, LockedCandidates, NakedPair, NakedQuad,
        NakedSingle, NakedTriple, XWing,
    },
};

fn bench_apply_cases<T>(
    c: &mut Criterion,
    bench_name: &'static str,
    technique: &T,
    puzzles: &[(&'static str, TechniqueGrid)],
) where
    T: Technique,
{
    for (param, grid) in puzzles {
        c.bench_with_input(BenchmarkId::new(bench_name, param), grid, |b, grid| {
            b.iter_batched_ref(
                || grid.clone(),
                |grid| technique.apply(grid).unwrap(),
                BatchSize::SmallInput,
            );
        });
    }
}

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

fn naked_quad_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(2, 0);
    let pos3 = Position::new(4, 0);
    let pos4 = Position::new(6, 0);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 && digit != Digit::D4 {
            grid.remove_candidate(pos1, digit);
            grid.remove_candidate(pos2, digit);
            grid.remove_candidate(pos3, digit);
            grid.remove_candidate(pos4, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn hidden_quad_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(2, 0);
    let pos3 = Position::new(4, 0);
    let pos4 = Position::new(6, 0);

    for pos in Position::ROWS[0] {
        if pos != pos1 && pos != pos2 && pos != pos3 && pos != pos4 {
            grid.remove_candidate(pos, Digit::D1);
            grid.remove_candidate(pos, Digit::D2);
            grid.remove_candidate(pos, Digit::D3);
            grid.remove_candidate(pos, Digit::D4);
        }
    }

    TechniqueGrid::from(grid)
}

fn x_wing_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let x1 = 1;
    let x2 = 7;
    let y1 = 0;
    let y2 = 4;

    for x in 0..9 {
        if x != x1 && x != x2 {
            grid.remove_candidate(Position::new(x, y1), Digit::D1);
            grid.remove_candidate(Position::new(x, y2), Digit::D1);
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
    bench_apply_cases(c, "naked_single_apply", &technique, &puzzles);
}

fn bench_hidden_single_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_single", hidden_single_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = HiddenSingle::new();
    bench_apply_cases(c, "hidden_single_apply", &technique, &puzzles);
}

fn bench_locked_candidates_apply(c: &mut Criterion) {
    let puzzles = [
        ("locked_candidates", locked_candidates_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = LockedCandidates::new();
    bench_apply_cases(c, "locked_candidates_apply", &technique, &puzzles);
}

fn bench_naked_pair_apply(c: &mut Criterion) {
    let puzzles = [
        ("naked_pair", naked_pair_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = NakedPair::new();
    bench_apply_cases(c, "naked_pair_apply", &technique, &puzzles);
}

fn bench_hidden_pair_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_pair", hidden_pair_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = HiddenPair::new();
    bench_apply_cases(c, "hidden_pair_apply", &technique, &puzzles);
}

fn bench_naked_triple_apply(c: &mut Criterion) {
    let puzzles = [
        ("naked_triple", naked_triple_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = NakedTriple::new();
    bench_apply_cases(c, "naked_triple_apply", &technique, &puzzles);
}

fn bench_hidden_triple_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_triple", hidden_triple_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = HiddenTriple::new();
    bench_apply_cases(c, "hidden_triple_apply", &technique, &puzzles);
}

fn bench_naked_quad_apply(c: &mut Criterion) {
    let puzzles = [
        ("naked_quad", naked_quad_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = NakedQuad::new();
    bench_apply_cases(c, "naked_quad_apply", &technique, &puzzles);
}

fn bench_hidden_quad_apply(c: &mut Criterion) {
    let puzzles = [
        ("hidden_quad", hidden_quad_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = HiddenQuad::new();
    bench_apply_cases(c, "hidden_quad_apply", &technique, &puzzles);
}

fn bench_x_wing_apply(c: &mut Criterion) {
    let puzzles = [("x_wing", x_wing_grid()), ("empty", TechniqueGrid::new())];
    let technique = XWing::new();
    bench_apply_cases(c, "x_wing_apply", &technique, &puzzles);
}

criterion_group!(
    name = benches_naked_single;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_naked_single_apply,
);

criterion_group!(
    name = benches_hidden_single;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_hidden_single_apply,
);

criterion_group!(
    name = benches_locked_candidates;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_locked_candidates_apply,
);

criterion_group!(
    name = benches_naked_pair;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_naked_pair_apply,
);

criterion_group!(
    name = benches_hidden_pair;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_hidden_pair_apply,
);

criterion_group!(
    name = benches_naked_triple;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_naked_triple_apply,
);

criterion_group!(
    name = benches_hidden_triple;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_hidden_triple_apply,
);

criterion_group!(
    name = benches_naked_quad;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_naked_quad_apply,
);

criterion_group!(
    name = benches_hidden_quad;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_hidden_quad_apply,
);

criterion_group!(
    name = benches_x_wing;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_x_wing_apply,
);

criterion_main!(
    benches_naked_single,
    benches_hidden_single,
    benches_locked_candidates,
    benches_naked_pair,
    benches_hidden_pair,
    benches_naked_triple,
    benches_hidden_triple,
    benches_naked_quad,
    benches_hidden_quad,
    benches_x_wing,
);
