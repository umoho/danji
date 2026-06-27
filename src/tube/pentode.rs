use crate::tube::params::PentodeParams;

/// 计算五极管屏极电流。
///
/// 基于改进的 Koren 模型计算五极管在给定电压下的屏极电流。
///
/// # 参数
///
/// * `vp` - 屏极电压（单位：V，范围：0 ~ 500）
/// * `vg1` - 控制栅极电压（单位：V，范围：-50 ~ 0）
/// * `vg2` - 帘栅极电压（单位：V，范围：0 ~ 500）
/// * `vc` - 阴极电压（单位：V，范围：0 ~ 50）
/// * `params` - 五极管参数
///
/// # 返回值
///
/// 返回屏极电流（单位：A，范围：0 ~ 0.5）
///
/// ---
///
/// Calculate pentode plate current.
///
/// Calculates pentode plate current based on modified Koren model.
///
/// # Arguments
///
/// * `vp` - Plate voltage (unit: V, range: 0 ~ 500)
/// * `vg1` - Control grid voltage (unit: V, range: -50 ~ 0)
/// * `vg2` - Screen grid voltage (unit: V, range: 0 ~ 500)
/// * `vc` - Cathode voltage (unit: V, range: 0 ~ 50)
/// * `params` - Pentode parameters
///
/// # Returns
///
/// Returns plate current (unit: A, range: 0 ~ 0.5)
pub fn plate_current(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let vpk = vp - vc;
    let vg1k = vg1 - vc;
    let vg2k = vg2 - vc;
    if vg2k <= 0.0 || vpk <= 0.0 {
        return 0.0;
    }
    let eg = params.kp * (1.0 / params.mu + vg1k / vg2k);
    let e1 = if eg > 50.0 {
        (vg2k / params.kp) * eg
    } else {
        (vg2k / params.kp) * (1.0 + eg.exp()).ln()
    };
    let ip = if e1 > 0.0 {
        (e1.powf(params.ex) + e1 * e1.powf(params.ex - 1.0)) / params.kg1
    } else {
        0.0
    };
    ip * (vpk / params.kvb).atan()
}

/// 计算五极管帘栅极电流。
///
/// # 参数
///
/// * `vg1` - 控制栅极电压（单位：V，范围：-50 ~ 0）
/// * `vg2` - 帘栅极电压（单位：V，范围：0 ~ 500）
/// * `vc` - 阴极电压（单位：V，范围：0 ~ 50）
/// * `params` - 五极管参数
///
/// # 返回值
///
/// 返回帘栅极电流（单位：A，范围：0 ~ 0.1）
///
/// ---
///
/// Calculate pentode screen grid current.
///
/// # Arguments
///
/// * `vg1` - Control grid voltage (unit: V, range: -50 ~ 0)
/// * `vg2` - Screen grid voltage (unit: V, range: 0 ~ 500)
/// * `vc` - Cathode voltage (unit: V, range: 0 ~ 50)
/// * `params` - Pentode parameters
///
/// # Returns
///
/// Returns screen grid current (unit: A, range: 0 ~ 0.1)
pub fn screen_current(vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let vg1k = vg1 - vc;
    let vg2k = vg2 - vc;
    if vg2k <= 0.0 {
        return 0.0;
    }
    let e = vg1k + vg2k / params.mu;
    if e <= 0.0 {
        return 0.0;
    }
    e.powf(1.5) / params.kg2
}

/// 计算五极管屏极电流对屏极电压的偏导数 (∂Ip/∂Vp)。
///
/// ---
///
/// Calculate pentode plate current partial derivative with respect to plate voltage (∂Ip/∂Vp).
pub fn dip_dvp(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vp.abs() * 1e-4);
    let i0 = plate_current(vp - eps, vg1, vg2, vc, params);
    let i1 = plate_current(vp + eps, vg1, vg2, vc, params);
    (i1 - i0) / (2.0 * eps)
}

/// 计算五极管屏极电流对控制栅极电压的偏导数 (∂Ip/∂Vg1)，即跨导 (gm)。
///
/// ---
///
/// Calculate pentode plate current partial derivative with respect to control grid voltage (∂Ip/∂Vg1),
/// also known as transconductance (gm).
pub fn dip_dvg1(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vg1.abs() * 1e-4);
    let i0 = plate_current(vp, vg1 - eps, vg2, vc, params);
    let i1 = plate_current(vp, vg1 + eps, vg2, vc, params);
    (i1 - i0) / (2.0 * eps)
}

/// 计算五极管屏极电流对帘栅极电压的偏导数 (∂Ip/∂Vg2)。
///
/// ---
///
/// Calculate pentode plate current partial derivative with respect to screen grid voltage (∂Ip/∂Vg2).
pub fn dip_dvg2(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vg2.abs() * 1e-4);
    let i0 = plate_current(vp, vg1, vg2 - eps, vc, params);
    let i1 = plate_current(vp, vg1, vg2 + eps, vc, params);
    (i1 - i0) / (2.0 * eps)
}

/// 计算五极管屏极电流对阴极电压的偏导数 (∂Ip/∂Vc)。
///
/// 通过链式法则计算：∂Ip/∂Vc = -(∂Ip/∂Vp + ∂Ip/∂Vg1 + ∂Ip/∂Vg2)
///
/// ---
///
/// Calculate pentode plate current partial derivative with respect to cathode voltage (∂Ip/∂Vc).
///
/// Calculated using chain rule: ∂Ip/∂Vc = -(∂Ip/∂Vp + ∂Ip/∂Vg1 + ∂Ip/∂Vg2)
pub fn dip_dvc(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    -(dip_dvp(vp, vg1, vg2, vc, params)
        + dip_dvg1(vp, vg1, vg2, vc, params)
        + dip_dvg2(vp, vg1, vg2, vc, params))
}

/// 计算五极管帘栅极电流对帘栅极电压的偏导数 (∂Ig2/∂Vg2)。
///
/// ---
///
/// Calculate pentode screen grid current partial derivative with respect to screen grid voltage (∂Ig2/∂Vg2).
pub fn dig2_dvg2(_vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vg2.abs() * 1e-4);
    let i0 = screen_current(vg1 - eps, vg2, vc, params);
    let i1 = screen_current(vg1 + eps, vg2, vc, params);
    (i1 - i0) / (2.0 * eps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_el84_plate_current() {
        let p = PentodeParams::new_el84();
        let ip = plate_current(250.0, -8.0, 250.0, 0.0, &p);
        assert!(ip > 0.0, "Ip should be positive, got {}", ip);
        assert!(ip < 0.1, "Ip should be reasonable, got {}", ip);
    }

    #[test]
    fn test_screen_current() {
        let p = PentodeParams::new_el84();
        let ig2 = screen_current(-8.0, 250.0, 0.0, &p);
        assert!(ig2 >= 0.0);
    }

    #[test]
    fn test_cutoff() {
        let p = PentodeParams::new_el84();
        let ip = plate_current(250.0, -50.0, 250.0, 0.0, &p);
        assert!(ip < 1e-10, "Should be cutoff, got {}", ip);
    }

    #[test]
    fn test_derivatives() {
        let p = PentodeParams::new_el84();
        let gp = dip_dvp(250.0, -8.0, 250.0, 0.0, &p);
        let gm = dip_dvg1(250.0, -8.0, 250.0, 0.0, &p);
        assert!(gp >= 0.0);
        assert!(gm > 0.0);
    }
}
