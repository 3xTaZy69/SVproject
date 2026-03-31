use std::collections::HashMap;

use crate::{ir::*, netlist::*};

#[derive(Clone, Debug)]
pub enum NetlistParts {
    Adder(Adder),
    Mux(Mux),
    Bus(Bus),
    Net(Wire),
    Reg(Reg),
    Cmp(Comparator),
    Edge(Edge)
}

pub struct Graph {
    symtable: HashMap<String, NetlistParts>,
    ir: Vec<SA>
}

impl Graph {
    pub fn var(&mut self, v: SA) {
        let name = v.target;

    }

    pub fn get_ref(&mut self, a: A) -> Bus {
        if let A::Ref { base, h, l } = a {
            let v = self.symtable.get(&base);
            let v = match v {
                None => panic!("CANT GET REF FROM A NOT EXISTING VARIABLE"),
                Some(x) => x.clone()
            };
            let b = match v {
                NetlistParts::Adder(x) => {
                    x.output
                }
                NetlistParts::Bus(x) => {
                    x
                }
                NetlistParts::Mux(x) => {
                    x.output
                }
                NetlistParts::Reg(x) => {
                    x.contains
                }
                NetlistParts::Net(x) => {
                    x.contains
                }
                _ => panic!("CANT SLICE: {v:?}")
            };
            let len = b.len();
            b.slice_bus(h as u64, (len - l) as u64)
        } else {
            panic!("EXPECTED REF, got {a:?}")
        }
    }
}