#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum SolverError {
    #[display("Contradiction detected")]
    Contradiction,
}
