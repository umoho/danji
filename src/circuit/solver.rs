use crate::circuit::element::{CircuitDef, MAX_NODES};
use crate::circuit::node::NodeId;
use crate::error::DanjiError;
use crate::tube::diode;
use crate::tube::diode::DiodeParams;
use crate::tube::params::PentodeParams;
use crate::tube::params::TriodeParams;
use crate::tube::pentode;
use crate::tube::triode;
use log::{debug, error, warn};

const MAX_ITER: usize = 50;
const TOL: f64 = 1e-9;
const VSRC_G: f64 = 1e6;

pub struct CircuitSolver {
    pub num_nodes: usize,
    pub g: [[f64; MAX_NODES]; MAX_NODES],
    pub i: [f64; MAX_NODES],
    pub v: [f64; MAX_NODES],
    v_prev: [f64; MAX_NODES],
}

impl CircuitSolver {
    pub fn new(num_nodes: usize) -> Self {
        Self {
            num_nodes,
            g: [[0.0; MAX_NODES]; MAX_NODES],
            i: [0.0; MAX_NODES],
            v: [0.0; MAX_NODES],
            v_prev: [0.0; MAX_NODES],
        }
    }

    pub fn reset(&mut self) {
        self.v = [0.0; MAX_NODES];
        self.v_prev = [0.0; MAX_NODES];
    }

    pub fn solve(
        &mut self,
        circuit: &CircuitDef,
        triode_params: &[TriodeParams],
        pentode_params: &[PentodeParams],
        diode_params: &[DiodeParams],
        h: f64,
        vin: f64,
    ) -> Result<(), DanjiError> {
        let n = self.num_nodes;

        for _iter in 0..MAX_ITER {
            self.g = [[0.0; MAX_NODES]; MAX_NODES];
            self.i = [0.0; MAX_NODES];

            self.g[0][0] = 1.0;

            for r in &circuit.resistors {
                let a = r.a.0;
                let b = r.b.0;
                let g_val = 1.0 / r.ohms;
                if a > 0 {
                    self.g[a][a] += g_val;
                    if b > 0 {
                        self.g[a][b] -= g_val;
                    }
                }
                if b > 0 {
                    self.g[b][b] += g_val;
                    if a > 0 {
                        self.g[b][a] -= g_val;
                    }
                }
            }

            for cap in &circuit.capacitors {
                let a = cap.a.0;
                let b = cap.b.0;
                let gc = cap.farads / h;
                let v_a = if a > 0 { self.v_prev[a] } else { 0.0 };
                let v_b = if b > 0 { self.v_prev[b] } else { 0.0 };
                if a > 0 {
                    self.g[a][a] += gc;
                    if b > 0 {
                        self.g[a][b] -= gc;
                    }
                    self.i[a] += gc * (v_a - v_b);
                }
                if b > 0 {
                    self.g[b][b] += gc;
                    if a > 0 {
                        self.g[b][a] -= gc;
                    }
                    self.i[b] += gc * (v_b - v_a);
                }
            }

            for ind in &circuit.inductors {
                let a = ind.a.0;
                let b = ind.b.0;
                let gl = ind.henrys.recip() * h;
                if a > 0 {
                    self.g[a][a] += gl;
                    if b > 0 {
                        self.g[a][b] -= gl;
                    }
                    self.i[a] += ind.i_prev;
                }
                if b > 0 {
                    self.g[b][b] += gl;
                    if a > 0 {
                        self.g[b][a] -= gl;
                    }
                    self.i[b] -= ind.i_prev;
                }
            }

            for ci in &circuit.coupled_inductors {
                let (pa, pb) = (ci.p_a.0, ci.p_b.0);
                let (sa, sb) = (ci.s_a.0, ci.s_b.0);
                let m = ci.coupling * (ci.l_primary * ci.l_secondary).sqrt();
                let det = ci.l_primary * ci.l_secondary - m * m;
                if det <= 1e-30 {
                    warn!("coupled inductor det={:.2e} <= 0 (k={})", det, ci.coupling);
                    continue;
                }
                let g11 = h * ci.l_secondary / det;
                let g22 = h * ci.l_primary / det;
                let g12 = -h * m / det;

                if pa > 0 {
                    self.g[pa][pa] += g11;
                    if pb > 0 {
                        self.g[pa][pb] -= g11;
                    }
                    if sa > 0 {
                        self.g[pa][sa] += g12;
                    }
                    if sb > 0 {
                        self.g[pa][sb] -= g12;
                    }
                    self.i[pa] += ci.i1_prev;
                }
                if pb > 0 {
                    self.g[pb][pb] += g11;
                    if pa > 0 {
                        self.g[pb][pa] -= g11;
                    }
                    if sa > 0 {
                        self.g[pb][sa] -= g12;
                    }
                    if sb > 0 {
                        self.g[pb][sb] += g12;
                    }
                    self.i[pb] -= ci.i1_prev;
                }
                if sa > 0 {
                    self.g[sa][sa] += g22;
                    if sb > 0 {
                        self.g[sa][sb] -= g22;
                    }
                    if pa > 0 {
                        self.g[sa][pa] += g12;
                    }
                    if pb > 0 {
                        self.g[sa][pb] -= g12;
                    }
                    self.i[sa] += ci.i2_prev;
                }
                if sb > 0 {
                    self.g[sb][sb] += g22;
                    if sa > 0 {
                        self.g[sb][sa] -= g22;
                    }
                    if pa > 0 {
                        self.g[sb][pa] -= g12;
                    }
                    if pb > 0 {
                        self.g[sb][pb] += g12;
                    }
                    self.i[sb] -= ci.i2_prev;
                }
            }

            // TODO: 3-winding coupled inductor (push-pull) - needs proper MNA stamp

            for tri in &circuit.triodes {
                let p = tri.plate.0;
                let g = tri.grid.0;
                let c = tri.cathode.0;
                let params = &triode_params[tri.params_idx];
                let vp = if p > 0 { self.v[p] } else { 0.0 };
                let vg = if g > 0 { self.v[g] } else { 0.0 };
                let vc = if c > 0 { self.v[c] } else { 0.0 };
                let vpk = vp - vc;
                let vgk = vg - vc;

                let ip = triode::plate_current(vpk, vgk, params);
                let gp = triode::dip_dvp(vpk, vgk, params);
                let gm = triode::dip_dvg(vpk, vgk, params);
                let iconst = ip - gp * vpk - gm * vgk;

                if p > 0 {
                    self.g[p][p] += gp;
                    if g > 0 {
                        self.g[p][g] += gm;
                    }
                    if c > 0 {
                        self.g[p][c] -= gp + gm;
                    }
                    self.i[p] -= iconst;
                }
                if c > 0 {
                    self.g[c][p] -= gp;
                    if g > 0 {
                        self.g[c][g] -= gm;
                    }
                    self.g[c][c] += gp + gm;
                    self.i[c] += iconst;
                }
            }

            for p in &circuit.pentodes {
                let pl = p.plate.0;
                let g = p.grid.0;
                let c = p.cathode.0;
                let s = p.screen.0;
                let params = &pentode_params[p.params_idx];
                let vp = if pl > 0 { self.v[pl] } else { 0.0 };
                let vg = if g > 0 { self.v[g] } else { 0.0 };
                let vc = if c > 0 { self.v[c] } else { 0.0 };
                let vs = if s > 0 { self.v[s] } else { 0.0 };

                let ip = pentode::plate_current(vp, vg, vs, vc, params);
                let ig = pentode::screen_current(vg, vs, vc, params);
                let gp = pentode::dip_dvp(vp, vg, vs, vc, params);
                let gm1 = pentode::dip_dvg1(vp, vg, vs, vc, params);
                let gm2 = pentode::dip_dvg2(vp, vg, vs, vc, params);
                let gc = -(gp + gm1 + gm2);
                let gs = pentode::dig2_dvg2(vp, vg, vs, vc, params);
                let vpk = vp - vc;
                let vgk = vg - vc;
                let vsk = vs - vc;

                let const_p = ip - gp * vpk - gm1 * vgk - gm2 * vsk;
                let const_s = ig - gs * vsk;

                if pl > 0 {
                    self.g[pl][pl] += gp;
                    if g > 0 {
                        self.g[pl][g] += gm1;
                    }
                    if s > 0 {
                        self.g[pl][s] += gm2;
                    }
                    if c > 0 {
                        self.g[pl][c] += gc;
                    }
                    self.i[pl] -= const_p;
                }
                if s > 0 {
                    self.g[s][s] += gs;
                    if c > 0 {
                        self.g[s][c] -= gs;
                    }
                    self.i[s] -= const_s;
                }
                if c > 0 {
                    self.g[c][pl] -= gp;
                    if g > 0 {
                        self.g[c][g] -= gm1;
                    }
                    if s > 0 {
                        self.g[c][s] -= gm2 + gs;
                    }
                    self.g[c][c] += gp + gm1 + gm2 + gs;
                    self.i[c] += const_p + const_s;
                }
            }

            for d in &circuit.diodes {
                let a = d.anode.0;
                let c = d.cathode.0;
                let params = &diode_params[d.params_idx];
                let va = if a > 0 { self.v[a] } else { 0.0 };
                let vc = if c > 0 { self.v[c] } else { 0.0 };
                let vak = va - vc;

                let id = diode::diode_current(vak, params);
                let gd = diode::diode_conductance(vak, params);
                let iconst = if vak > 0.0 { id - gd * vak } else { 0.0 };

                if a > 0 {
                    self.g[a][a] += gd;
                    if c > 0 {
                        self.g[a][c] -= gd;
                    }
                    self.i[a] -= iconst;
                }
                if c > 0 {
                    self.g[c][a] -= gd;
                    self.g[c][c] += gd;
                    self.i[c] += iconst;
                }
            }

            if circuit.bplus_node != NodeId(0) {
                let bn = circuit.bplus_node.0;
                self.g[bn][bn] += VSRC_G;
                self.i[bn] += VSRC_G * circuit.bplus_voltage;
            }

            if circuit.input_node != NodeId(0) {
                let in_n = circuit.input_node.0;
                self.g[in_n][in_n] += VSRC_G;
                self.i[in_n] += VSRC_G * vin;
            }

            let v_old = self.v;
            self.solve_linear()?;

            let mut max_delta = 0.0;
            for (vj, v_oldj) in self.v[..n].iter().zip(v_old[..n].iter()) {
                let d = (vj - v_oldj).abs();
                if d > max_delta {
                    max_delta = d;
                }
            }
            if max_delta < TOL {
                if _iter > 10 {
                    debug!(
                        "solver converged in {} iterations, max_delta={:.2e}",
                        _iter, max_delta
                    );
                }
                self.v_prev = self.v;
                return Ok(());
            }
        }

        warn!("solver diverged after {} iterations", MAX_ITER);
        Err(DanjiError::Diverged {
            sample: 0,
            iterations: MAX_ITER,
        })
    }

    #[allow(clippy::needless_range_loop)]
    fn solve_linear(&mut self) -> Result<(), DanjiError> {
        let n = self.num_nodes;
        let mut a = self.g;
        let mut b = self.i;

        for col in 0..n {
            let mut max_val = a[col][col].abs();
            let mut max_row = col;
            for row in (col + 1)..n {
                let val = a[row][col].abs();
                if val > max_val {
                    max_val = val;
                    max_row = row;
                }
            }
            if max_val < 1e-40 {
                error!("singular matrix at column {}, pivot={:.2e}", col, max_val);
                return Err(DanjiError::SingularMatrix { node: col });
            }
            if max_row != col {
                a.swap(col, max_row);
                b.swap(col, max_row);
            }
            let pivot = a[col][col];
            for row in (col + 1)..n {
                let factor = a[row][col] / pivot;
                for j in col..n {
                    a[row][j] -= factor * a[col][j];
                }
                b[row] -= factor * b[col];
            }
        }

        for row in (0..n).rev() {
            let mut sum = 0.0;
            for j in (row + 1)..n {
                sum += a[row][j] * self.v[j];
            }
            self.v[row] = (b[row] - sum) / a[row][row];
        }

        Ok(())
    }
}
