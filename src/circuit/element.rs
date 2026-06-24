use crate::circuit::node::NodeId;

#[derive(Debug, Clone)]
pub struct Resistor {
    pub a: NodeId,
    pub b: NodeId,
    pub ohms: f64,
}

impl Resistor {
    pub fn new(a: NodeId, b: NodeId, ohms: f64) -> Self {
        Self { a, b, ohms }
    }
}

#[derive(Debug, Clone)]
pub struct Capacitor {
    pub a: NodeId,
    pub b: NodeId,
    pub farads: f64,
    pub(crate) v_prev: f64,
}

impl Capacitor {
    pub fn new(a: NodeId, b: NodeId, farads: f64) -> Self {
        Self {
            a,
            b,
            farads,
            v_prev: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Inductor {
    pub a: NodeId,
    pub b: NodeId,
    pub henrys: f64,
    pub(crate) i_prev: f64,
}

impl Inductor {
    pub fn new(a: NodeId, b: NodeId, henrys: f64) -> Self {
        Self {
            a,
            b,
            henrys,
            i_prev: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TriodeInstance {
    pub plate: NodeId,
    pub grid: NodeId,
    pub cathode: NodeId,
    pub params_idx: usize,
}

#[derive(Debug, Clone)]
pub struct DiodeInstance {
    pub anode: NodeId,
    pub cathode: NodeId,
    pub params_idx: usize,
}

pub const MAX_NODES: usize = 30;

#[derive(Debug, Clone)]
pub struct CircuitDef {
    pub num_nodes: usize,
    pub resistors: Vec<Resistor>,
    pub capacitors: Vec<Capacitor>,
    pub inductors: Vec<Inductor>,
    pub triodes: Vec<TriodeInstance>,
    pub diodes: Vec<DiodeInstance>,
    pub input_node: NodeId,
    pub output_node: NodeId,
    pub bplus_node: NodeId,
    pub bplus_voltage: f64,
}
