#![allow(unused)]
mod lex;
use lex::*;
mod parser;
mod netlist;
use parser::*;

use crate::netlist::Bus;


fn main() {
    let text = "

    int a = 2;
    int b;
    if (a == 2) b = 0;
    else b = 1;

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