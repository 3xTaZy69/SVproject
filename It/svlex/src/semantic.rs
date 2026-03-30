// AI was used here!!!

use crate::parser::*;

pub fn analyse(ast: Vec<Part>) -> Result<(), Vec<String>> {
    let mut __errors__: Vec<String> = Vec::new();
    for part in &ast {
        if let Part::Block(Block::Module { name, ports, code }) = part {
            analyse_module(name, ports.to_vec(), code.to_vec(), &mut __errors__);
        }
    }
    if __errors__.is_empty() { Ok(()) } else { Err(__errors__) }
}

pub fn resolve_decls(ast: Vec<Part>) -> Vec<Part> {
    let snapshot = ast.clone();
    ast.into_iter().map(|part| {
        match part {
            Part::Stmt(Stmt::Decl { target, length, value, vtype }) => {
                let structure = match find_assigning(target.clone(), snapshot.clone()) {
                    Structure::Null => Structure::Const,
                    other => other,
                };
                Part::Stmt(Stmt::Decl { target, length, value, vtype: update_vtype(vtype, structure) })
            }
            Part::Block(Block::Module { name, ports, code }) => {
                Part::Block(Block::Module { name, ports, code: resolve_decls_in_module(code) })
            }
            other => other,
        }
    }).collect()
}

fn resolve_decls_in_module(code: Vec<Part>) -> Vec<Part> {
    let snapshot = code.clone();
    code.into_iter().map(|part| {
        match part {
            Part::Stmt(Stmt::Decl { target, length, value, vtype }) => {
                let structure = match find_assigning(target.clone(), snapshot.clone()) {
                    Structure::Null => Structure::Const,
                    other => other,
                };
                Part::Stmt(Stmt::Decl { target, length, value, vtype: update_vtype(vtype, structure) })
            }
            Part::Block(Block::Module { name, ports, code }) => {
                Part::Block(Block::Module { name, ports, code: resolve_decls_in_module(code) })
            }
            other => other,
        }
    }).collect()
}

fn update_vtype(vtype: Vtypes, structure: Structure) -> Vtypes {
    match vtype {
        Vtypes::Logic(_) => Vtypes::Logic(structure),
        Vtypes::Wire(_)  => Vtypes::Wire(structure),
        Vtypes::Reg(_)   => Vtypes::Reg(structure),
        Vtypes::Int(_)   => Vtypes::Int(structure),
    }
}

fn analyse_module(name: &String, ports: Vec<Inout>, code: Vec<Part>, __errors__: &mut Vec<String>) {
    for port in ports {
        match port {
            Inout::Input(x, y) => {
                if let Structure::Null = find_assigning(x.clone(), code.clone()) {} else {
                    __errors__.push(format!("input port '{}' is driven inside module '{}'", x, name));
                }
            }
            Inout::Output(x, y) => {
                if let Structure::Null = find_assigning(x.clone(), code.clone()) {
                    __errors__.push(format!("output port '{}' is never driven in module '{}'", x, name));
                }
            }
        }
    }
}

fn find_assigning(vname: String, ast: Vec<Part>) -> Structure {
    let mut __result__ = Structure::Null;
    for line in ast {

        if let Structure::Null = __result__ {} else { break; }

        match line {
            Part::Block(block) => match block {
                Block::Module { code, .. } => {
                    __result__ = find_assigning(vname.clone(), code);
                }
                Block::AlwaysComb { code } => {
                    __result__ = find_assigningst(vname.clone(), code, false);
                }
                Block::AlwaysFf { code, .. } => {
                    __result__ = find_assigningst(vname.clone(), code, true);
                }
                Block::Initial { code } => {
                    let s = find_assigningst(vname.clone(), code.clone(), false);
                    if let Structure::Null = s {} else {
                        __result__ = s;
                    }

                    for ln in code.clone() {
                        if let Stmt::Tick { timing, dst } = ln {
                            if dst == vname { __result__ = Structure::Clock(timing) }
                        }
                    }
                }
            },
            Part::Stmt(stmt) => {
                __result__ = check_stmt(vname.clone(), stmt, false);
            }
        }
    }

    if let Structure::Null = __result__ {
        Structure::Const
    } else {
        __result__
    }
}

fn find_assigningst(vname: String, stmts: Vec<Stmt>, in_ff: bool) -> Structure {
    for stmt in stmts {
        let s = check_stmt(vname.clone(), stmt, in_ff);
        if let Structure::Null = s {
            continue;
        } else {
            return s;
        }
    }
    Structure::Null
}

fn check_stmt(vname: String, stmt: Stmt, in_ff: bool) -> Structure {
    match stmt {
        Stmt::ContinuousAssign { target, .. } => {
            if exprtovar(&target) == vname { Structure::Net } else { Structure::Null }
        }
        Stmt::BlockAssign { target, .. } => {
            if exprtovar(&target) == vname {
                if in_ff { Structure::Reg } else { Structure::Net }
            } else {
                Structure::Null
            }
        }
        Stmt::NonBlockAssign { target, .. } => {
            if exprtovar(&target) == vname {
                return Structure::Reg;
            }
            Structure::Null
        }
        Stmt::Case { cases, .. } => {
            for case in cases {
                let body = match case {
                    Case::Value { body, .. } => body,
                    Case::Default { body } => body,
                };
                let s = find_assigningst(vname.clone(), body, in_ff);
                if let Structure::Null = s {} else { return s; }
            }
            Structure::Null
        }
        Stmt::IfStmt { true_code, false_code, .. } => {
            let s = find_assigningst(vname.clone(), true_code, in_ff);
            if let Structure::Null = s {} else { return s; }
            find_assigningst(vname.clone(), false_code, in_ff)
        }
        _ => Structure::Null,
    }
}

fn exprtovar(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::Ref { base, .. } => base.clone(),
        _ => String::new(),
    }
}