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
        HiddenPair, HiddenQuad, HiddenSingle, HiddenTriple, Jellyfish, LockedCandidates, NakedPair,
        NakedQuad, NakedSingle, NakedTriple, RemotePair, Skyscraper, Swordfish, TwoStringKite,
        WxyzWing, XChain, XWing, XyChain, XyzWing, YWing,
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
                |grid| technique.apply_pass(grid).unwrap(),
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

fn skyscraper_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let digit = Digit::D1;
    let col1 = 1;
    let col2 = 7;
    let base_row = 0;
    let col1_roof_row = 3;
    let col2_roof_row = 4;

    for row in 0..9u8 {
        if row != base_row && row != col1_roof_row {
            grid.remove_candidate(Position::new(col1, row), digit);
        }
    }
    for row in 0..9u8 {
        if row != base_row && row != col2_roof_row {
            grid.remove_candidate(Position::new(col2, row), digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn two_string_kite_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let digit = Digit::D1;
    let row = 0;
    let row_box_col = 1;
    let row_other_col = 4;
    let col = 2;
    let col_box_row = 1;
    let col_other_row = 4;

    for x in 0..9u8 {
        if x != row_box_col && x != row_other_col {
            grid.remove_candidate(Position::new(x, row), digit);
        }
    }
    for y in 0..9u8 {
        if y != col_box_row && y != col_other_row {
            grid.remove_candidate(Position::new(col, y), digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn y_wing_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pivot = Position::new(1, 1);
    let wing1 = Position::new(1, 5);
    let wing2 = Position::new(5, 1);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 {
            grid.remove_candidate(pivot, digit);
        }
    }

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D3 {
            grid.remove_candidate(wing1, digit);
        }
    }

    for digit in Digit::ALL {
        if digit != Digit::D2 && digit != Digit::D3 {
            grid.remove_candidate(wing2, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn swordfish_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let digit = Digit::D1;
    let rows = [0u8, 4, 8];
    let cols = [1u8, 4, 7];

    for &row in &rows {
        for x in 0..9u8 {
            if !cols.contains(&x) {
                grid.remove_candidate(Position::new(x, row), digit);
            }
        }
    }

    TechniqueGrid::from(grid)
}

fn jellyfish_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let digit = Digit::D1;
    let rows = [0u8, 2, 5, 8];
    let cols = [1u8, 4, 6, 8];

    for &row in &rows {
        for x in 0..9u8 {
            if !cols.contains(&x) {
                grid.remove_candidate(Position::new(x, row), digit);
            }
        }
    }

    TechniqueGrid::from(grid)
}

fn remote_pair_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let digit1 = Digit::D1;
    let digit2 = Digit::D2;

    let chain_start = Position::new(0, 0);
    let chain_mid1 = Position::new(4, 0);
    let chain_mid2 = Position::new(4, 5);
    let chain_end = Position::new(1, 5);

    for pos in [chain_start, chain_mid1, chain_mid2, chain_end] {
        for digit in Digit::ALL {
            if digit != digit1 && digit != digit2 {
                grid.remove_candidate(pos, digit);
            }
        }
    }

    TechniqueGrid::from(grid)
}

fn x_chain_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let digit = Digit::D1;
    let chain_start = Position::new(0, 0);
    let strong_link_partner = Position::new(4, 0);

    for pos in Position::ROWS[0] {
        if pos != chain_start && pos != strong_link_partner {
            grid.remove_candidate(pos, digit);
        }
    }

    let weak_link_node = Position::new(3, 1);
    let chain_end = Position::new(3, 7);
    for pos in Position::COLUMNS[3] {
        if pos != weak_link_node && pos != chain_end {
            grid.remove_candidate(pos, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn xy_chain_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let start = Position::new(1, 1);
    let mid = Position::new(1, 5);
    let end = Position::new(5, 5);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 {
            grid.remove_candidate(start, digit);
        }
    }

    for digit in Digit::ALL {
        if digit != Digit::D2 && digit != Digit::D3 {
            grid.remove_candidate(mid, digit);
        }
    }

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D3 {
            grid.remove_candidate(end, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn xyz_wing_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pivot = Position::new(1, 1);
    let wing1 = Position::new(1, 2);
    let wing2 = Position::new(2, 1);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
            grid.remove_candidate(pivot, digit);
        }
    }

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D2 {
            grid.remove_candidate(wing1, digit);
        }
    }

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D3 {
            grid.remove_candidate(wing2, digit);
        }
    }

    TechniqueGrid::from(grid)
}

fn wxyz_wing_grid() -> TechniqueGrid {
    let mut grid = CandidateGrid::new();
    let pos1 = Position::new(0, 0);
    let pos2 = Position::new(1, 0);
    let pos3 = Position::new(0, 1);
    let pos4 = Position::new(1, 1);

    for digit in Digit::ALL {
        if digit != Digit::D1 && digit != Digit::D4 {
            grid.remove_candidate(pos1, digit);
        }
        if digit != Digit::D2 && digit != Digit::D4 {
            grid.remove_candidate(pos2, digit);
        }
        if digit != Digit::D1 && digit != Digit::D2 && digit != Digit::D3 {
            grid.remove_candidate(pos3, digit);
            grid.remove_candidate(pos4, digit);
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

fn bench_skyscraper_apply(c: &mut Criterion) {
    let puzzles = [
        ("skyscraper", skyscraper_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = Skyscraper::new();
    bench_apply_cases(c, "skyscraper_apply", &technique, &puzzles);
}

fn bench_two_string_kite_apply(c: &mut Criterion) {
    let puzzles = [
        ("two_string_kite", two_string_kite_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = TwoStringKite::new();
    bench_apply_cases(c, "two_string_kite_apply", &technique, &puzzles);
}

fn bench_y_wing_apply(c: &mut Criterion) {
    let puzzles = [("y_wing", y_wing_grid()), ("empty", TechniqueGrid::new())];
    let technique = YWing::new();
    bench_apply_cases(c, "y_wing_apply", &technique, &puzzles);
}

fn bench_swordfish_apply(c: &mut Criterion) {
    let puzzles = [
        ("swordfish", swordfish_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = Swordfish::new();
    bench_apply_cases(c, "swordfish_apply", &technique, &puzzles);
}

fn bench_jellyfish_apply(c: &mut Criterion) {
    let puzzles = [
        ("jellyfish", jellyfish_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = Jellyfish::new();
    bench_apply_cases(c, "jellyfish_apply", &technique, &puzzles);
}

fn bench_remote_pair_apply(c: &mut Criterion) {
    let puzzles = [
        ("remote_pair", remote_pair_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = RemotePair::new();
    bench_apply_cases(c, "remote_pair_apply", &technique, &puzzles);
}

fn bench_x_chain_apply(c: &mut Criterion) {
    let puzzles = [("x_chain", x_chain_grid()), ("empty", TechniqueGrid::new())];
    let technique = XChain::new();
    bench_apply_cases(c, "x_chain_apply", &technique, &puzzles);
}

fn bench_xy_chain_apply(c: &mut Criterion) {
    let puzzles = [
        ("xy_chain", xy_chain_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = XyChain::new();
    bench_apply_cases(c, "xy_chain_apply", &technique, &puzzles);
}

fn bench_xyz_wing_apply(c: &mut Criterion) {
    let puzzles = [
        ("xyz_wing", xyz_wing_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = XyzWing::new();
    bench_apply_cases(c, "xyz_wing_apply", &technique, &puzzles);
}

fn bench_wxyz_wing_apply(c: &mut Criterion) {
    let puzzles = [
        ("wxyz_wing", wxyz_wing_grid()),
        ("empty", TechniqueGrid::new()),
    ];
    let technique = WxyzWing::new();
    bench_apply_cases(c, "wxyz_wing_apply", &technique, &puzzles);
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

criterion_group!(
    name = benches_skyscraper;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_skyscraper_apply,
);

criterion_group!(
    name = benches_two_string_kite;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_two_string_kite_apply,
);

criterion_group!(
    name = benches_y_wing;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_y_wing_apply,
);

criterion_group!(
    name = benches_swordfish;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_swordfish_apply,
);

criterion_group!(
    name = benches_jellyfish;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_jellyfish_apply,
);

criterion_group!(
    name = benches_remote_pair;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_remote_pair_apply,
);

criterion_group!(
    name = benches_x_chain;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_x_chain_apply,
);

criterion_group!(
    name = benches_xy_chain;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_xy_chain_apply,
);

criterion_group!(
    name = benches_xyz_wing;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_xyz_wing_apply,
);

criterion_group!(
    name = benches_wxyz_wing;
    config =
        Criterion::default()
            .plotting_backend(PlottingBackend::Plotters);
    targets =
        bench_wxyz_wing_apply,
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
    benches_skyscraper,
    benches_two_string_kite,
    benches_y_wing,
    benches_swordfish,
    benches_jellyfish,
    benches_remote_pair,
    benches_x_chain,
    benches_xy_chain,
    benches_xyz_wing,
    benches_wxyz_wing,
);
