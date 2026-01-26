use numelace_core::ConsistencyError;

/// Errors that can occur during solving.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum SolverError {
    /// An inconsistency was detected in the candidate grid.
    ///
    /// This error wraps [`ConsistencyError`] and indicates that the puzzle is
    /// invalid or unsolvable. See [`ConsistencyError`] for the specific types
    /// of inconsistencies that can occur.
    #[display("inconsistency detected: {_0}")]
    Inconsistent(ConsistencyError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_error_converts_to_solver_error() {
        let consistency_error = ConsistencyError::NoCandidates;
        let solver_error: SolverError = consistency_error.into();
        assert!(matches!(
            solver_error,
            SolverError::Inconsistent(ConsistencyError::NoCandidates)
        ));
    }

    #[test]
    fn test_consistency_error_conversion_with_question_mark() {
        fn check() -> Result<(), SolverError> {
            Err(ConsistencyError::DuplicatedDecidedDigits)?
        }

        let result = check();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SolverError::Inconsistent(ConsistencyError::DuplicatedDecidedDigits)
        ));
    }
}
