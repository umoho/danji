use std::fmt;

/// danji 库的错误类型。
///
/// 本枚举定义了仿真过程中可能出现的所有错误情况。
///
/// ---
///
/// Error type for the danji library.
///
/// This enum defines all possible error conditions during simulation.
#[derive(Debug)]
pub enum DanjiError {
    /// GPU 初始化失败
    ///
    /// GPU initialization failed
    GpuInit(String),

    /// 迭代发散
    ///
    /// 在指定采样点处，Newton-Raphson 迭代未能收敛
    ///
    /// Iteration diverged at the specified sample point
    Diverged {
        /// 发生发散的采样点索引（从 0 开始）
        ///
        /// Sample index where divergence occurred (0-based)
        sample: usize,
        /// 已执行的迭代次数
        ///
        /// Number of iterations performed
        iterations: usize,
    },

    /// 矩阵奇异
    ///
    /// MNA 矩阵在指定节点处奇异，无法求解
    ///
    /// Singular matrix at the specified node
    SingularMatrix {
        /// 导致奇异的节点索引
        ///
        /// Node index causing singularity
        node: usize,
    },

    /// 无效电路配置
    ///
    /// Invalid circuit configuration
    InvalidCircuit(String),

    /// 数值计算错误
    ///
    /// Numerical computation error
    Numerical(String),

    /// 缓冲区大小不匹配
    ///
    /// 输入缓冲区大小与仿真器配置不匹配
    ///
    /// Input buffer size doesn't match simulator configuration
    BufferSize {
        /// 期望的缓冲区大小
        ///
        /// Expected buffer size
        expected: usize,
        /// 实际的缓冲区大小
        ///
        /// Actual buffer size
        actual: usize,
    },

    /// 无效参数
    ///
    /// Invalid parameter
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
