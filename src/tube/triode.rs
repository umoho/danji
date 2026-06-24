use crate::tube::params::TriodeParams;

pub fn plate_current(vp: f64, vg: f64, params: &TriodeParams) -> f64 {
    let vp_abs = vp.abs();
    let inner = 1.0 / params.mu + vg / (params.kvb + vp_abs * vp_abs).sqrt();
    let arg = params.kp * inner;
    let e1 = if arg > 700.0 {
        (vp / params.kp) * arg
    } else {
        (vp / params.kp) * (1.0 + arg.exp()).ln()
    };

    let ip = if e1 > 0.0 {
        (e1.powf(params.ex) + e1 * e1.powf(params.ex - 1.0)) / params.kg1
    } else {
        0.0
    };
    ip.max(0.0)
}

pub fn dip_dvp(vp: f64, vg: f64, params: &TriodeParams) -> f64 {
    let eps = 1e-6;
    let ip0 = plate_current(vp - eps, vg, params);
    let ip1 = plate_current(vp + eps, vg, params);
    (ip1 - ip0) / (2.0 * eps)
}

pub fn dip_dvg(vp: f64, vg: f64, params: &TriodeParams) -> f64 {
    let eps = 1e-6;
    let ip0 = plate_current(vp, vg - eps, params);
    let ip1 = plate_current(vp, vg + eps, params);
    (ip1 - ip0) / (2.0 * eps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tube::params::TriodeParams;

    #[test]
    fn test_12ax7_cutoff() {
        let p = TriodeParams::new_12ax7();
        assert!(plate_current(250.0, -10.0, &p) < 1e-12);
        assert!(plate_current(100.0, -5.0, &p) < 1e-12);
        assert_eq!(plate_current(0.0, 0.0, &p), 0.0);
    }

    #[test]
    fn test_12ax7_positive_current() {
        let p = TriodeParams::new_12ax7();
        assert!(plate_current(250.0, -2.0, &p) > 0.0);
        assert!(plate_current(200.0, -1.0, &p) > 0.0);
        assert!(plate_current(300.0, 0.0, &p) > 0.0);
    }

    #[test]
    fn test_12ax7_current_increases_with_vp() {
        let p = TriodeParams::new_12ax7();
        let ip100 = plate_current(100.0, -1.0, &p);
        let ip200 = plate_current(200.0, -1.0, &p);
        let ip300 = plate_current(300.0, -1.0, &p);
        assert!(ip300 > ip200);
        assert!(ip200 > ip100);
    }

    #[test]
    fn test_12ax7_current_increases_with_vg() {
        let p = TriodeParams::new_12ax7();
        let ip_neg2 = plate_current(250.0, -2.0, &p);
        let ip_neg1 = plate_current(250.0, -1.0, &p);
        let ip_0 = plate_current(250.0, 0.0, &p);
        assert!(ip_0 > ip_neg1);
        assert!(ip_neg1 > ip_neg2);
    }

    #[test]
    fn test_12ax7_typical_bias_point() {
        let p = TriodeParams::new_12ax7();
        let ip = plate_current(250.0, -2.0, &p) * 1e3;
        assert!((ip - 0.95).abs() < 0.2, "Ip = {} mA", ip);
    }

    #[test]
    fn test_12ax7_plate_resistance() {
        let p = TriodeParams::new_12ax7();
        let gp = dip_dvp(250.0, -2.0, &p);
        let rp = 1.0 / gp;
        assert!(rp > 30_000.0 && rp < 80_000.0, "rp = {} ohm", rp);
    }

    #[test]
    fn test_12ax7_transconductance() {
        let p = TriodeParams::new_12ax7();
        let gm = dip_dvg(250.0, -2.0, &p);
        assert!(gm > 0.001 && gm < 0.003, "gm = {} S", gm);
    }

    #[test]
    fn test_12au7_higher_current() {
        let p_ax7 = TriodeParams::new_12ax7();
        let p_au7 = TriodeParams::new_12au7();
        let ip_ax7 = plate_current(250.0, -2.0, &p_ax7);
        let ip_au7 = plate_current(250.0, -2.0, &p_au7);
        assert!(ip_au7 > ip_ax7, "12AU7 Ip({}) should > 12AX7 Ip({})", ip_au7, ip_ax7);
    }

    #[test]
    fn test_derivatives_positive_in_active_region() {
        let p = TriodeParams::new_12ax7();
        let gp = dip_dvp(250.0, -2.0, &p);
        let gm = dip_dvg(250.0, -2.0, &p);
        assert!(gp > 0.0, "gp = {}", gp);
        assert!(gm > 0.0, "gm = {}", gm);
    }
}
