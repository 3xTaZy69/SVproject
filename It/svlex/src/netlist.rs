#![allow(dead_code)]

use std::{cmp::min, collections::HashMap, ops::{self, Add}, sync::Mutex};
use lazy_static::lazy_static;

#[derive(Clone)]

pub struct NBlock {
    pub nl_index: u32,
    bl_index: u8,
    x: i32,
    y: i32,
    z: i32,
    fixed: bool,
    active: u8,
    args: Vec<String>
}

static NL: Mutex<u32> = Mutex::new(0);
lazy_static! { 
    static ref BLOCKS: Mutex<Vec<NBlock>> = Mutex::new(Vec::new()); 
    static ref CONNECTIONS: Mutex<Vec<Connection>> = Mutex::new(Vec::new()); 
}


impl NBlock {
    pub fn new(bl_index: u8, x: i32, y: i32, z: i32, fixed: bool, active: u8, args: Vec<String>) -> NBlock {
        let mut nl = NL.lock().unwrap();
        *nl += 1;
        let b = NBlock { nl_index: *nl, 
            bl_index: bl_index, 
            x: x, 
            y: y, 
            z: z, 
            fixed: fixed,
            active: active,
            args: args,
        };
        BLOCKS.lock().unwrap().push(b.clone());
        b

    }
    fn text(&self) -> String {
        if self.bl_index == 5 {
            format!("{},{},{},{},{},0+0", self.bl_index, self.active, self.x, self.y, self.z)
        } else if self.args.is_empty() {
            format!("{},{},{},{},{},", self.bl_index, self.active, self.x, self.y, self.z)
        } else {
            format!("{},{},{},{},{},{}", self.bl_index, self.active, self.x, self.y, self.z, self.args.join("+"))
        }
    }
}
#[derive(Clone, Copy)]
pub struct Connection {
    lhs: u32,
    rhs: u32,
}

impl Connection {
    pub fn new(lhs: u32, rhs: u32) -> Connection {
        let c = Connection { lhs: lhs, rhs: rhs };
        CONNECTIONS.lock().unwrap().push(c);
        c
    }
    fn text(&self) -> String {
        format!("{},{}", self.lhs, self.rhs)
    }
}

pub fn connect(left: u32, right: u32) -> Connection {
    Connection::new(left, right)
}

#[derive(Clone)]
pub struct Bus {
    width: u32,
    contains: Vec<NBlock>
}


impl Bus {
    pub fn new(width: u32, pos: [i32; 3], fixed: bool, block_type: u8) -> Bus {
        let mut x = pos[0];
        let y = pos[1];
        let z = pos[2];

        let mut blocks: Vec<NBlock> = Vec::new();

        for _ in 0..width {
            let block = NBlock::new(block_type, x, y, z, fixed, 0, Vec::new());
            x += 1;
            blocks.push(block);
        }

        Bus { width, contains: blocks }
    }
    pub fn reg_mask(width: u32, pos: [i32; 3], fixed: bool, mask: String) -> Bus {
        let mut x = 0;
        let y = 0;
        let z = 0;

        let mut blocks: Vec<NBlock> = Vec::new();

        for i in 0..pos[0] {
            let idx = i as usize;
            if &mask[idx..idx+1] == "1" {
                let block = NBlock::new(5, x, y, z, fixed, 1, Vec::new());
                x += 1;
                blocks.push(block);
            } else {
                let block = NBlock::new(5, x, y, z, fixed, 0, Vec::new());
                x += 1;
                blocks.push(block);
            }
        }

        Bus { width, contains: blocks }
    }
    pub fn new_mask(width: u32, pos: [i32; 3], fixed: bool, mask: String) -> Bus {
        let mut x = 0;
        let y = 0;
        let z = 0;

        let mut blocks: Vec<NBlock> = Vec::new();

        for i in 0..pos[0] {
            let idx = i as usize;
            if &mask[idx..idx+1] == "1" {
                let block = NBlock::new(0, x, y, z, fixed, 0, Vec::new());
                x += 1;
                blocks.push(block);
            } else {
                let block = NBlock::new(15, x, y, z, fixed, 0, Vec::new());
                x += 1;
                blocks.push(block);
            }
        }

        Bus { width, contains: blocks }
    }
    fn get_msb(&self) -> NBlock {
        self.contains[0].clone()
    }
    fn get_lsb(&self) -> NBlock {
        let ln = self.contains.len()-1;
        self.contains[ln].clone()
    }
    fn get_msbi(&self) -> u32 {
        self.contains[0].nl_index
    }
    fn get_lsbi(&self) -> u32 {
        let ln = self.contains.len()-1;
        self.contains[ln].nl_index
    }
    fn slice_bus(&self, h: u64, l: u64) -> Bus {
        let slice: Vec<NBlock> = self.contains[h as usize..l as usize].iter().cloned().collect();
        Bus { width: slice.len() as u32, contains: slice }
    }
    fn get(&self, index: u32) -> NBlock {
        self.contains[index as usize].clone()
    }
    fn concat(&self, other: Bus) -> Bus {
        let mut newcontains = self.contains.clone();
        newcontains.extend(other.contains);
        Bus { width: newcontains.len() as u32, contains: newcontains }
    }
    fn connect_from_one(&self, other: NBlock) {
        for n in &self.contains {
            connect(other.nl_index, n.nl_index);
        }
    }
    fn connect_logic_bitwise_by(&self, other: Bus) {
        for self_bit in 0..other.width as usize {
            for other_bit in self_bit..other.width as usize {
                connect(self.contains[self_bit].nl_index, other.contains[other_bit].nl_index);
            }
        }
    }

}

pub struct Reg {
    width: u32,
    contains: Bus
}

impl Reg {
    pub fn new(width: u32, pos: [i32; 3], fixed: bool) -> Reg {
        let contains = Bus::new(width, pos, fixed, 5);
        Reg {width: width, contains: contains}
    }
    fn mask_new(width: u32, pos: [i32; 3], fixed: bool, mask: String) -> Reg {
        let contains = Bus::reg_mask(width, pos, fixed, mask);
        Reg {width: width, contains: contains}
    }
    fn get_msb(&self) -> NBlock {
        self.contains.get_msb()
    }
    fn get_lsb(&self) -> NBlock {
        self.contains.get_lsb()
    }
    fn get_msbi(&self) -> u32 {
        self.contains.get_msbi()
    }
    fn get_lsbi(&self) -> u32 {
        self.contains.get_lsbi()
    }
    fn slice_bus(&self, h: u64, l: u64) -> Bus {
        self.contains.slice_bus(h, l)
    }
    fn slice_reg(&self, h: u64, l: u64) -> Reg {
        let slice = self.contains.slice_bus(h, l);
        Reg { width: slice.contains.len() as u32, contains: slice}
    }
    fn get(&self, index: u32) -> NBlock {
        self.contains.contains[index as usize].clone()
    }
    fn adddff(&self, dff: Dff) {
        dff.output.connect_bitwise(&self.contains);
    }
    pub fn addin(&self, pos: [i32; 3], edges: Vec<Edge>, width: u32, fixed: bool, input: Bus)  {
        let [x, y, z] = pos;
        let xor = Bus::new(width, [x,y,z], fixed, 3);
        let and = Bus::new(width, [x,y,z-1], fixed, 1);
        let edgein = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new());
        &self.contains + &xor;
        &xor + &and;
        &and + &self.contains;
        &input + &xor;
        and.connect_from_one(edgein.clone());
        for edge in edges {
            &edge.output + &edgein;
        }
    }
}

struct Wire {
    width: u32,
    contains: Bus
}

impl Wire {
   fn  new(width: u32, pos: [i32; 3], fixed: bool) -> Wire {
    let contains = Bus::new(width, pos, fixed, 15);
    Wire { width: width, contains: contains }
   }
   fn get_msb(&self) -> NBlock {
        self.contains.get_msb()
    }
    fn get_lsb(&self) -> NBlock {
        self.contains.get_lsb()
    }
    fn get_msbi(&self) -> u32 {
        self.contains.get_msbi()
    }
    fn get_lsbi(&self) -> u32 {
        self.contains.get_lsbi()
    }
    fn slice_bus(&self, h: u64, l: u64) -> Bus {
        self.contains.slice_bus(h, l)
    }
    fn slice_wire(&self, h: u64, l: u64) -> Wire {
        let slice = self.contains.slice_bus(h, l);
        Wire {width: slice.contains.len() as u32, contains: slice}
    }
}

pub trait Bits {
    fn connect_bitwise(&self, other: &dyn Bits) {}
    fn connect_logic(&self, other: NBlock) {}
    fn get(&self, index: u32) -> NBlock {unimplemented!()}
    fn len(&self) -> u32 {unimplemented!()}
}

impl Bits for Reg {
    fn connect_bitwise(&self, other: &dyn Bits) {
        for i in 0..self.width {
            connect(self.contains.contains[i as usize].nl_index, other.get(i).nl_index);
        }
    }
    fn get(&self, index: u32) -> NBlock {
        self.contains.contains[index as usize].clone()
    }
    fn connect_logic(&self, other: NBlock) {
        for i in 0..self.width {
            connect(self.contains.contains[i as usize].nl_index, other.nl_index);
        }
    }
    fn len(&self) -> u32 {
        self.contains.contains.len() as u32
    }
}

impl Bits for Wire {
    fn connect_bitwise(&self, other: &dyn Bits) {
        for i in 0..self.width {
            connect(self.contains.contains[i as usize].nl_index, other.get(i).nl_index);
        }
    }
    fn get(&self, index: u32) -> NBlock {
        self.contains.contains[index as usize].clone()
    }
    fn connect_logic(&self, other: NBlock) {
        for i in 0..self.width {
            connect(self.contains.contains[i as usize].nl_index, other.nl_index);
        }
    }
    fn len(&self) -> u32 {
        self.contains.contains.len() as u32
    }
}

impl Bits for Bus {
    fn connect_bitwise(&self, other: &dyn Bits) {
        let ln = min(self.contains.len() as u32, other.len());
        for i in 0..ln {
            connect(self.contains[i as usize].nl_index, other.get(i).nl_index);
        }
    }
    fn get(&self, index: u32) -> NBlock {
        self.contains[index as usize].clone()
    }
    fn connect_logic(&self, other: NBlock) {
        for i in 0..self.width {
            connect(self.contains[i as usize].nl_index, other.nl_index);
        }
    }
    fn len(&self) -> u32 {
        self.contains.len() as u32
    }
}

pub fn assemble() {
    let blocks: Vec<NBlock> = BLOCKS.lock().unwrap().iter().cloned().collect();
    let mut output_blocks : Vec<String> = Vec::new();
    for block in blocks {
        output_blocks.push(block.text())
    }
    let mut output_connections : Vec<String> = Vec::new();
    let connections: Vec<Connection> = CONNECTIONS.lock().unwrap().iter().cloned().collect();
    for connection in connections {
        output_connections.push(connection.text())
    }
    println!("{}?{}??", output_blocks.join(";"), output_connections.join(";"))
}

pub struct Adder {
    width: u32,
    output: Bus,
    cout: NBlock
}

impl Adder {
    pub fn new(pos: [i32; 3], width: u32, fixed: bool, sub: bool) -> Adder {
        let [mut x, mut y, mut z] = pos;
        let a = Bus::new(width, [x,y,z-1], fixed, 15);
        let b;
        if !sub {
            b = Bus::new(width, [x,y,z], fixed, 15);
        } else {
            b = Bus::new(width, [x,y,z], fixed, 0);
        }

        let mut prevg = Bus::new(width, [x,y,z-2], fixed, 1);
        let mut prevp = Bus::new(width, [x,y,z-3], fixed, 3);
        let p = prevp.clone();

        &a + &prevg;
        &a + &prevp;
        &b + &prevp;
        &b + &prevg;

        let levels = (width as f32).log2().ceil() as u32;
        let mut free = 1;
        let mut logic = width - 1;
        let mut newp;
        let mut newgor;
        let mut newgand;
        let mut newfp;
        let mut newfg;

        z -= 4;

        for level in 1..levels+1 {
            logic = width - free;

            newp = Bus::new(logic, [x,y,z], fixed, 1);
            newfp = Bus::new(free, [x+logic as i32,y,z], fixed, 15);

            let tmpp = newp.concat(newfp);
            prevp.connect_bitwise(&tmpp);

            prevp.slice_bus(free as u64, (width) as u64).connect_bitwise(&newp);

            newgand = Bus::new(logic, [x,y,z-1], fixed, 1);
            newgor = Bus::new(logic, [x,y+1,z-1], fixed, 15);

            newfg = Bus::new(free, [x+logic as i32,y+1,z-1], fixed, 15);

            prevp.connect_bitwise(&newgand);
            prevg.slice_bus(free as u64, (width) as u64).connect_bitwise(&newgand);
            
            let tmpg = newgor.concat(newfg);
            prevg.connect_bitwise(&tmpg);
            newgand.connect_bitwise(&newgor);

            prevg = tmpg;
            prevp = tmpp;
            z -= 2;
            free *= 2;
            
        }

        let carryand = Bus::new(width, [x,y,z], fixed, 1);
        let carryor = Bus::new(width, [x,y+1,z], fixed, 15);
        carryand.connect_bitwise(&carryor);
        let cout;
        if !sub {
            cout = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new())
        } else {
            cout = NBlock::new(0, x+width as i32, y, z, fixed, 0, Vec::new())
        }

        carryand.connect_from_one(cout.clone());
        prevp.connect_bitwise(&carryand);
        prevg.connect_bitwise(&carryor);

        let out = Bus::new(width, [x,y,z-2], fixed, 3);

        p.connect_bitwise(&out);
        carryor.slice_bus(1, width as u64).connect_bitwise(&out);
        connect(cout.nl_index, out.get(width-1).nl_index);

        Adder { width, output: out, cout: carryor.get(0) }

    }
}

impl ops::Add for &Bus {
    type Output = ();

    fn add(self, rhs: Self) -> Self::Output {
        self.connect_bitwise(rhs);
    }
}

impl ops::Add for &NBlock {
    type Output = ();

    fn add(self, rhs: Self) -> Self::Output {
        connect(self.nl_index, rhs.nl_index);
    }
}

enum Edges {
    Posedge,
    Negedge
}

pub struct Edge {
    etype: Edges,
    
    output: NBlock,
}

impl Edge {
    pub fn new_posedge(pos: [i32; 3], clock: NBlock, fixed: bool) -> Edge {
        let [x, y, z] = pos;
        let n = NBlock::new(0, x, y, z, fixed, 1, Vec::new());
        let a = NBlock::new(1, x+1, y, z, fixed, 0, Vec::new());
        &n + &a;
        &clock + &n;
        &clock + &a;
        Edge { etype: Edges::Posedge, output: a }
    }
    pub fn new_negedge(pos: [i32; 3], clock: NBlock, fixed: bool) -> Edge {
        let [x, y, z] = pos;
        let a = NBlock::new(1, x, y, z, fixed, 0, Vec::new());
        let n1 = NBlock::new(0, x-1, y, z, fixed, 0, Vec::new());
        let o = NBlock::new(2, x-2, y, z, fixed, 0, Vec::new());
        let n2 = NBlock::new(0, x-2, y, z, fixed, 0, Vec::new());

        &n2 + &a;
        &n1 + &a;
        &o + &n1;
        &clock + &o;
        &n1 + &n2;
        Edge { etype: Edges::Negedge, output: a }
    }
}

pub struct Dff {
    output: Bus,
    edgein: NBlock,
    width: u32
}

impl Dff {
    pub fn new(pos: [i32; 3], edges: Vec<Edge>, width: u32, fixed: bool, input: Bus) -> Dff {
        let [x, y, z] = pos;
        let xor = Bus::new(width, [x,y,z], fixed, 3);
        let and = Bus::new(width, [x,y,z-1], fixed, 1);
        let tff = Bus::new(width, [x,y,z-2], fixed, 5);
        let edgein = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new());
        &tff + &xor;
        &xor + &and;
        &and + &tff;
        &input + &xor;
        and.connect_from_one(edgein.clone());
        for edge in edges {
            &edge.output + &edgein;
        }

        Dff { output: tff, edgein: edgein, width: width }
    }
}

pub struct Comparator {
    pub output: NBlock
}

impl Comparator {
    pub fn new_eq(pos: [i32; 3], a: Bus, b: Bus, fixed: bool, width: u32) -> Comparator {
        let xnor = Bus::new(width, pos, fixed, 11);
        let out = NBlock::new(1, pos[0]+width as i32, pos[1], pos[2], fixed, 0, Vec::new());
        xnor.connect_logic(out.clone());
        &a + &xnor;
        &b + &xnor;
        Comparator { output: out }
    }
    pub fn new_neq(pos: [i32; 3], a: Bus, b: Bus, fixed: bool, width: u32) -> Comparator {
        let xnor = Bus::new(width, pos, fixed, 11);
        let out = NBlock::new(10, pos[0]+width as i32, pos[1], pos[2], fixed, 0, Vec::new());
        xnor.connect_logic(out.clone());
        &a + &xnor;
        &b + &xnor;
        Comparator { output: out }
    }
    pub fn new_gt(pos: [i32; 3], a: Bus, b: Bus, fixed: bool, width: u32) -> Comparator {
        let [x, y, z] = pos;
        let xor = Bus::new(width, pos, fixed, 3);
        let and = Bus::new(width, [x,y,z-1], fixed, 1);
        let not = Bus::new(width-1, [x+1,y,z-2], fixed, 0);
        let out = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new());

        &a + &xor;
        &b + &xor;
        &a + &and;
        &xor + &and;
        and.connect_logic(out.clone());
        let andnot = and.slice_bus(1, width as u64);
        &not + &andnot;
        xor.connect_logic_bitwise_by(not);

        Comparator { output: out }

    }
    pub fn new_lt(pos: [i32; 3], a: Bus, b: Bus, fixed: bool, width: u32) -> Comparator {
        let [x, y, z] = pos;
        let xor = Bus::new(width, pos, fixed, 3);
        let and = Bus::new(width, [x,y,z-1], fixed, 1);
        let not = Bus::new(width-1, [x+1,y,z-2], fixed, 0);
        let out = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new());

        &a + &xor;
        &b + &xor;
        &b + &and;
        &xor + &and;
        and.connect_logic(out.clone());
        let andnot = and.slice_bus(1, width as u64);
        &not + &andnot;
        xor.connect_logic_bitwise_by(not);

        Comparator { output: out }

    }
    pub fn new_ge(pos: [i32; 3], a: Bus, b: Bus, fixed: bool, width: u32) -> Comparator {
        let [x, y, mut z] = pos;
        let eq  = Comparator::new_eq(pos, a.clone(), b.clone(), fixed, width);
        z -= 1;
        let xor = Bus::new(width, pos, fixed, 3);
        let and = Bus::new(width, [x,y,z-1], fixed, 1);
        let not = Bus::new(width-1, [x+1,y,z-2], fixed, 0);
        let out = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new());

        &a + &xor;
        &b + &xor;
        &a + &and;
        &xor + &and;
        and.connect_logic(out.clone());
        let andnot = and.slice_bus(1, width as u64);
        &not + &andnot;
        &eq.output + &out;
        xor.connect_logic_bitwise_by(not);

        Comparator { output: out }

    }
    pub fn new_le(pos: [i32; 3], a: Bus, b: Bus, fixed: bool, width: u32) -> Comparator {
        let [x, y, mut z] = pos;
        let eq  = Comparator::new_eq(pos, a.clone(), b.clone(), fixed, width);
        z -= 1;
        let xor = Bus::new(width, pos, fixed, 3);
        let and = Bus::new(width, [x,y,z-1], fixed, 1);
        let not = Bus::new(width-1, [x+1,y,z-2], fixed, 0);
        let out = NBlock::new(15, x+width as i32, y, z, fixed, 0, Vec::new());

        &a + &xor;
        &b + &xor;
        &b + &and;
        &xor + &and;
        and.connect_logic(out.clone());
        let andnot = and.slice_bus(1, width as u64);
        &not + &andnot;
        &eq.output + &out;
        xor.connect_logic_bitwise_by(not);

        Comparator { output: out }

    }
}

pub struct Mux {
    output: Bus
}

impl Mux {
    pub fn new(true_val: Bus, false_val: Bus, sel: NBlock, pos: [i32; 3], fixed: bool, width: u32) -> Mux {
        let [x, y, z] = pos;
        let t = Bus::new(width, pos, fixed, 1);
        let f = Bus::new(width, [x,y,z-1], fixed, 1);
        let nsel = NBlock::new(0, x-1, y, z, fixed, 1, Vec::new());
        let o = Bus::new(width, [x,y,z-2], fixed, 15);
        &t + &o;
        &f + &o;
        &true_val + &t;
        &false_val + &f;
        t.connect_from_one(sel.clone());
        f.connect_from_one(nsel.clone());
        &sel + &nsel;
        Mux { output: o }
    }
}