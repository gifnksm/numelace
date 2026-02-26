use std::str::FromStr as _;

use numelace_core::{Digit, DigitGrid, DigitSet, Position};

use crate::{BoxedTechniqueStep, Technique, TechniqueApplication, TechniqueGrid};

#[derive(Debug)]
pub struct TechniqueTester {
    initial: TechniqueGrid,
    current: TechniqueGrid,
    check_find_step_consistency: bool,
}

impl TechniqueTester {
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

    #[track_caller]
    pub fn from_str(s: &str) -> Self {
        let grid = DigitGrid::from_str(s).unwrap();
        Self::new(grid)
    }

    #[must_use]
    #[expect(dead_code)]
    pub fn with_find_step_consistency(mut self, enabled: bool) -> Self {
        self.check_find_step_consistency = enabled;
        self
    }

    #[must_use]
    pub fn without_find_step_consistency(mut self) -> Self {
        self.check_find_step_consistency = false;
        self
    }

    #[track_caller]
    pub fn apply_pass<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        let before = self.current.clone();
        let changed = technique.apply_pass(&mut self.current).unwrap();
        if self.check_find_step_consistency {
            Self::assert_find_step_consistent_once(technique, &before, &self.current, changed);
        }
        self
    }

    #[track_caller]
    pub fn apply_until_stuck<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        loop {
            let before = self.current.clone();
            let changed = technique.apply_pass(&mut self.current).unwrap();
            if self.check_find_step_consistency {
                Self::assert_find_step_consistent_once(technique, &before, &self.current, changed);
            }
            if changed == 0 {
                break;
            }
        }
        self
    }

    #[track_caller]
    pub fn apply_times<T>(mut self, technique: &T, times: usize) -> Self
    where
        T: Technique,
    {
        for _ in 0..times {
            let before = self.current.clone();
            let changed = technique.apply_pass(&mut self.current).unwrap();
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
        changed: usize,
    ) where
        T: Technique,
    {
        let name = technique.name();
        let step = technique.find_step(before).unwrap();
        match step {
            None => {
                assert_eq!(
                    changed, 0,
                    "Expected {name} to report no change when find_step returned None"
                );
                Self::assert_candidates_unchanged(before, after);
            }
            Some(step) => {
                assert_ne!(
                    changed, 0,
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
        TechniqueTier,
    };

    // Mock technique for testing that always returns false (no change)
    #[derive(Debug)]
    struct NoOpTechnique;

    impl Technique for NoOpTechnique {
        fn id(&self) -> &'static str {
            "no_op"
        }

        fn name(&self) -> &'static str {
            "No-op"
        }

        fn tier(&self) -> TechniqueTier {
            TechniqueTier::Fundamental
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

        fn apply_step(&self, _grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            Ok(false)
        }

        fn apply_pass(&self, _grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
            Ok(0)
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
        fn id(&self) -> &'static str {
            "place_d1_at_00"
        }

        fn name(&self) -> &'static str {
            "Place D1 at (0, 0)"
        }

        fn tier(&self) -> TechniqueTier {
            TechniqueTier::Fundamental
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

        fn apply_step(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            let pos = Position::new(0, 0);
            let candidates = grid.candidates_at(pos);
            if candidates.len() == 1 {
                Ok(false)
            } else {
                grid.place(pos, Digit::D1);
                Ok(true)
            }
        }

        fn apply_pass(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
            let pos = Position::new(0, 0);
            let candidates = grid.candidates_at(pos);
            if candidates.len() == 1 {
                Ok(0)
            } else {
                grid.place(pos, Digit::D1);
                Ok(1)
            }
        }
    }

    #[derive(Debug)]
    struct InconsistentTechnique;

    impl Technique for InconsistentTechnique {
        fn id(&self) -> &'static str {
            "inconsistent"
        }

        fn name(&self) -> &'static str {
            "Inconsistent"
        }

        fn tier(&self) -> TechniqueTier {
            TechniqueTier::Fundamental
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

        fn apply_step(&self, _grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            Ok(false)
        }

        fn apply_pass(&self, _grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
            Ok(0)
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
    fn test_apply_pass() {
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

        let result = tester.apply_pass(&PlaceD1At00);
        // Should not panic - technique was applied in one pass
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

        // PlaceD1At00 will apply once, then return 0
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
    #[should_panic(expected = "Expected Inconsistent to report a change")]
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
        .apply_pass(&InconsistentTechnique);
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
        .apply_pass(&InconsistentTechnique);
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
            .apply_pass(&PlaceD1At00)
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
            .apply_pass(&NoOpTechnique)
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
            .apply_pass(&NoOpTechnique)
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
            .apply_pass(&PlaceD1At00)
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
            .apply_pass(&PlaceD1At00)
            .assert_placed(Position::new(0, 0), Digit::D1)
            .apply_pass(&NoOpTechnique)
            .assert_no_change(Position::new(5, 5));
    }
}
