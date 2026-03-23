#![allow(unused)]
mod lex;
use lex::*;
mod parser;
mod netlist;
use parser::*;
mod ir;

use crate::netlist::Bus;


fn main() {
    let text = "

    int a = 1;
    int b;
    case (a)
        0: begin b = 1; end
        4'b0001: begin b = 2; end
        4'b0010: b = 4;
        4'b0011: b = 8;
    endcase

    ".to_string();

    let mut lxr = Lexer::new(text);
    lxr.lex();

    let mut _prs = Parser::new(lxr.tokens.clone());
    let x = _prs.parse();

    for var in &_prs.vars {
        println!("var {}", var)
    }
    println!("{:?}", _prs.vars);
    
    for t in lxr.tokens {
        println!("{t}")
    } 
}


/*
fn main() {

    let clk = netlist::NBlock::new(5, 0, 5, 0, false, 0, Vec::new());

    let edg = netlist::Edge::new_negedge([0,0,0], clk, false);

    let mut v = Vec::new();
    v.push(edg);

    let input = Bus::new(8, [0,0,-2], false, 5);

    let reg = netlist::Reg::new(8, [0,0,5], false);

    reg.addin([0,0,3], v, 8, false, input);

    netlist::assemble();
}
*/