use crate::tube::params::PentodeParams;

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

pub fn dip_dvp(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vp.abs() * 1e-4);
    let i0 = plate_current(vp - eps, vg1, vg2, vc, params);
    let i1 = plate_current(vp + eps, vg1, vg2, vc, params);
    (i1 - i0) / (2.0 * eps)
}

pub fn dip_dvg1(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vg1.abs() * 1e-4);
    let i0 = plate_current(vp, vg1 - eps, vg2, vc, params);
    let i1 = plate_current(vp, vg1 + eps, vg2, vc, params);
    (i1 - i0) / (2.0 * eps)
}

pub fn dip_dvg2(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    let eps = (1e-6_f64).max(vg2.abs() * 1e-4);
    let i0 = plate_current(vp, vg1, vg2 - eps, vc, params);
    let i1 = plate_current(vp, vg1, vg2 + eps, vc, params);
    (i1 - i0) / (2.0 * eps)
}

pub fn dip_dvc(vp: f64, vg1: f64, vg2: f64, vc: f64, params: &PentodeParams) -> f64 {
    -(dip_dvp(vp, vg1, vg2, vc, params)
        + dip_dvg1(vp, vg1, vg2, vc, params)
        + dip_dvg2(vp, vg1, vg2, vc, params))
}

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
