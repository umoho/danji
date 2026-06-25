use crate::circuit::element::{
    Capacitor, CircuitDef, CoupledInductor, CoupledInductor3, DiodeInstance, Inductor,
    PentodeInstance, Resistor, TriodeInstance,
};
use crate::circuit::node::NodeId;
use crate::circuit::solver::CircuitSolver;
use crate::error::DanjiError;
use crate::tube::diode::DiodeParams;
use crate::tube::params::PentodeParams;
use crate::tube::params::TriodeParams;
use log::{debug, info};

pub struct Simulator {
    config: SimConfig,
    solver: CircuitSolver,
    triode_params: Vec<TriodeParams>,
    pentode_params: Vec<PentodeParams>,
    diode_params: Vec<DiodeParams>,
    sample_count: usize,
}

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub sample_rate: u32,
    pub num_nodes: usize,
    pub resistors: Vec<Resistor>,
    pub capacitors: Vec<Capacitor>,
    pub inductors: Vec<Inductor>,
    pub coupled_inductors: Vec<CoupledInductor>,
    pub coupled_inductors3: Vec<CoupledInductor3>,
    pub triodes: Vec<TriodeInstance>,
    pub pentodes: Vec<PentodeInstance>,
    pub diodes: Vec<DiodeInstance>,
    pub input_node: NodeId,
    pub(crate) input2_node: NodeId,
    pub(crate) input2_voltage: f64,
    pub output_node: NodeId,
    pub bplus_node: NodeId,
    pub bplus_voltage: f64,
}

impl SimConfig {
    pub fn new(sample_rate: u32, num_nodes: usize) -> Self {
        Self {
            sample_rate,
            num_nodes,
            resistors: Vec::new(),
            capacitors: Vec::new(),
            inductors: Vec::new(),
            coupled_inductors: Vec::new(),
            coupled_inductors3: Vec::new(),
            triodes: Vec::new(),
            pentodes: Vec::new(),
            diodes: Vec::new(),
            input_node: NodeId(0),
            input2_node: NodeId(0),
            input2_voltage: 0.0,
            output_node: NodeId(0),
            bplus_node: NodeId(0),
            bplus_voltage: 0.0,
        }
    }

    pub fn add_resistor(&mut self, a: NodeId, b: NodeId, ohms: f64) -> &mut Self {
        self.resistors.push(Resistor::new(a, b, ohms));
        self
    }

    pub fn add_capacitor(&mut self, a: NodeId, b: NodeId, farads: f64) -> &mut Self {
        self.capacitors.push(Capacitor::new(a, b, farads));
        self
    }

    pub fn add_inductor(&mut self, a: NodeId, b: NodeId, henrys: f64) -> &mut Self {
        self.inductors.push(Inductor::new(a, b, henrys));
        self
    }

    pub fn add_coupled_inductor(
        &mut self,
        p_a: NodeId,
        p_b: NodeId,
        s_a: NodeId,
        s_b: NodeId,
        l_primary: f64,
        l_secondary: f64,
        coupling: f64,
    ) -> &mut Self {
        self.coupled_inductors.push(CoupledInductor::new(
            p_a,
            p_b,
            s_a,
            s_b,
            l_primary,
            l_secondary,
            coupling,
        ));
        self
    }

    pub fn add_coupled_inductor3(
        &mut self,
        p1: NodeId,
        ct: NodeId,
        p2: NodeId,
        s1: NodeId,
        s2: NodeId,
        l1: f64,
        l2: f64,
        l3: f64,
        k12: f64,
        k13: f64,
        k23: f64,
    ) -> &mut Self {
        self.coupled_inductors3.push(CoupledInductor3::new(
            p1, ct, p2, s1, s2, l1, l2, l3, k12, k13, k23,
        ));
        self
    }

    pub fn add_triode(
        &mut self,
        plate: NodeId,
        grid: NodeId,
        cathode: NodeId,
        params_idx: usize,
    ) -> &mut Self {
        self.triodes.push(TriodeInstance {
            plate,
            grid,
            cathode,
            params_idx,
        });
        self
    }

    pub fn add_pentode(
        &mut self,
        plate: NodeId,
        grid: NodeId,
        cathode: NodeId,
        screen: NodeId,
        params_idx: usize,
    ) -> &mut Self {
        self.pentodes.push(PentodeInstance {
            plate,
            grid,
            cathode,
            screen,
            params_idx,
        });
        self
    }

    pub fn add_diode(&mut self, anode: NodeId, cathode: NodeId, params_idx: usize) -> &mut Self {
        self.diodes.push(DiodeInstance {
            anode,
            cathode,
            params_idx,
        });
        self
    }

    pub fn input(&mut self, node: NodeId) -> &mut Self {
        self.input_node = node;
        self
    }

    pub fn output(&mut self, node: NodeId) -> &mut Self {
        self.output_node = node;
        self
    }

    pub fn bplus(&mut self, node: NodeId, voltage: f64) -> &mut Self {
        self.bplus_node = node;
        self.bplus_voltage = voltage;
        self
    }

    #[allow(dead_code)]
    pub fn input2(&mut self, node: NodeId) -> &mut Self {
        self.input2_node = node;
        self
    }

    fn to_circuit_def(&self) -> CircuitDef {
        CircuitDef {
            num_nodes: self.num_nodes,
            resistors: self.resistors.clone(),
            capacitors: self.capacitors.clone(),
            inductors: self.inductors.clone(),
            coupled_inductors: self.coupled_inductors.clone(),
            coupled_inductors3: self.coupled_inductors3.clone(),
            triodes: self.triodes.clone(),
            pentodes: self.pentodes.clone(),
            diodes: self.diodes.clone(),
            input_node: self.input_node,
            input2_node: self.input2_node,
            input2_voltage: self.input2_voltage,
            output_node: self.output_node,
            bplus_node: self.bplus_node,
            bplus_voltage: self.bplus_voltage,
        }
    }
}

impl Simulator {
    pub fn new(
        config: SimConfig,
        triode_params: Vec<TriodeParams>,
        pentode_params: Vec<PentodeParams>,
        diode_params: Vec<DiodeParams>,
    ) -> Self {
        let solver = CircuitSolver::new(config.num_nodes);
        info!(
            "create simulator: {} nodes, {} R, {} C, {} L, {} CL, {} CL3, {} triodes, {} pentodes, {} diodes",
            config.num_nodes,
            config.resistors.len(),
            config.capacitors.len(),
            config.inductors.len(),
            config.coupled_inductors.len(),
            config.coupled_inductors3.len(),
            config.triodes.len(),
            config.pentodes.len(),
            config.diodes.len(),
        );
        Self {
            config,
            solver,
            triode_params,
            pentode_params,
            diode_params,
            sample_count: 0,
        }
    }

    pub fn reset(&mut self) {
        debug!("simulator reset after {} samples", self.sample_count);
        self.solver.reset();
        for cap in &mut self.config.capacitors {
            cap.v_prev = 0.0;
        }
        for ind in &mut self.config.inductors {
            ind.i_prev = 0.0;
        }
        for ci in &mut self.config.coupled_inductors {
            ci.i1_prev = 0.0;
            ci.i2_prev = 0.0;
        }
        self.sample_count = 0;
    }

    pub fn process_sample_dual(&mut self, input1: f32, input2: f64) -> Result<f32, DanjiError> {
        let prev = self.config.input2_voltage;
        self.config.input2_voltage = input2;
        let result = self.process_sample(input1);
        self.config.input2_voltage = if result.is_ok() { 0.0 } else { prev };
        result
    }

    pub fn process_sample(&mut self, input: f32) -> Result<f32, DanjiError> {
        let fs = self.config.sample_rate as f64;
        let h = 1.0 / fs;
        let circuit_def = self.config.to_circuit_def();

        self.solver.solve(
            &circuit_def,
            &self.triode_params,
            &self.pentode_params,
            &self.diode_params,
            h,
            input as f64,
        )?;

        for cap in &mut self.config.capacitors {
            let a = cap.a.0;
            let b = cap.b.0;
            let v_a = if a > 0 { self.solver.v[a] } else { 0.0 };
            let v_b = if b > 0 { self.solver.v[b] } else { 0.0 };
            cap.v_prev = v_a - v_b;
        }

        for ind in &mut self.config.inductors {
            let a = ind.a.0;
            let b = ind.b.0;
            let v_a = if a > 0 { self.solver.v[a] } else { 0.0 };
            let v_b = if b > 0 { self.solver.v[b] } else { 0.0 };
            let gl = ind.henrys.recip() * h;
            ind.i_prev += gl * (v_a - v_b);
            // BE DC leakage: prevents DC accumulation in lossless BE model
            // Use ~100Ω DCR as damping, not the full reflected load
            let damping = (h * 100.0 / ind.henrys).min(0.5);
            ind.i_prev *= 1.0 - damping;
        }

        for ci in &mut self.config.coupled_inductors {
            let (pa, pb) = (ci.p_a.0, ci.p_b.0);
            let (sa, sb) = (ci.s_a.0, ci.s_b.0);
            let v_p = if pa > 0 { self.solver.v[pa] } else { 0.0 }
                - if pb > 0 { self.solver.v[pb] } else { 0.0 };
            let v_s = if sa > 0 { self.solver.v[sa] } else { 0.0 }
                - if sb > 0 { self.solver.v[sb] } else { 0.0 };
            let m = ci.coupling * (ci.l_primary * ci.l_secondary).sqrt();
            let det = ci.l_primary * ci.l_secondary - m * m;
            if det <= 0.0 {
                continue;
            }
            let g11 = h * ci.l_secondary / det;
            let g22 = h * ci.l_primary / det;
            let g12 = -h * m / det;
            // BE with Rdc damping: prevents unbounded DC accumulation
            // dI/dt = V/L - R/L * I  →  I[n] = (1-hR/L)*I[n-1] + h/L*V[n]
            let rdc_pri = 150.0;
            let rdc_sec = 0.5;
            let d1 = h * rdc_pri / ci.l_primary;
            let d2 = h * rdc_sec / ci.l_secondary;
            ci.i1_prev = (1.0 - d1) * ci.i1_prev + g11 * v_p + g12 * v_s;
            ci.i2_prev = (1.0 - d2) * ci.i2_prev + g12 * v_p + g22 * v_s;
        }

        for ci in &mut self.config.coupled_inductors3 {
            let v = [
                if ci.p1.0 > 0 {
                    self.solver.v[ci.p1.0]
                } else {
                    0.0
                } - if ci.ct.0 > 0 {
                    self.solver.v[ci.ct.0]
                } else {
                    0.0
                },
                if ci.p2.0 > 0 {
                    self.solver.v[ci.p2.0]
                } else {
                    0.0
                } - if ci.ct.0 > 0 {
                    self.solver.v[ci.ct.0]
                } else {
                    0.0
                },
                if ci.s1.0 > 0 {
                    self.solver.v[ci.s1.0]
                } else {
                    0.0
                } - if ci.s2.0 > 0 {
                    self.solver.v[ci.s2.0]
                } else {
                    0.0
                },
            ];
            let ls = [ci.l1, ci.l2, ci.l3];
            let m12 = ci.k12 * (ls[0] * ls[1]).sqrt();
            let m13 = ci.k13 * (ls[0] * ls[2]).sqrt();
            let m23 = ci.k23 * (ls[1] * ls[2]).sqrt();
            let det = ls[0] * ls[1] * ls[2] + 2.0 * m12 * m13 * m23
                - ls[0] * m23 * m23
                - ls[1] * m13 * m13
                - ls[2] * m12 * m12;
            if det <= 1e-30 {
                continue;
            }
            let y00 = h * (ls[1] * ls[2] - m23 * m23) / det;
            let y11 = h * (ls[0] * ls[2] - m13 * m13) / det;
            let y22 = h * (ls[0] * ls[1] - m12 * m12) / det;
            let y01 = h * (m13 * m23 - ls[2] * m12) / det;
            let y02 = h * (m12 * m23 - ls[1] * m13) / det;
            let y12 = h * (m12 * m13 - ls[0] * m23) / det;
            let d = (h * 150.0 / ls[0]).min(0.5); // primary DCR damping
            ci.i1_prev = (1.0 - d) * ci.i1_prev + y00 * v[0] + y01 * v[1] + y02 * v[2];
            ci.i2_prev = (1.0 - d) * ci.i2_prev + y01 * v[0] + y11 * v[1] + y12 * v[2];
            ci.i3_prev = (1.0 - d) * ci.i3_prev + y02 * v[0] + y12 * v[1] + y22 * v[2];
        }

        self.sample_count += 1;

        let out = self.config.output_node.0;
        Ok(if out > 0 {
            self.solver.v[out] as f32
        } else {
            0.0
        })
    }

    pub fn process_buffer(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), DanjiError> {
        if input.len() != output.len() {
            return Err(DanjiError::BufferSize {
                expected: input.len(),
                actual: output.len(),
            });
        }
        for (i, sample) in output.iter_mut().enumerate() {
            *sample = self.process_sample(input[i])?;
        }
        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.config.num_nodes
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn set_bplus(&mut self, voltage: f64) {
        self.config.bplus_voltage = voltage;
    }

    pub fn set_input2(&mut self, voltage: f64) {
        self.config.input2_voltage = voltage;
    }

    pub fn node_voltage(&self, node: NodeId) -> f32 {
        let n = node.0;
        if n < self.solver.v.len() {
            self.solver.v[n] as f32
        } else {
            0.0
        }
    }
}

pub fn single_triode_config(
    sample_rate: u32,
    plate_resistor: f64,
    cathode_resistor: f64,
    cathode_capacitor: f64,
    grid_resistor: f64,
    bplus: f64,
) -> SimConfig {
    use crate::circuit::node::NodeId;
    let gnd = NodeId(0);
    let grid = NodeId(1);
    let cathode = NodeId(2);
    let plate = NodeId(3);
    let bplus_node = NodeId(4);

    let mut config = SimConfig::new(sample_rate, 5);
    config
        .resistors
        .push(Resistor::new(plate, bplus_node, plate_resistor));
    config
        .resistors
        .push(Resistor::new(cathode, gnd, cathode_resistor));
    config
        .resistors
        .push(Resistor::new(grid, gnd, grid_resistor));
    config.resistors.push(Resistor::new(bplus_node, gnd, 1e6));
    if cathode_capacitor > 0.0 {
        config
            .capacitors
            .push(Capacitor::new(cathode, gnd, cathode_capacitor));
    }
    config.triodes.push(TriodeInstance {
        plate,
        grid,
        cathode,
        params_idx: 0,
    });
    config.input_node = grid;
    config.output_node = plate;
    config.bplus_node = bplus_node;
    config.bplus_voltage = bplus;
    config
}
