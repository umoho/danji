use std::fmt;

#[derive(Debug)]
pub enum DanjiError {
    GpuInit(String),
    Diverged { sample: usize, iterations: usize },
    SingularMatrix { node: usize },
    InvalidCircuit(String),
    Numerical(String),
    BufferSize { expected: usize, actual: usize },
    InvalidParam(String),
}

impl fmt::Display for DanjiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DanjiError::GpuInit(msg) => write!(f, "GPU init failed: {}", msg),
            DanjiError::Diverged { sample, iterations } => {
                write!(
                    f,
                    "Diverged at sample {} after {} iterations",
                    sample, iterations
                )
            }
            DanjiError::SingularMatrix { node } => {
                write!(f, "Singular matrix at node {}", node)
            }
            DanjiError::InvalidCircuit(msg) => write!(f, "Invalid circuit: {}", msg),
            DanjiError::Numerical(msg) => write!(f, "Numerical error: {}", msg),
            DanjiError::BufferSize { expected, actual } => {
                write!(f, "Buffer mismatch: expected {}, got {}", expected, actual)
            }
            DanjiError::InvalidParam(msg) => write!(f, "Invalid param: {}", msg),
        }
    }
}

impl std::error::Error for DanjiError {}
