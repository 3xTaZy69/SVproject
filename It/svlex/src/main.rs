#![allow(unused)]
mod lex;
use lex::*;
mod parser;
mod netlist;
use parser::*;
mod ir;
mod semantic;

fn main() {
    let text = "
        logic a = 0;
        always_ff @(posedge clk) begin
            a <= 2;
        end
    ".to_string();

    let mut lxr = Lexer::new(text);
    lxr.lex();

    let mut _prs = Parser::new(lxr.tokens);
    let x = _prs.parse_decl(); 
    _prs.ast.push(Part::Stmt(x));
    _prs.parse();



    semantic::analyse(_prs.ast.clone()).unwrap();
    
    _prs.ast = semantic::resolve_decls(_prs.ast);

    let mut i = ir::Ir {ir: Vec::new(), tmpcounter: 0};
    
    if let Some(resolved_first) = _prs.ast.first() {
        if let Part::Stmt(stmt) = resolved_first {
            i.lower_decl(stmt.clone());
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