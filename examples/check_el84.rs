use danji::tube::pentode;
use danji::PentodeParams;

fn main() {
    let p = PentodeParams::new_el84();
    println!("EL84 pentode model check:");
    for vg1 in [-20.0, -15.0, -10.0, -8.0, -5.0, 0.0] {
        let ip = pentode::plate_current(250.0, vg1, 250.0, 0.0, &p);
        let ig = pentode::screen_current(vg1, 250.0, 0.0, &p);
        println!("  Vg1={:5.1}V Ip={:8.2}mA Ig2={:7.2}mA", vg1, ip * 1000.0, ig * 1000.0);
    }
}
