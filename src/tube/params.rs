/// 三极管参数（Koren 模型）。
///
/// 包含三极管仿真的所有物理参数，基于 Koren 模型。
/// 使用 [`TriodeParams::new_12ax7`] 等工厂方法获取预设参数。
///
/// # 字段说明
///
/// * `mu` - 放大系数 (μ)
/// * `ex` - 幂律指数
/// * `kg1` - 电流系数
/// * `kp` - 阴极耦合系数
/// * `kvb` - 电压反馈系数
///
/// ---
///
/// Triode parameters (Koren model).
///
/// Contains all physical parameters for triode simulation,
/// based on the Koren model. Use factory methods like
/// [`TriodeParams::new_12ax7`] for preset parameters.
///
/// # Fields
///
/// * `mu` - Amplification factor (μ)
/// * `ex` - Power law exponent
/// * `kg1` - Current coefficient
/// * `kp` - Cathode coupling coefficient
/// * `kvb` - Voltage feedback coefficient
#[derive(Debug, Clone)]
pub struct TriodeParams {
    /// 放大系数 (μ)
    pub mu: f64,
    /// 幂律指数
    pub ex: f64,
    /// 电流系数
    pub kg1: f64,
    /// 阴极耦合系数
    pub kp: f64,
    /// 电压反馈系数
    pub kvb: f64,
}

impl TriodeParams {
    /// 创建三极管参数。
    ///
    /// # 参数
    ///
    /// * `mu` - 放大系数 (μ)
    /// * `ex` - 幂律指数
    /// * `kg1` - 电流系数
    /// * `kp` - 阴极耦合系数
    /// * `kvb` - 电压反馈系数
    ///
    /// ---
    ///
    /// Create triode parameters.
    ///
    /// # Arguments
    ///
    /// * `mu` - Amplification factor (μ)
    /// * `ex` - Power law exponent
    /// * `kg1` - Current coefficient
    /// * `kp` - Cathode coupling coefficient
    /// * `kvb` - Voltage feedback coefficient
    pub const fn new(mu: f64, ex: f64, kg1: f64, kp: f64, kvb: f64) -> Self {
        Self {
            mu,
            ex,
            kg1,
            kp,
            kvb,
        }
    }

    /// 12AX7 (ECC83) - 高增益前级管
    pub const fn new_12ax7() -> Self {
        Self::new(100.0, 1.4, 1060.0, 600.0, 300.0)
    }

    /// 12AU7 (ECC82) - 中增益前级管
    pub const fn new_12au7() -> Self {
        Self::new(21.5, 1.3, 1180.0, 84.0, 300.0)
    }

    /// 12AT7 (ECC81) - 中高增益前级管
    pub const fn new_12at7() -> Self {
        Self::new(60.0, 1.35, 1200.0, 200.0, 300.0)
    }

    /// 6DJ8 (ECC88) - 低增益前级管
    pub const fn new_6dj8() -> Self {
        Self::new(28.0, 1.3, 330.0, 320.0, 300.0)
    }

    /// 6L6GC - 束射四极管（作为三极管使用）
    pub const fn new_6l6gc() -> Self {
        Self::new(8.7, 1.35, 1460.0, 48.0, 12.0)
    }

    /// 6550 - 功率五极管（作为三极管使用）
    pub const fn new_6550() -> Self {
        Self::new(7.9, 1.35, 890.0, 60.0, 24.0)
    }

    /// EL34 - 功率五极管（作为三极管使用）
    pub const fn new_el34() -> Self {
        Self::new(10.0, 1.35, 1200.0, 50.0, 15.0)
    }

    /// KT88 - 功率五极管（作为三极管使用）
    pub const fn new_kt88() -> Self {
        Self::new(8.8, 1.35, 730.0, 32.0, 16.0)
    }
}

/// 五极管参数（Koren 模型）。
///
/// 包含五极管仿真的所有物理参数，基于 Koren 模型。
/// 使用 [`PentodeParams::new_el84`] 等工厂方法获取预设参数。
///
/// # 字段说明
///
/// * `mu` - 放大系数 (μ)
/// * `ex` - 幂律指数
/// * `kg1` - 屏极电流系数
/// * `kg2` - 帘栅极电流系数
/// * `kp` - 阴极耦合系数
/// * `kvb` - 电压反馈系数
///
/// ---
///
/// Pentode parameters (Koren model).
///
/// Contains all physical parameters for pentode simulation,
/// based on the Koren model. Use factory methods like
/// [`PentodeParams::new_el84`] for preset parameters.
///
/// # Fields
///
/// * `mu` - Amplification factor (μ)
/// * `ex` - Power law exponent
/// * `kg1` - Plate current coefficient
/// * `kg2` - Screen grid current coefficient
/// * `kp` - Cathode coupling coefficient
/// * `kvb` - Voltage feedback coefficient
#[derive(Debug, Clone)]
pub struct PentodeParams {
    /// 放大系数 (μ)
    pub mu: f64,
    /// 幂律指数
    pub ex: f64,
    /// 屏极电流系数
    pub kg1: f64,
    /// 帘栅极电流系数
    pub kg2: f64,
    /// 阴极耦合系数
    pub kp: f64,
    /// 电压反馈系数
    pub kvb: f64,
}

impl PentodeParams {
    /// 创建五极管参数。
    ///
    /// # 参数
    ///
    /// * `mu` - 放大系数 (μ)
    /// * `ex` - 幂律指数
    /// * `kg1` - 屏极电流系数
    /// * `kg2` - 帘栅极电流系数
    /// * `kp` - 阴极耦合系数
    /// * `kvb` - 电压反馈系数
    ///
    /// ---
    ///
    /// Create pentode parameters.
    ///
    /// # Arguments
    ///
    /// * `mu` - Amplification factor (μ)
    /// * `ex` - Power law exponent
    /// * `kg1` - Plate current coefficient
    /// * `kg2` - Screen grid current coefficient
    /// * `kp` - Cathode coupling coefficient
    /// * `kvb` - Voltage feedback coefficient
    pub const fn new(mu: f64, ex: f64, kg1: f64, kg2: f64, kp: f64, kvb: f64) -> Self {
        Self {
            mu,
            ex,
            kg1,
            kg2,
            kp,
            kvb,
        }
    }

    /// EL84 / 6BQ5
    pub const fn new_el84() -> Self {
        Self::new(19.0, 1.35, 700.0, 2000.0, 100.0, 20.0)
    }

    /// EL34
    pub const fn new_el34() -> Self {
        Self::new(10.0, 1.35, 1200.0, 4500.0, 50.0, 15.0)
    }

    /// 6L6GC (pentode mode)
    pub const fn new_6l6gc_pentode() -> Self {
        Self::new(8.7, 1.35, 1460.0, 4500.0, 48.0, 12.0)
    }

    /// 6550 (pentode mode)
    pub const fn new_6550_pentode() -> Self {
        Self::new(7.9, 1.35, 890.0, 4200.0, 60.0, 24.0)
    }

    /// KT88 (pentode mode)
    pub const fn new_kt88_pentode() -> Self {
        Self::new(8.8, 1.35, 730.0, 4200.0, 32.0, 16.0)
    }
}
