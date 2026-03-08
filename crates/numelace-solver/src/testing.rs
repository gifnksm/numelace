use std::str::FromStr as _;

use numelace_core::{ConsistencyError, Digit, DigitGrid, DigitSet, Position};

use crate::{BoxedTechniqueStep, SolverError, Technique, TechniqueApplication, TechniqueGrid};

#[derive(Debug, Clone)]
pub struct TechniqueTester {
    initial: TechniqueGrid,
    current: TechniqueGrid,
    check_find_step_consistency: bool,
    context: Option<String>,
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
            context: None,
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

    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    #[track_caller]
    pub fn apply_pass<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        let context = self.context_suffix();
        let before = self.current.clone();
        let changed = technique
            .apply_pass(&mut self.current)
            .unwrap_or_else(|err| {
                panic!("apply_pass failed with an unexpected error: {err:?}{context}");
            });
        if self.check_find_step_consistency {
            self.assert_find_step_consistent_once(technique, &before, changed);
        }
        self
    }

    #[track_caller]
    pub fn apply_pass_fail_with_constraint_violation<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        let context = self.context_suffix();
        match technique.apply_pass(&mut self.current) {
            Err(SolverError::Inconsistent(ConsistencyError::CandidateConstraintViolation)) => {}
            Ok(result) => panic!(
                "Expected apply_pass to fail, but it succeeded with result {result}{context}"
            ),
            Err(err) => panic!(
                "Expected apply_pass to fail with CandidateConstraintViolation, but it failed with a different error: {err:?}{context}"
            ),
        }
        self
    }

    #[track_caller]
    pub fn apply_until_stuck<T>(mut self, technique: &T) -> Self
    where
        T: Technique,
    {
        let context = self.context_suffix();
        loop {
            let before = self.current.clone();
            let changed = technique
                .apply_pass(&mut self.current)
                .unwrap_or_else(|err| {
                    panic!("apply_pass failed with an unexpected error: {err:?}{context}");
                });
            if self.check_find_step_consistency {
                self.assert_find_step_consistent_once(technique, &before, changed);
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
        let context = self.context_suffix();
        for _ in 0..times {
            let before = self.current.clone();
            let changed = technique
                .apply_pass(&mut self.current)
                .unwrap_or_else(|err| {
                    panic!("apply_pass failed with an unexpected error: {err:?}{context}")
                });
            if self.check_find_step_consistency {
                self.assert_find_step_consistent_once(technique, &before, changed);
            }
        }
        self
    }

    fn context_suffix(&self) -> String {
        self.context
            .as_ref()
            .map(|context| format!(" ({context})"))
            .unwrap_or_default()
    }

    #[track_caller]
    fn assert_find_step_consistent_once<T>(
        &self,
        technique: &T,
        before: &TechniqueGrid,
        changed: usize,
    ) where
        T: Technique,
    {
        let name = technique.name();
        let context = self.context_suffix();
        let step = technique.find_step(before).unwrap_or_else(|err| {
            panic!("find_step failed with an unexpected error: {err:?}{context}");
        });
        match step {
            None => {
                assert_eq!(
                    changed, 0,
                    "Expected {name} to report no change when find_step returned None{context}"
                );
                self.assert_candidates_unchanged(before);
            }
            Some(step) => {
                assert_ne!(
                    changed, 0,
                    "Expected {name} to report a change when find_step returned a step{context}"
                );
                self.assert_step_application_applied(before, &step);
            }
        }
    }

    #[track_caller]
    fn assert_candidates_unchanged(&self, before: &TechniqueGrid) {
        let after = &self.current;
        let context = self.context_suffix();
        for digit in Digit::ALL {
            let before_positions = before.digit_positions(digit);
            let after_positions = after.digit_positions(digit);
            assert_eq!(
                before_positions, after_positions,
                "Expected candidates to remain unchanged for {digit:?}{context}"
            );
        }
    }

    #[track_caller]
    fn assert_step_application_applied(&self, before: &TechniqueGrid, step: &BoxedTechniqueStep) {
        let after = &self.current;
        let context = self.context_suffix();
        let name = step.technique_name();
        for application in step.application() {
            match application {
                TechniqueApplication::Placement { position, digit } => {
                    let candidates = after.candidates_at(position);
                    assert_eq!(
                        candidates.len(),
                        1,
                        "Expected {position:?} to be univalue after applying {name}, but candidates are {candidates:?}{context}"
                    );
                    assert!(
                        candidates.contains(digit),
                        "Expected {position:?} to contain {digit:?} after applying {name}, but candidates are {candidates:?}{context}"
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
                                    "Expected {digit:?} to be removed from {pos:?} after applying {name}, but candidates are {after_candidates:?}{context}"
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
        let context = self.context_suffix();

        assert!(
            initial.len() > 1,
            "Expected initial position at {pos:?} to be non-univalue (>1 candidates), but had {} candidates: {initial:?}{context}",
            initial.len()
        );
        assert_eq!(
            current.len(),
            1,
            "Expected position at {pos:?} to be univalue (1 candidate), but has {} candidates: {current:?}{context}",
            current.len()
        );
        assert!(
            current.contains(digit),
            "Expected position at {pos:?} to contain {digit:?}, but candidates are: {current:?}{context}"
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
        let context = self.context_suffix();
        assert_eq!(
            initial & digits,
            digits,
            "Expected initial candidates at {pos:?} to include {digits:?}, but initial candidates are: {initial:?}{context}"
        );
        assert!(
            (current & digits).is_empty(),
            "Expected all of {digits:?} to be removed from {pos:?}, but {current:?} still contains some: {:?}{context}",
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
        let context = self.context_suffix();
        assert_eq!(
            removed, digits,
            "Expected exactly {digits:?} to be removed from {pos:?}, but removed candidates are: {removed:?} (initial: {initial:?}, current: {current:?}){context}"
        );
        self
    }

    #[track_caller]
    pub fn assert_no_change(self, pos: Position) -> Self {
        let initial = self.initial.candidates_at(pos);
        let current = self.current.candidates_at(pos);
        let context = self.context_suffix();
        assert_eq!(
            initial, current,
            "Expected no change at {pos:?}, but candidates changed from {initial:?} to {current:?}{context}"
        );
        self
    }

    #[track_caller]
    pub fn assert_no_change_all(mut self) -> Self {
        for pos in Position::ALL {
            self = self.assert_no_change(pos);
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSymmetry {
    Identity,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipH,
    FlipV,
    FlipDiag,
    FlipAntiDiag,
}

impl GridSymmetry {
    pub const ALL: [Self; 8] = [
        Self::Identity,
        Self::Rotate90,
        Self::Rotate180,
        Self::Rotate270,
        Self::FlipH,
        Self::FlipV,
        Self::FlipDiag,
        Self::FlipAntiDiag,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Identity => "Identity",
            Self::Rotate90 => "Rotate 90°",
            Self::Rotate180 => "Rotate 180°",
            Self::Rotate270 => "Rotate 270°",
            Self::FlipH => "Flip Horizontal",
            Self::FlipV => "Flip Vertical",
            Self::FlipDiag => "Flip Diagonal (\\)",
            Self::FlipAntiDiag => "Flip Anti-Diagonal (/)",
        }
    }

    pub fn apply_position(self, pos: Position) -> Position {
        match self {
            GridSymmetry::Identity => pos,
            GridSymmetry::Rotate90 => Position::from_xy(pos.y(), 8 - pos.x()),
            GridSymmetry::Rotate180 => Position::from_xy(8 - pos.x(), 8 - pos.y()),
            GridSymmetry::Rotate270 => Position::from_xy(8 - pos.y(), pos.x()),
            GridSymmetry::FlipH => Position::from_xy(8 - pos.x(), pos.y()),
            GridSymmetry::FlipV => Position::from_xy(pos.x(), 8 - pos.y()),
            GridSymmetry::FlipDiag => Position::from_xy(pos.y(), pos.x()),
            GridSymmetry::FlipAntiDiag => Position::from_xy(8 - pos.y(), 8 - pos.x()),
        }
    }

    pub fn apply_grid(self, grid: &TechniqueGrid) -> TechniqueGrid {
        let mut transformed = TechniqueGrid::new();
        for pos in Position::ALL {
            let digits = grid.candidates_at(pos);
            let transformed_pos = self.apply_position(pos);
            transformed.set_candidate_at(transformed_pos, digits);
        }
        for pos in grid.univalue_propagated() {
            let transformed_pos = self.apply_position(pos);
            transformed.insert_univalue_propagated(transformed_pos);
        }
        transformed
    }

    pub fn invert(self) -> Self {
        match self {
            Self::Identity => Self::Identity,
            Self::Rotate90 => Self::Rotate270,
            Self::Rotate180 => Self::Rotate180,
            Self::Rotate270 => Self::Rotate90,
            Self::FlipH => Self::FlipH,
            Self::FlipV => Self::FlipV,
            Self::FlipDiag => Self::FlipDiag,
            Self::FlipAntiDiag => Self::FlipAntiDiag,
        }
    }
}

impl TechniqueTester {
    pub fn with_symmetry(self, symmetry: GridSymmetry) -> Self {
        let initial = symmetry.apply_grid(&self.initial);
        let current = symmetry.apply_grid(&self.current);
        Self {
            initial,
            current,
            check_find_step_consistency: self.check_find_step_consistency,
            context: self.context,
        }
    }

    pub fn with_symmetry_back(self, symmetry: GridSymmetry) -> Self {
        let inverse = symmetry.invert();
        let initial = inverse.apply_grid(&self.initial);
        let current = inverse.apply_grid(&self.current);
        Self {
            initial,
            current,
            check_find_step_consistency: self.check_find_step_consistency,
            context: self.context,
        }
    }
}

#[track_caller]
pub fn run_with_symmetries<G, FApply, FAssert>(grid: G, mut apply: FApply, mut assert: FAssert)
where
    G: Into<TechniqueGrid>,
    FApply: FnMut(TechniqueTester) -> TechniqueTester,
    FAssert: FnMut(TechniqueTester),
{
    let tester = TechniqueTester::new(grid);
    for symmetry in GridSymmetry::ALL {
        let transformed_tester = tester
            .clone()
            .with_symmetry(symmetry)
            .with_context(format!("Symmetry: {}", symmetry.name()));
        let applied_tester = apply(transformed_tester).with_symmetry_back(symmetry);
        assert(applied_tester);
    }
}

pub fn test_technique_apply_until_stuck<G, T, FAssert>(grid: G, technique: &T, assert: FAssert)
where
    G: Into<TechniqueGrid>,
    T: Technique,
    FAssert: FnMut(TechniqueTester),
{
    run_with_symmetries(grid, |tester| tester.apply_until_stuck(technique), assert);
}

pub fn test_technique_apply_pass<G, T, FAssert>(grid: G, technique: &T, assert: FAssert)
where
    G: Into<TechniqueGrid>,
    T: Technique,
    FAssert: FnMut(TechniqueTester),
{
    run_with_symmetries(grid, |tester| tester.apply_pass(technique), assert);
}

pub fn test_technique_apply_pass_fail_with_constraint_violation<G, T>(grid: G, technique: &T)
where
    G: Into<TechniqueGrid>,
    T: Technique,
{
    run_with_symmetries(
        grid,
        |t| t.apply_pass_fail_with_constraint_violation(technique),
        |_| {},
    );
}

pub fn test_technique_apply_pass_no_changes<G, T>(grid: G, technique: &T)
where
    G: Into<TechniqueGrid>,
    T: Technique,
{
    run_with_symmetries(
        grid,
        |t| t.apply_pass(technique),
        |t| {
            t.assert_no_change_all();
        },
    );
}

#[cfg(test)]
mod tests {
    use numelace_core::DigitPositions;

    use super::*;
    use crate::{
        BoxedTechnique, BoxedTechniqueStep, ConditionDigitPositions, ConditionPositions,
        SolverError, TechniqueApplication, TechniqueStep, TechniqueTier,
    };

    fn assert_same_candidates(left: &TechniqueGrid, right: &TechniqueGrid) {
        for pos in Position::ALL {
            assert_eq!(
                left.candidates_at(pos),
                right.candidates_at(pos),
                "Candidates differ at {pos:?}"
            );
        }
        assert_eq!(
            left.univalue_propagated(),
            right.univalue_propagated(),
            "Univalue-propagated positions differ"
        );
    }

    #[test]
    fn test_grid_symmetry_round_trip() {
        let mut grid = TechniqueGrid::new();
        let pos = Position::from_xy(2, 3);
        grid.set_candidate_at(pos, DigitSet::from_iter([Digit::D1, Digit::D4]));
        let single_pos = Position::from_xy(8, 1);
        grid.set_candidate_at(single_pos, DigitSet::from_iter([Digit::D9]));
        grid.insert_univalue_propagated(single_pos);

        for symmetry in GridSymmetry::ALL {
            let transformed = symmetry.apply_grid(&grid);
            let round_trip = symmetry.invert().apply_grid(&transformed);
            assert_same_candidates(&grid, &round_trip);
        }
    }

    #[test]
    #[should_panic(expected = "Symmetry: Identity")]
    fn test_run_with_symmetries_panics_with_name() {
        let grid = TechniqueGrid::new();
        run_with_symmetries(
            grid,
            |tester| tester.apply_pass(&PlaceD1At00),
            |tester| {
                tester.assert_no_change(Position::from_xy(0, 0));
            },
        );
    }

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

        fn condition_positions(&self) -> ConditionPositions {
            DigitPositions::from_elem(Position::from_xy(0, 0))
        }

        fn condition_digit_positions(&self) -> ConditionDigitPositions {
            vec![(
                DigitPositions::from_elem(Position::from_xy(0, 0)),
                DigitSet::from_elem(Digit::D1),
            )]
        }

        fn application(&self) -> Vec<TechniqueApplication> {
            vec![TechniqueApplication::Placement {
                position: Position::from_xy(0, 0),
                digit: Digit::D1,
            }]
        }
    }

    // Mock technique that places a digit at (0, 0) if it's not already univalue
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
            let pos = Position::from_xy(0, 0);
            let candidates = grid.candidates_at(pos);
            if candidates.len() == 1 {
                Ok(None)
            } else {
                Ok(Some(Box::new(PlaceD1At00Step)))
            }
        }

        fn apply_step(&self, grid: &mut TechniqueGrid) -> Result<bool, SolverError> {
            let pos = Position::from_xy(0, 0);
            let candidates = grid.candidates_at(pos);
            if candidates.len() == 1 {
                Ok(false)
            } else {
                grid.place(pos, Digit::D1);
                Ok(true)
            }
        }

        fn apply_pass(&self, grid: &mut TechniqueGrid) -> Result<usize, SolverError> {
            let pos = Position::from_xy(0, 0);
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
            .assert_placed(Position::from_xy(0, 0), Digit::D1);
    }

    #[test]
    #[should_panic(expected = "Expected position at")]
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
            .assert_placed(Position::from_xy(0, 0), Digit::D1);
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
            .assert_no_change(Position::from_xy(0, 0));
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
            .assert_no_change(Position::from_xy(0, 0));
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
            .assert_placed(Position::from_xy(0, 0), Digit::D1)
            .apply_pass(&NoOpTechnique)
            .assert_no_change(Position::from_xy(5, 5));
    }
}
