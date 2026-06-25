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
pub struct CoupledInductor {
    pub p_a: NodeId,
    pub p_b: NodeId,
    pub s_a: NodeId,
    pub s_b: NodeId,
    pub l_primary: f64,
    pub l_secondary: f64,
    pub coupling: f64,
    pub(crate) i1_prev: f64,
    pub(crate) i2_prev: f64,
}

impl CoupledInductor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p_a: NodeId,
        p_b: NodeId,
        s_a: NodeId,
        s_b: NodeId,
        l_primary: f64,
        l_secondary: f64,
        coupling: f64,
    ) -> Self {
        Self {
            p_a,
            p_b,
            s_a,
            s_b,
            l_primary,
            l_secondary,
            coupling,
            i1_prev: 0.0,
            i2_prev: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoupledInductor3 {
    pub p1: NodeId,
    pub ct: NodeId,
    pub p2: NodeId,
    pub s1: NodeId,
    pub s2: NodeId,
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub k12: f64,
    pub k13: f64,
    pub k23: f64,
    pub(crate) i1_prev: f64,
    pub(crate) i2_prev: f64,
    pub(crate) i3_prev: f64,
}

impl CoupledInductor3 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
    ) -> Self {
        Self {
            p1,
            ct,
            p2,
            s1,
            s2,
            l1,
            l2,
            l3,
            k12,
            k13,
            k23,
            i1_prev: 0.0,
            i2_prev: 0.0,
            i3_prev: 0.0,
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

#[derive(Debug, Clone)]
pub struct PentodeInstance {
    pub plate: NodeId,
    pub grid: NodeId,
    pub cathode: NodeId,
    pub screen: NodeId,
    pub params_idx: usize,
}

pub const MAX_NODES: usize = 30;

#[derive(Debug, Clone)]
pub struct CircuitDef {
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
    pub input2_node: NodeId,
    pub input2_voltage: f64,
    pub output_node: NodeId,
    pub bplus_node: NodeId,
    pub bplus_voltage: f64,
}
