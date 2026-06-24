#[derive(Debug, Clone)]
pub struct TriodeParams {
    pub mu: f64,
    pub ex: f64,
    pub kg1: f64,
    pub kp: f64,
    pub kvb: f64,
}

impl TriodeParams {
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
