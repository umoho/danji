use crate::circuit::element::{Capacitor, CircuitDef, Inductor, Resistor, TriodeInstance};
use crate::circuit::node::NodeId;
use crate::circuit::solver::CircuitSolver;
use crate::error::DanjiError;
use crate::tube::params::TriodeParams;

pub struct Simulator {
    config: SimConfig,
    solver: CircuitSolver,
    triode_params: Vec<TriodeParams>,
    sample_count: usize,
}

#[derive(Debug, Clone)]
pub struct SimConfig {
    pub sample_rate: u32,
    pub num_nodes: usize,
    pub resistors: Vec<Resistor>,
    pub capacitors: Vec<Capacitor>,
    pub inductors: Vec<Inductor>,
    pub triodes: Vec<TriodeInstance>,
    pub input_node: NodeId,
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
            triodes: Vec::new(),
            input_node: NodeId(0),
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

    pub fn add_triode(&mut self, plate: NodeId, grid: NodeId, cathode: NodeId, params_idx: usize) -> &mut Self {
        self.triodes.push(TriodeInstance { plate, grid, cathode, params_idx });
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

    fn to_circuit_def(&self) -> CircuitDef {
        CircuitDef {
            num_nodes: self.num_nodes,
            resistors: self.resistors.clone(),
            capacitors: self.capacitors.clone(),
            inductors: self.inductors.clone(),
            triodes: self.triodes.clone(),
            input_node: self.input_node,
            output_node: self.output_node,
            bplus_node: self.bplus_node,
            bplus_voltage: self.bplus_voltage,
        }
    }
}

impl Simulator {
    pub fn new(config: SimConfig, triode_params: Vec<TriodeParams>) -> Self {
        let solver = CircuitSolver::new(config.num_nodes);
        Self {
            config,
            solver,
            triode_params,
            sample_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.solver.reset();
        for cap in &mut self.config.capacitors {
            cap.v_prev = 0.0;
        }
        for ind in &mut self.config.inductors {
            ind.i_prev = 0.0;
        }
        self.sample_count = 0;
    }

    pub fn process_sample(&mut self, input: f32) -> Result<f32, DanjiError> {
        let fs = self.config.sample_rate as f64;
        let h = 1.0 / fs;
        let circuit_def = self.config.to_circuit_def();

        self.solver.solve(&circuit_def, &self.triode_params, h, input as f64)?;

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
        }

        self.sample_count += 1;

        let out = self.config.output_node.0;
        Ok(if out > 0 { self.solver.v[out] as f32 } else { 0.0 })
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

    pub fn node_voltage(&self, node: NodeId) -> f32 {
        let n = node.0;
        if n < self.solver.v.len() { self.solver.v[n] as f32 } else { 0.0 }
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
    config.resistors.push(Resistor::new(plate, bplus_node, plate_resistor));
    config.resistors.push(Resistor::new(cathode, gnd, cathode_resistor));
    config.resistors.push(Resistor::new(grid, gnd, grid_resistor));
    config.resistors.push(Resistor::new(bplus_node, gnd, 1e6));
    if cathode_capacitor > 0.0 {
        config.capacitors.push(Capacitor::new(cathode, gnd, cathode_capacitor));
    }
    config.triodes.push(TriodeInstance { plate, grid, cathode, params_idx: 0 });
    config.input_node = grid;
    config.output_node = plate;
    config.bplus_node = bplus_node;
    config.bplus_voltage = bplus;
    config
}
