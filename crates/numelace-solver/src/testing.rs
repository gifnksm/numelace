//! Test utilities for technique implementations.
//!
//! This module provides [`TechniqueTester`], a testing harness for verifying
//! that sudoku solving techniques work as expected.
//!
//! # Example
//!
//! ```
//! # use numelace_solver::testing::TechniqueTester;
//! # use numelace_solver::technique::{BoxedTechniqueStep, Technique};
//! # use numelace_core::{Position, Digit};
//! # #[derive(Debug)] struct DummyTechnique;
//! # impl Technique for DummyTechnique {
//! #     fn name(&self) -> &str { "dummy" }
//! #     fn clone_box(&self) -> Box<dyn Technique> { Box::new(DummyTechnique) }
//! #     fn find_step(&self, _: &numelace_core::CandidateGrid) -> Result<Option<BoxedTechniqueStep>, numelace_solver::SolverError> { Ok(None) }
//! #     fn apply(&self, _: &mut numelace_core::CandidateGrid) -> Result<bool, numelace_solver::SolverError> { Ok(false) }
//! # }
//! # let technique = DummyTechnique;
//! TechniqueTester::from_str("
//!     5__ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//!     ___ ___ ___
//! ")
//! .apply_once(&technique)
//! .assert_placed(Position::new(1, 0), Digit::D1);
//! ```

use std::str::FromStr as _;

use numelace_core::{Digit, DigitGrid, DigitSet, Position};

use crate::{BoxedTechniqueStep, Technique, TechniqueApplication, TechniqueGrid};

/// A test harness for verifying technique implementations.
///
/// `TechniqueTester` tracks the initial and current state of a sudoku grid,
/// allowing you to apply techniques and assert that they produce the expected
/// changes.
///
/// # Method Chaining
///
/// All methods return `self`, enabling fluent method chaining for readable tests.
///
/// # Panics
///
/// All assertion methods panic with detailed messages on failure, using
/// `#[track_caller]` to report the correct source location.
#[derive(Debug)]
pub struct TechniqueTester {
    initial: TechniqueGrid,
    current: TechniqueGrid,
    check_find_step_consistency: bool,
}

impl TechniqueTester {
    /// Creates a new tester from an initial grid state.
    pub fn new<T>(initial: T) -> Self
    where
        T: Into<TechniqueGrid>,
    {
        let initial = initial.into();
        let current = initial.clone();
        Self {
            initial,
            current,
            check_find_step_consistency: true,
        }
    }

    /// Creates a new tester from a grid string.
    ///
    /// The string format matches [`DigitGrid::from_str`]:
    /// - Digits 1-9 represent filled cells
    /// - `.`, `_`, or `0` represent empty cells
    /// - Whitespace is ignored
    ///
    /// # Panics
    ///
    /// Panics if the string cannot be parsed as a valid grid.
    ///
    /// # Example
    ///
    /// ```
    /// # use numelace_solver::testing::TechniqueTester;
    /// let tester = TechniqueTester::from_str(
    ///     "
    ///     53_ _7_ ___
    ///     6__ 195 ___
    ///     _98 ___ _6_
    ///     8__ _6_ __3
    ///     4__ 8_3 __1
    ///     7__ _2_ __6
    ///     _6_ ___ 28_
    ///     ___ 419 __5
    ///     ___ _8_ _79
    /// ",
    /// );
    /// ```
    #[track_caller]
    pub fn from_str(s: &str) -> Self {
        let grid = DigitGrid::from_str(s).unwrap();
        Self::new(grid)
    }

    /// Enables or disables `find_step` consistency checks.
    ///
    /// When enabled, `apply_*` methods assert that `find_step` and `apply` are consistent.
    #[must_use]
    #[expect(dead_code)]
    pub fn with_find_step_consistency(mut self, enabled: bool) -> Self {
        self.check_find_step_consistency = enabled;
        self
    }

    /// Disables `find_step`/`apply` consistency checks for this tester.
    #[must_use]
    pub fn without_find_step_consistency(mut self) -> Self {
        self.check_find_step_consistency = false;
        self
    }

    /// Applies the technique once and returns self for chaining.
    ///
    /// # Panics
    ///
    /// Panics if the technique returns an error.
    #[track_caller]
    pub fn apply_once<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        let before = self.current.clone();
        let changed = technique.apply(&mut self.current).unwrap();
        if self.check_find_step_consistency {
            Self::assert_find_step_consistent_once(technique, &before, &self.current, changed);
        }
        self
    }

    /// Applies the technique repeatedly until it makes no more progress.
    ///
    /// # Panics
    ///
    /// Panics if the technique returns an error.
    #[track_caller]
    pub fn apply_until_stuck<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        loop {
            let before = self.current.clone();
            let changed = technique.apply(&mut self.current).unwrap();
            if self.check_find_step_consistency {
                Self::assert_find_step_consistent_once(technique, &before, &self.current, changed);
            }
            if !changed {
                break;
            }
        }
        self
    }

    /// Applies the technique a specific number of times.
    ///
    /// # Panics
    ///
    /// Panics if the technique returns an error.
    #[track_caller]
    pub fn apply_times<T>(mut self, technique: &T, times: usize) -> Self
    where
        T: Technique,
    {
        for _ in 0..times {
            let before = self.current.clone();
            let changed = technique.apply(&mut self.current).unwrap();
            if self.check_find_step_consistency {
                Self::assert_find_step_consistent_once(technique, &before, &self.current, changed);
            }
        }
        self
    }

    #[track_caller]
    fn assert_find_step_consistent_once<T>(
        technique: &T,
        before: &TechniqueGrid,
        after: &TechniqueGrid,
        changed: bool,
    ) where
        T: Technique,
    {
        let name = technique.name();
        let step = technique.find_step(before).unwrap();
        match step {
            None => {
                assert!(
                    !changed,
                    "Expected {name} to report no change when find_step returned None"
                );
                Self::assert_candidates_unchanged(before, after);
            }
            Some(step) => {
                assert!(
                    changed,
                    "Expected {name} to report a change when find_step returned a step"
                );
                Self::assert_step_application_applied(before, &step, after);
            }
        }
    }

    #[track_caller]
    fn assert_candidates_unchanged(before: &TechniqueGrid, after: &TechniqueGrid) {
        for digit in Digit::ALL {
            let before_positions = before.digit_positions(digit);
            let after_positions = after.digit_positions(digit);
            assert_eq!(
                before_positions, after_positions,
                "Expected candidates to remain unchanged for {digit:?}"
            );
        }
    }

    #[track_caller]
    fn assert_step_application_applied(
        before: &TechniqueGrid,
        step: &BoxedTechniqueStep,
        after: &TechniqueGrid,
    ) {
        let name = step.technique_name();
        for application in step.application() {
            match application {
                TechniqueApplication::Placement { position, digit } => {
                    let candidates = after.candidates_at(position);
                    assert_eq!(
                        candidates.len(),
                        1,
                        "Expected {position:?} to be decided after applying {name}, but candidates are {candidates:?}"
                    );
                    assert!(
                        candidates.contains(digit),
                        "Expected {position:?} to contain {digit:?} after applying {name}, but candidates are {candidates:?}"
                    );
                }
                TechniqueApplication::CandidateElimination { positions, digits } => {
                    for pos in positions {
                        let before_candidates = before.candidates_at(pos);
                        let after_candidates = after.candidates_at(pos);
                        for digit in digits {
                            if before_candidates.contains(digit) {
                                assert!(
                                    !after_candidates.contains(digit),
                                    "Expected {digit:?} to be removed from {pos:?} after applying {name}, but candidates are {after_candidates:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Asserts that a cell was placed (decided) with the given digit.
    ///
    /// This verifies that:
    /// - The cell was initially undecided (had multiple candidates)
    /// - The cell is now decided (has exactly one candidate)
    /// - That candidate is the expected digit
    ///
    /// # Panics
    ///
    /// Panics if the cell was not placed as expected.
    #[track_caller]
    pub fn assert_placed(self, pos: Position, digit: Digit) -> Self {
        let initial = self.initial.candidates_at(pos);
        let current = self.current.candidates_at(pos);

        assert!(
            initial.len() > 1,
            "Expected initial cell at {pos:?} to be undecided (>1 candidates), but had {} candidates: {initial:?}",
            initial.len()
        );
        assert_eq!(
            current.len(),
            1,
            "Expected cell at {pos:?} to be decided (1 candidate), but has {} candidates: {current:?}",
            current.len()
        );
        assert!(
            current.contains(digit),
            "Expected cell at {pos:?} to contain {digit:?}, but candidates are: {current:?}"
        );

        self
    }

    /// Asserts that all specified candidates were removed from a cell.
    ///
    /// This verifies that:
    /// - The specified digits were initially present in the cell's candidates
    /// - All of those digits have been removed from the current candidates
    ///
    /// Other candidates may also have been removed; this method only checks
    /// that the specified ones are gone.
    ///
    /// # Panics
    ///
    /// Panics if any of the specified digits are still present in the cell's candidates.
    #[track_caller]
    pub fn assert_removed_includes<C>(self, pos: Position, digits: C) -> Self
    where
        C: IntoIterator<Item = Digit>,
    {
        let digits = DigitSet::from_iter(digits);
        let initial = self.initial.candidates_at(pos);
        let current = self.current.candidates_at(pos);
        assert_eq!(
            initial & digits,
            digits,
            "Expected initial candidates at {pos:?} to include {digits:?}, but initial candidates are: {initial:?}"
        );
        assert!(
            (current & digits).is_empty(),
            "Expected all of {digits:?} to be removed from {pos:?}, but {current:?} still contains some: {:?}",
            current & digits
        );
        self
    }

    /// Asserts that exactly the specified candidates were removed from a cell.
    ///
    /// This verifies that the set of removed candidates exactly matches the
    /// specified set - no more, no less.
    ///
    /// # Panics
    ///
    /// Panics if the removed candidates don't exactly match the specified set.
    #[track_caller]
    pub fn assert_removed_exact<C>(self, pos: Position, digits: C) -> Self
    where
        C: IntoIterator<Item = Digit>,
    {
        let digits = DigitSet::from_iter(digits);
        let initial = self.initial.candidates_at(pos);
        let current = self.current.candidates_at(pos);
        let removed = initial.difference(current);
        assert_eq!(
            removed, digits,
            "Expected exactly {digits:?} to be removed from {pos:?}, but removed candidates are: {removed:?} (initial: {initial:?}, current: {current:?})"
        );
        self
    }

    /// Asserts that a cell's candidates have not changed.
    ///
    /// # Panics
    ///
    /// Panics if the cell's candidates differ from the initial state.
    #[track_caller]
    pub fn assert_no_change(self, pos: Position) -> Self {
        let initial = self.initial.candidates_at(pos);
        let current = self.current.candidates_at(pos);
        assert_eq!(
            initial, current,
            "Expected no change at {pos:?}, but candidates changed from {initial:?} to {current:?}"
        );
        self
    }
}

#[cfg(test)]
mod tests {
    use numelace_core::DigitPositions;

    use super::*;
    use crate::{
        BoxedTechnique, BoxedTechniqueStep, SolverError, TechniqueApplication, TechniqueStep,
    };

    // Mock technique for testing that always returns false (no change)
    #[derive(Debug)]
    struct NoOpTechnique;

    impl Technique for NoOpTechnique {
        fn name(&self) -> &'static str {
            "no-op"
        }

        fn clone_box(&self) -> BoxedTechnique {
            Box::new(NoOpTechnique)
        }

        fn find_step(
            &self,
            _grid: &TechniqueGrid,
        ) -> Result<Option<BoxedTechniqueStep>, SolverError> {
            Ok(None)
        }

        fn apply(&self, _grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            Ok(false)
        }
    }

    #[derive(Debug, Clone)]
    struct PlaceD1At00Step;

    impl TechniqueStep for PlaceD1At00Step {
        fn technique_name(&self) -> &'static str {
            "place-d1-at-00"
        }

        fn clone_box(&self) -> BoxedTechniqueStep {
            Box::new(self.clone())
        }

        fn condition_cells(&self) -> DigitPositions {
            DigitPositions::from_elem(Position::new(0, 0))
        }

        fn condition_digit_cells(&self) -> Vec<(DigitPositions, DigitSet)> {
            vec![(
                DigitPositions::from_elem(Position::new(0, 0)),
                DigitSet::from_elem(Digit::D1),
            )]
        }

        fn application(&self) -> Vec<TechniqueApplication> {
            vec![TechniqueApplication::Placement {
                position: Position::new(0, 0),
                digit: Digit::D1,
            }]
        }
    }

    // Mock technique that places a digit at (0, 0) if it's not already decided
    #[derive(Debug)]
    struct PlaceD1At00;

    impl Technique for PlaceD1At00 {
        fn name(&self) -> &'static str {
            "place-d1-at-00"
        }

        fn clone_box(&self) -> BoxedTechnique {
            Box::new(PlaceD1At00)
        }

        fn find_step(
            &self,
            grid: &TechniqueGrid,
        ) -> Result<Option<BoxedTechniqueStep>, SolverError> {
            let pos = Position::new(0, 0);
            let candidates = grid.candidates_at(pos);
            if candidates.len() == 1 {
                Ok(None)
            } else {
                Ok(Some(Box::new(PlaceD1At00Step)))
            }
        }

        fn apply(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            let pos = Position::new(0, 0);
            let candidates = grid.candidates_at(pos);
            if candidates.len() == 1 {
                Ok(false)
            } else {
                grid.place(pos, Digit::D1);
                Ok(true)
            }
        }
    }

    #[derive(Debug)]
    struct InconsistentTechnique;

    impl Technique for InconsistentTechnique {
        fn name(&self) -> &'static str {
            "inconsistent"
        }

        fn clone_box(&self) -> BoxedTechnique {
            Box::new(InconsistentTechnique)
        }

        fn find_step(
            &self,
            _grid: &TechniqueGrid,
        ) -> Result<Option<BoxedTechniqueStep>, SolverError> {
            Ok(Some(Box::new(PlaceD1At00Step)))
        }

        fn apply(&self, _grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            Ok(false)
        }
    }

    #[test]
    fn test_from_str_creates_tester() {
        let tester = TechniqueTester::from_str(
            "
            1__ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        // Should not panic
        let _ = tester;
    }

    #[test]
    fn test_apply_once() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        let result = tester.apply_once(&PlaceD1At00);
        // Should not panic - technique was applied once
        let _ = result;
    }

    #[test]
    fn test_apply_until_stuck() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        // PlaceD1At00 will apply once, then return false
        let result = tester.apply_until_stuck(&PlaceD1At00);
        let _ = result;
    }

    #[test]
    fn test_apply_times() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        let result = tester.apply_times(&NoOpTechnique, 5);
        let _ = result;
    }

    #[test]
    #[should_panic(expected = "Expected inconsistent to report a change")]
    fn test_find_step_consistency_panics_on_inconsistent_apply() {
        TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        )
        .apply_once(&InconsistentTechnique);
    }

    #[test]
    fn test_find_step_consistency_opt_out() {
        TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        )
        .without_find_step_consistency()
        .apply_once(&InconsistentTechnique);
    }

    #[test]
    fn test_assert_placed() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        tester
            .apply_once(&PlaceD1At00)
            .assert_placed(Position::new(0, 0), Digit::D1);
    }

    #[test]
    #[should_panic(expected = "Expected cell at")]
    fn test_assert_placed_fails_when_not_placed() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        tester
            .apply_once(&NoOpTechnique)
            .assert_placed(Position::new(0, 0), Digit::D1);
    }

    #[test]
    fn test_assert_no_change() {
        let tester = TechniqueTester::from_str(
            "
            1__ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        tester
            .apply_once(&NoOpTechnique)
            .assert_no_change(Position::new(0, 0));
    }

    #[test]
    #[should_panic(expected = "Expected no change at")]
    fn test_assert_no_change_fails_when_changed() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        tester
            .apply_once(&PlaceD1At00)
            .assert_no_change(Position::new(0, 0));
    }

    #[test]
    fn test_method_chaining() {
        let tester = TechniqueTester::from_str(
            "
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
            ___ ___ ___
        ",
        );

        tester
            .apply_once(&PlaceD1At00)
            .assert_placed(Position::new(0, 0), Digit::D1)
            .apply_once(&NoOpTechnique)
            .assert_no_change(Position::new(5, 5));
    }
}
