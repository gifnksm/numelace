//! Example demonstrating Sudoku puzzle solving with step-by-step output.
//!
//! # Usage
//!
//! Solve a puzzle from stdin:
//!
//! ```sh
//! cargo run --example solve_puzzle
//! ```
//!
//! Solve a puzzle from a file:
//!
//! ```sh
//! cargo run --example solve_puzzle -- path/to/puzzle.txt
//! ```
//!
//! Enable backtracking:
//!
//! ```sh
//! cargo run --example solve_puzzle -- --backtracking
//! ```
//!
//! Show step output:
//!
//! ```sh
//! cargo run --example solve_puzzle -- -vv
//! cargo run --example solve_puzzle -- -vvv
//! ```

use std::{
    fmt::Write as _,
    fs,
    io::{self, Read},
    path::PathBuf,
    process,
};

use clap::{ArgAction, Parser};
use numelace_core::{Digit, DigitGrid, DigitSet, Position, PositionIndexedArray};
use numelace_solver::{BoxedTechnique, SolverError, TechniqueGrid, backtrack, technique};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Enable backtracking when techniques get stuck.
    #[arg(long)]
    backtracking: bool,

    /// Show solver progress (-v, -vv, -vvv).
    #[arg(short, action = ArgAction::Count)]
    verbose: u8,

    /// Optional puzzle file (defaults to stdin).
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
enum StepMode {
    Silent,
    Pass,
    Step,
}

#[derive(Debug, Clone, Copy, Default)]
struct CellDiff {
    removed: DigitSet,
    decided: bool,
}

#[derive(Debug, Clone, Copy)]
struct CellStyle {
    fg: u8,
    bg: u8,
}

#[derive(Debug, Default)]
struct SolveStats {
    assumptions: Vec<(Position, Digit)>,
    backtrack_count: usize,
}

#[derive(Debug)]
struct SolveResult {
    solution: Option<TechniqueGrid>,
    stats: SolveStats,
    last_grid: TechniqueGrid,
}

#[derive(Debug)]
struct SearchState {
    grid: TechniqueGrid,
    assumptions: Vec<(Position, Digit)>,
    assumption: Option<(Position, DigitSet)>,
}

struct StepPrinter {
    verbosity: u8,
    step_index: usize,
    backtracking_occurred: bool,
}

impl StepPrinter {
    fn new(verbosity: u8) -> Self {
        Self {
            verbosity,
            step_index: 0,
            backtracking_occurred: false,
        }
    }

    fn note_backtrack(&mut self) {
        self.backtracking_occurred = true;
    }

    fn print_step(
        &mut self,
        technique_name: &str,
        count: usize,
        before: &TechniqueGrid,
        after: &TechniqueGrid,
    ) {
        if self.verbosity < 2 {
            return;
        }
        self.step_index += 1;
        if count > 1 {
            println!("Step {}: {} x{}", self.step_index, technique_name, count);
        } else {
            println!("Step {}: {}", self.step_index, technique_name);
        }
        let diffs = build_diffs(before, after);
        println!("{}", format_grid(after, Some(&diffs)));
    }

    fn print_assumption(&mut self, pos: Position, digit: Digit) {
        if self.verbosity < 1 {
            return;
        }
        self.step_index += 1;
        println!("Step {}: Assumption ({} = {})", self.step_index, pos, digit);
    }

    fn print_backtrack(&mut self, message: &str) {
        self.note_backtrack();
        if self.verbosity < 1 {
            return;
        }
        self.step_index += 1;
        println!("Step {}: Backtrack ({})", self.step_index, message);
    }
}

fn main() {
    let args = Args::parse();
    let input = match read_input(args.input.as_ref()) {
        Ok(input) => input,
        Err(err) => {
            eprintln!("Failed to read input: {err}");
            process::exit(2);
        }
    };

    let digit_grid: DigitGrid = match input.parse() {
        Ok(grid) => grid,
        Err(err) => {
            eprintln!("Invalid puzzle input: {err}");
            process::exit(2);
        }
    };

    let techniques = technique::all_techniques();
    let step_mode = match args.verbose {
        0 => StepMode::Silent,
        1 | 2 => StepMode::Pass,
        _ => StepMode::Step,
    };

    let mut printer = StepPrinter::new(args.verbose);

    let result = match solve_puzzle(
        TechniqueGrid::from_digit_grid(&digit_grid),
        &techniques,
        args.backtracking,
        step_mode,
        &mut printer,
    ) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Solver error: {err}");
            process::exit(2);
        }
    };

    if let Some(solution) = result.solution {
        println!("Solved:");
        println!("{}", format_grid(&solution, None));
    } else {
        println!("Could not solve:");
        println!("{}", format_grid(&result.last_grid, None));
    }

    if args.verbose >= 1 && (printer.backtracking_occurred || result.stats.backtrack_count > 0) {
        println!("Backtracking occurred: {}", result.stats.backtrack_count);
        if !result.stats.assumptions.is_empty() {
            println!("Assumptions:");
            for (i, (pos, digit)) in result.stats.assumptions.iter().enumerate() {
                println!("  {}. {} = {}", i + 1, *pos, digit);
            }
        }
    }
}

fn read_input(path: Option<&PathBuf>) -> io::Result<String> {
    if let Some(path) = path {
        fs::read_to_string(path)
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    }
}

fn solve_puzzle(
    grid: TechniqueGrid,
    techniques: &[BoxedTechnique],
    backtracking: bool,
    step_mode: StepMode,
    printer: &mut StepPrinter,
) -> Result<SolveResult, SolverError> {
    let mut stats = SolveStats::default();
    let mut last_grid = grid.clone();
    let mut stack = vec![SearchState {
        grid,
        assumptions: Vec::new(),
        assumption: None,
    }];

    while let Some(mut state) = stack.pop() {
        if let Some((pos, remaining_digits)) = &mut state.assumption {
            let Some(digit) = remaining_digits.pop_first() else {
                stats.backtrack_count += 1;
                printer.print_backtrack(&format!("exhausted candidates at {}", *pos));
                continue;
            };

            let pos = *pos;
            let mut grid = state.grid.clone();
            let mut assumptions = state.assumptions.clone();
            stack.push(state);

            printer.print_assumption(pos, digit);
            assumptions.push((pos, digit));
            grid.place(pos, digit);

            match solve_techniques_until_stuck(&mut grid, techniques, step_mode, printer) {
                Ok(SolveProgress::Solved) => {
                    last_grid = grid.clone();
                    stats.assumptions = assumptions;
                    return Ok(SolveResult {
                        solution: Some(grid),
                        stats,
                        last_grid,
                    });
                }
                Ok(SolveProgress::Stuck) => {
                    last_grid = grid.clone();
                    if backtracking {
                        let assumption = backtrack::find_best_assumption(&grid);
                        stack.push(SearchState {
                            grid,
                            assumptions,
                            assumption: Some(assumption),
                        });
                    } else {
                        stats.assumptions = assumptions;
                        return Ok(SolveResult {
                            solution: None,
                            stats,
                            last_grid,
                        });
                    }
                }
                Err(err) => {
                    stats.backtrack_count += 1;
                    printer.print_backtrack(&format!("inconsistent: {err}"));
                }
            }
            continue;
        }

        match solve_techniques_until_stuck(&mut state.grid, techniques, step_mode, printer) {
            Ok(SolveProgress::Solved) => {
                last_grid = state.grid.clone();
                stats.assumptions = state.assumptions;
                return Ok(SolveResult {
                    solution: Some(state.grid),
                    stats,
                    last_grid,
                });
            }
            Ok(SolveProgress::Stuck) => {
                last_grid = state.grid.clone();
                if !backtracking {
                    stats.assumptions = state.assumptions;
                    return Ok(SolveResult {
                        solution: None,
                        stats,
                        last_grid,
                    });
                }
                let assumption = backtrack::find_best_assumption(&state.grid);
                state.assumption = Some(assumption);
                stack.push(state);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(SolveResult {
        solution: None,
        stats,
        last_grid,
    })
}

#[derive(Debug, Clone, Copy)]
enum SolveProgress {
    Solved,
    Stuck,
}

fn solve_techniques_until_stuck(
    grid: &mut TechniqueGrid,
    techniques: &[BoxedTechnique],
    mode: StepMode,
    printer: &mut StepPrinter,
) -> Result<SolveProgress, SolverError> {
    grid.check_consistency()?;
    loop {
        if grid.is_solved()? {
            return Ok(SolveProgress::Solved);
        }

        let progressed = match mode {
            StepMode::Step => apply_step_with_techniques(grid, techniques, printer)?,
            StepMode::Pass | StepMode::Silent => {
                apply_pass_with_techniques(grid, techniques, printer)?
            }
        };

        if !progressed {
            return Ok(SolveProgress::Stuck);
        }
    }
}

fn apply_step_with_techniques(
    grid: &mut TechniqueGrid,
    techniques: &[BoxedTechnique],
    printer: &mut StepPrinter,
) -> Result<bool, SolverError> {
    grid.check_consistency()?;
    for technique in techniques {
        let before = grid.clone();
        if technique.apply_step(grid)? {
            grid.check_consistency()?;
            printer.print_step(technique.name(), 1, &before, grid);
            return Ok(true);
        }
    }
    Ok(false)
}

fn apply_pass_with_techniques(
    grid: &mut TechniqueGrid,
    techniques: &[BoxedTechnique],
    printer: &mut StepPrinter,
) -> Result<bool, SolverError> {
    grid.check_consistency()?;
    for technique in techniques {
        let before = grid.clone();
        let progress = technique.apply_pass(grid)?;
        if progress == 0 {
            continue;
        }
        grid.check_consistency()?;
        printer.print_step(technique.name(), progress, &before, grid);
        return Ok(true);
    }
    Ok(false)
}

fn build_diffs(before: &TechniqueGrid, after: &TechniqueGrid) -> PositionIndexedArray<CellDiff> {
    let mut diffs = PositionIndexedArray::from([CellDiff::default(); 81]);
    for pos in Position::ALL {
        let before_set = before.candidates_at(pos);
        let after_set = after.candidates_at(pos);
        if after_set.len() < before_set.len() {
            let decided = after_set.len() == 1 && before_set.len() > 1;
            let removed = if decided {
                DigitSet::EMPTY
            } else {
                before_set.difference(after_set)
            };
            diffs[pos] = CellDiff { removed, decided };
        }
    }
    diffs
}

fn format_grid(grid: &TechniqueGrid, diffs: Option<&PositionIndexedArray<CellDiff>>) -> String {
    let mut output = String::new();
    let horizontal = "+---------+---------+---------+\n";
    for y in 0..9 {
        if y % 3 == 0 {
            output.push_str(horizontal);
        }

        let mut cell_lines = Vec::with_capacity(9);
        for x in 0..9 {
            let pos = Position::new(x, y);
            let candidates = grid.candidates_at(pos);
            let diff = diffs.map(|map| map[pos]).unwrap_or_default();
            cell_lines.push(build_cell_lines(candidates, diff));
        }

        #[expect(clippy::needless_range_loop)]
        for sub_row in 0..3 {
            output.push('|');
            for block_x in 0..3 {
                for cell_x in 0..3 {
                    let cell_index = block_x * 3 + cell_x;
                    output.push_str(&cell_lines[cell_index][sub_row]);
                }
                output.push('|');
            }
            output.push('\n');
        }
    }
    output.push_str(horizontal);
    output
}

fn build_cell_lines(candidates: DigitSet, diff: CellDiff) -> [String; 3] {
    let decided_digit = candidates.as_single();
    let mut chars = [[' '; 3]; 3];
    if let Some(digit) = decided_digit {
        let value = u32::from(digit.value());
        chars[1][1] = char::from_digit(value, 10).unwrap_or('?');
    } else {
        for digit in Digit::ALL {
            if candidates.contains(digit) {
                let idx = digit.value() - 1;
                let row = (idx / 3) as usize;
                let col = (idx % 3) as usize;
                let value = u32::from(digit.value());
                chars[row][col] = char::from_digit(value, 10).unwrap_or('?');
            }
        }
    }

    let base_style = if diff.decided {
        CellStyle { fg: 30, bg: 42 }
    } else if decided_digit.is_some() {
        CellStyle { fg: 37, bg: 40 }
    } else {
        CellStyle { fg: 30, bg: 47 }
    };
    let mut styles = [[base_style; 3]; 3];

    if !diff.decided {
        for digit in diff.removed {
            let idx = digit.value() - 1;
            let row = (idx / 3) as usize;
            let col = (idx % 3) as usize;
            styles[row][col] = CellStyle { fg: 30, bg: 41 };
        }
    }

    let mut lines = [String::new(), String::new(), String::new()];
    for row in 0..3 {
        for col in 0..3 {
            let style = styles[row][col];
            let ch = chars[row][col];
            let _ = write!(lines[row], "\x1b[{};{}m{}\x1b[0m", style.fg, style.bg, ch);
        }
    }

    lines
}
