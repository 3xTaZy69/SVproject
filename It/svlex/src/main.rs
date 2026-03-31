#![allow(unused)]
mod lex;
use lex::*;
mod parser;
mod netlist;
use parser::*;
mod ir;
mod semantic;
mod graph;

fn main() {
    let text = "
        logic a;
        logic b;
        if (1 == 3) begin
            if (2 == 4) begin
                a = 2;
                b = 3;
            end
        end else begin
            a = 9;
            b = a;
        end
        
    ".to_string();

    let mut lxr = Lexer::new(text);
    lxr.lex();

    let mut _prs = Parser::new(lxr.tokens);
    let y = _prs.parse_decl();
    let y = _prs.parse_decl();
    let x = _prs.parse_if(); 
    _prs.ast.push(Part::Stmt(x));
    _prs.parse();



    semantic::analyse(_prs.ast.clone()).unwrap();
    
    _prs.ast = semantic::resolve_decls(_prs.ast);

    let mut i = ir::Ir {ir: Vec::new(), tmpcounter: 0};
    
    if let Some(resolved_first) = _prs.ast.first() {
        if let Part::Stmt(stmt) = resolved_first {
            i.lower_if(stmt.clone(), None);
        }
    } else {
        println!("AST пустое!");
    }
    
    for j in i.ir {
        println!(
            "{:?}\n------------",
            j
        )
    }
}