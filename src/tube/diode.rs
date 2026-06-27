/// 二极管参数（Child-Langmuir 模型）。
///
/// 包含二极管仿真的所有物理参数。
///
/// ---
///
/// Diode parameters (Child-Langmuir model).
///
/// Contains all physical parameters for diode simulation.
pub struct DiodeParams {
    /// Child-Langmuir 系数
    ///
    /// Child-Langmuir coefficient
    pub k: f64,
    /// 幂律指数
    ///
    /// Power law exponent
    pub gamma: f64,
}

impl DiodeParams {
    /// 创建二极管参数。
    ///
    /// # 参数
    ///
    /// * `k` - Child-Langmuir 系数
    /// * `gamma` - 幂律指数
    ///
    /// ---
    ///
    /// Create diode parameters.
    ///
    /// # Arguments
    ///
    /// * `k` - Child-Langmuir coefficient
    /// * `gamma` - Power law exponent
    pub const fn new(k: f64, gamma: f64) -> Self {
        Self { k, gamma }
    }

    /// 5AR4 / GZ34
    pub const fn new_5ar4() -> Self {
        Self::new(0.005, 1.5)
    }

    /// 5U4G
    pub const fn new_5u4g() -> Self {
        Self::new(0.003, 1.5)
    }

    /// 6X4
    pub const fn new_6x4() -> Self {
        Self::new(0.002, 1.5)
    }

    /// EZ81
    pub const fn new_ez81() -> Self {
        Self::new(0.004, 1.5)
    }

    /// 硅二极管（用于对比）
    pub const fn new_silicon() -> Self {
        Self::new(1e-6, 2.0)
    }
}

/// 计算二极管电流。
///
/// 基于 Child-Langmuir 定律计算二极管在给定电压下的电流。
///
/// # 参数
///
/// * `vpk` - 阳极-阴极电压（单位：V，范围：0 ~ 1000）
/// * `params` - 二极管参数
///
/// # 返回值
///
/// 返回二极管电流（单位：A，范围：0 ~ 1.0）
///
/// ---
///
/// Calculate diode current.
///
/// Calculates diode current based on Child-Langmuir law.
///
/// # Arguments
///
/// * `vpk` - Anode-cathode voltage (unit: V, range: 0 ~ 1000)
/// * `params` - Diode parameters
///
/// # Returns
///
/// Returns diode current (unit: A, range: 0 ~ 1.0)
pub fn diode_current(vpk: f64, params: &DiodeParams) -> f64 {
    if vpk <= 0.0 {
        return 0.0;
    }
    params.k * vpk.powf(params.gamma)
}

/// 计算二极管电导 (dI/dV)。
///
/// 使用数值差分法计算二极管电导。
///
/// # 参数
///
/// * `vpk` - 阳极-阴极电压（单位：V，范围：0 ~ 1000）
/// * `params` - 二极管参数
///
/// # 返回值
///
/// 返回二极管电导（单位：S，西门子）
///
/// ---
///
/// Calculate diode conductance (dI/dV).
///
/// Uses numerical differentiation.
///
/// # Arguments
///
/// * `vpk` - Anode-cathode voltage (unit: V, range: 0 ~ 1000)
/// * `params` - Diode parameters
///
/// # Returns
///
/// Returns diode conductance (unit: S, Siemens)
pub fn diode_conductance(vpk: f64, params: &DiodeParams) -> f64 {
    let eps = (1e-6_f64).max(vpk.abs() * 1e-4);
    let i0 = diode_current(vpk - eps, params);
    let i1 = diode_current(vpk + eps, params);
    (i1 - i0) / (2.0 * eps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diode_zero_at_reverse() {
        let p = DiodeParams::new_5ar4();
        assert_eq!(diode_current(-100.0, &p), 0.0);
        assert_eq!(diode_current(-1.0, &p), 0.0);
    }

    #[test]
    fn test_diode_forward_current() {
        let p = DiodeParams::new_5ar4();
        assert!(diode_current(10.0, &p) > 0.0);
        assert!(diode_current(50.0, &p) > 0.0);
    }

    #[test]
    fn test_diode_current_increases_with_voltage() {
        let p = DiodeParams::new_5ar4();
        let i10 = diode_current(10.0, &p);
        let i50 = diode_current(50.0, &p);
        assert!(i50 > i10);
    }

    #[test]
    fn test_diode_conductance_positive() {
        let p = DiodeParams::new_5ar4();
        let g = diode_conductance(50.0, &p);
        assert!(g > 0.0);
    }

    #[test]
    fn test_5ar4_vs_5u4g() {
        let p1 = DiodeParams::new_5ar4();
        let p2 = DiodeParams::new_5u4g();
        assert!(diode_current(50.0, &p1) > diode_current(50.0, &p2));
    }
}
