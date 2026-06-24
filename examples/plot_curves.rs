use danji::tube::params::TriodeParams;
use danji::tube::triode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params_12ax7 = TriodeParams::new_12ax7();
    let params_12au7 = TriodeParams::new_12au7();

    println!("=== 12AX7 Plate Characteristic Curves ===");
    println!("{:>5} {:>5} {:>10}", "Vp(V)", "Vg(V)", "Ip(mA)");
    for vg in [0, -1, -2, -3, -4] {
        for vp in (0..=400).step_by(20) {
            let ip = triode::plate_current(vp as f64, vg as f64, &params_12ax7) * 1e3;
            println!("{:5} {:5} {:10.4}", vp, vg, ip);
        }
        println!();
    }

    println!("=== 12AX7 vs 12AU7 at Vp=250V, Vg=-2V ===");
    let ip_ax7 = triode::plate_current(250.0, -2.0, &params_12ax7);
    let ip_au7 = triode::plate_current(250.0, -2.0, &params_12au7);
    println!("12AX7: Ip = {:.4} mA", ip_ax7 * 1e3);
    println!("12AU7: Ip = {:.4} mA", ip_au7 * 1e3);

    let gp_ax7 = triode::dip_dvp(250.0, -2.0, &params_12ax7);
    let gm_ax7 = triode::dip_dvg(250.0, -2.0, &params_12ax7);
    println!("12AX7 @ Vp=250V, Vg=-2V:");
    println!("  gp = {:.6} S (plate conductance)", gp_ax7);
    println!("  gm = {:.6} S (transconductance)", gm_ax7);
    println!("  rp = {:.1} kohm (plate resistance)", 1.0 / gp_ax7 / 1e3);

    Ok(())
}
