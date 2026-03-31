/*

Static Assignment IR ( Not single assignment )

A lot of AI(Claude) was used here!!!

*/

use crate::parser::{self, *};
#[derive(Debug)]
pub enum BinOp {
    BitwiseAnd,
    BitwiseOr,
    Xor,
    LogicAnd,
    LogicOr,
    Add,
    Sub,
    Lsh,
    Rsh,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge
}
#[derive(Debug)]
pub enum UnOp {
    BitwiseNot,
    LogicNot,
    Neg
}
#[derive(Debug)]
pub struct SA {
    pub target: String,
    pub value: A
}

#[derive(Debug)]
pub enum A {
    Reg { len: u32, value: u64, signed: bool },
    Net { len: u32, signed: bool },
    Clock { timing: u32 },
    Const { len: u32, value: u64, signed: bool },
    UnOp { operation: UnOp, operand: String },
    BinOp { left: String, right: String, op: BinOp },
    Dff { clocks: Vec<Edge>, input: String }, /*
        not real "d flip flop", just specification for register input
     */
    Mux { sel: String, trueval: String, falseval: String },
    Block { from: String },
    Assign { from: String },
    Fixed { x: i32, y: i32, z: i32 },
    Ref { base: String, h: u32, l: u32},
}
#[derive(Debug)]
pub struct Ir {
    pub ir: Vec<SA>,
    pub tmpcounter: u32,
}

impl Ir {
    fn gettmp(&mut self) -> String {
        self.tmpcounter += 1;
        format!(
            "%{}",
            self.tmpcounter-1
        )
    }
    fn log(&mut self, i: u64) -> u32 {
        i.ilog2()
    }
    fn poptoirop(&mut self, op: parser::BinOps) -> BinOp {
        match op {
            BinOps::Add => BinOp::Add,
            BinOps::Sub => BinOp::Sub,
            BinOps::ShiftL => BinOp::Lsh,
            BinOps::ShiftR => BinOp::Rsh,
            BinOps::BitWiseAnd => BinOp::BitwiseAnd,
            BinOps::BitWiseOr => BinOp::BitwiseOr,
            BinOps::Xor => BinOp::Xor,
            BinOps::LogicAnd => BinOp::LogicAnd,
            BinOps::LogicOr => BinOp::LogicOr,
            BinOps::Eq => BinOp::Eq,
            BinOps::Neq => BinOp::Neq,
            BinOps::Ge => BinOp::Ge,
            BinOps::Gt => BinOp::Gt,
            BinOps::Le => BinOp::Le,
            BinOps::Lt => BinOp::Lt,
            _ => panic!("NOT SUPPORTED OPERATION: {:?}", op)
        }
    }
    fn unoptoirop(&mut self, op: parser::UnOps) -> UnOp {
        match op {
            UnOps::Neg => UnOp::Neg,
            UnOps::BitWiseNot => UnOp::BitwiseNot,
            UnOps::LogicNot => UnOp::LogicNot,
        }
    }
    pub fn lower_exp(&mut self, expr: Expr) -> String {
        match expr {
            Expr::Ident(x) => x,
            Expr::Int(i) => {
                let tmp = self.gettmp();
                let mut len = if i == 0 {
                    1
                } else {
                    i.ilog2() + 2
                };
                self.ir.push(SA { target: tmp.clone(), value: A::Const { len: len, value: i, signed: true } });
                tmp
            }
            Expr::BinExpr { left, op, right } => {
                let l = self.lower_exp(*left);
                let r = self.lower_exp(*right);
                let opr = self.poptoirop(op);
                let tmp = self.gettmp();
                self.ir.push(SA { target: tmp.clone(), value: A::BinOp { left: l.clone(), right: r.clone(), op: opr } });
                tmp
            }
            Expr::Imm { value, width } => {
                let tmp = self.gettmp();
                self.ir.push(SA { target: tmp.clone(), value: A::Const { len: width+1, value: value, signed: false } });
                tmp
            }
            Expr::Ref { base, h, l } => {
                let tmp = self.gettmp();
                self.ir.push(SA { target: tmp.clone(), value: A::Ref { base: base.clone(), h: h as u32, l: l as u32 } });
                tmp
            }
            Expr::UnExpr { operand, op } => {
                let unoperand = self.lower_exp(*operand);
                let opr = self.unoptoirop(op);
                let tmp = self.gettmp();
                self.ir.push(SA { target: tmp.clone(), value: A::UnOp { operation: opr, operand: unoperand.clone() }});
                tmp
            }
            Expr::Parameter { name, value } => {
                panic!("PARAMETER NOT SUPPORTED")
            }
        }
    }
    pub fn number_expr(&mut self, expr: Expr) -> u64 {
        match expr {
            Expr::Int(i) => {
                i
            }
            Expr::Imm { value, width } => {
                value
            }
            _ => panic!("EXPECTED NUMBER, GOT EXPRESSION")
        }
    }
    pub fn lower_decl(&mut self, decl: Stmt) -> String {
        if let Stmt::Decl { target, length, value, vtype } = decl.clone() {
            let st = match vtype {
                Vtypes::Int(x) => x,
                Vtypes::Reg(x) => x,
                Vtypes::Logic(x) => x,
                Vtypes::Wire(x) => x,
            };
            let mut expr;
            let mut signed;
            if let Expr::Imm { value, width } = value.clone() {
                signed = false;
            } else {
                signed = true;
            }
            if matches!(st, Structure::Const | Structure::Reg ) {
                expr = self.number_expr(value);
            } else {
                expr = 0;
            }
            self.ir.push(SA { target: target.clone(), value:
                match st {
                    Structure::Clock(tm) => A::Clock { timing: tm },
                    Structure::Const => A::Const { len: length as u32, value: expr, signed: signed },
                    Structure::Net => A::Net { len: length as u32, signed },
                    Structure::Reg => A::Reg { len: length as u32, value: expr, signed },
                    Structure::Null => panic!("NULL STRUCTURE CANT BE IN CICUIT, {decl:?}")
                }
             });
             target
        } else {
            panic!("EXPECTED DECL, GOT {decl:?}")
        }
    }
    pub fn exprtostr(&mut self, expr: Expr) -> String {
        match expr {
            Expr::Ident(x) => x,
            Expr::Ref { base, h, l } => {
                let tmp = self.gettmp();
                self.ir.push(SA { target: tmp.clone(), value: A::Ref { base: base.clone(), h: h as u32, l: l as u32 } });
                tmp
            }
            _ => panic!("EXPECTED VARIABLE OR REF, GOT {expr:?}")
        }
    }
    pub fn lower_assign(&mut self, assign: Stmt, ffclocks: Option<Vec<Edge>> ) -> String {

        let sa = match assign {
            Stmt::BlockAssign { target, value } => {
                let name = self.exprtostr(target);
                let value = self.lower_exp(value);
                SA {target: name, value: A::Block { from: value }}
            }
            Stmt::ContinuousAssign { target, value } => {
                let name = self.exprtostr(target);
                let value = self.lower_exp(value);
                SA {target: name, value: A::Assign { from: value }}
            }
            Stmt::NonBlockAssign { target, value } => {
                let name = self.exprtostr(target);
                let value = self.lower_exp(value);
                SA {target: name, value: A::Dff { clocks: ffclocks.unwrap(), input: value.clone() }}
            }
            _ => panic!("EXPECTED ASSIGN, GOT {assign:?}")
        };

        String::new()
    }
    // made with AI
    pub fn lower_code(&mut self, stmts: Vec<Stmt>, ffclocks: Option<Vec<Edge>>) {
        for stmt in stmts {
            match stmt.clone() {
                Stmt::Decl { .. } => {
                    self.lower_decl(stmt);
                }
                Stmt::BlockAssign { .. } 
                | Stmt::NonBlockAssign { .. }
                | Stmt::ContinuousAssign { .. } => {
                    self.lower_assign(stmt, ffclocks.clone());
                }
                Stmt::IfStmt { .. } => {
                    self.lower_if(stmt, ffclocks.clone());
                }
                Stmt::Case { expr, cases } => {
                    self.lower_case(stmt, ffclocks.clone())
                }
                _ => {}
            }
        }
    }

    fn subst_expr(&mut self, expr: Expr, map: &std::collections::HashMap<String, String>) -> Expr {
        match expr {
            Expr::Ident(name) => {
                Expr::Ident(map.get(&name).cloned().unwrap_or(name))
            }
            Expr::BinExpr { left, op, right } => Expr::BinExpr {
                left:  Box::new(self.subst_expr(*left,  map)),
                op,
                right: Box::new(self.subst_expr(*right, map)),
            },
            Expr::UnExpr { operand, op } => Expr::UnExpr {
                operand: Box::new(self.subst_expr(*operand, map)),
                op,
            },
            other => other,
        }
    }

    fn collect_targets(&mut self, stmts: &Vec<Stmt>) -> std::collections::HashSet<String> {
        let mut targets = std::collections::HashSet::new();
        for stmt in stmts {
            match stmt {
                Stmt::BlockAssign { target, .. }
                | Stmt::NonBlockAssign { target, .. } => {
                    let name = match target {
                        Expr::Ident(x) => x.clone(),
                        Expr::Ref { base, .. } => base.clone(),
                        _ => continue,
                    };
                    targets.insert(name);
                }
                Stmt::IfStmt { true_code, false_code, .. } => {
                    targets.extend(self.collect_targets(true_code));
                    targets.extend( self.collect_targets(false_code));
                }
                _ => {}
            }
        }
        targets
    }

    fn collect_branch(
        &mut self,
        stmts: Vec<Stmt>,
        ffclocks: Option<Vec<Edge>>,
    ) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();

        for stmt in stmts {
            match stmt {
                Stmt::IfStmt { expr, true_code, false_code } => {
                    let inner_targets = self.collect_targets(&true_code)
                        .into_iter()
                        .chain(self.collect_targets(&false_code));
                    
                    self.lower_if(
                        Stmt::IfStmt { expr, true_code, false_code },
                        ffclocks.clone()
                    );
                    
                    for k in inner_targets {
                        map.insert(k.clone(), k.clone());
                    }
                }
                Stmt::BlockAssign { ref target, ref value } => {
                    let name = self.exprtostr(target.clone());
                    let subst = self.subst_expr(value.clone(), &map);
                    let val   = self.lower_exp(subst);
                    map.insert(name, val);
                }
                Stmt::NonBlockAssign { ref target, ref value } => {
                    let name = self.exprtostr(target.clone());
                    let subst = self.subst_expr(value.clone(), &map);
                    let val   = self.lower_exp(subst);
                    map.insert(name, val);
                }
                other => {
                    self.lower_code(vec![other], ffclocks.clone());
                }
            }
        }

        map
    }

    pub fn lower_if(&mut self, ifst: Stmt, ffclocks: Option<Vec<Edge>>) {
        if let Stmt::IfStmt { expr, true_code, false_code } = ifst {
            let sel = self.lower_exp(expr);

            let then_map = self.collect_branch(true_code,  ffclocks.clone());
            let else_map = self.collect_branch(false_code, ffclocks.clone());

            let all_targets: Vec<String> = then_map.keys()
                .chain(else_map.keys())
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            for target in all_targets {
                let mux = match (then_map.get(&target), else_map.get(&target)) {
                    (Some(t), Some(f)) => A::Mux {
                        sel: sel.clone(),
                        trueval:  t.clone(),
                        falseval: f.clone(),
                    },
                    (Some(t), None) => A::Mux {
                        sel: sel.clone(),
                        trueval:  t.clone(),
                        falseval: target.clone(),
                    },
                    (None, Some(f)) => A::Mux {
                        sel: sel.clone(),
                        trueval:  target.clone(),
                        falseval: f.clone(),
                    },
                    (None, None) => unreachable!(),
                };
                self.ir.push(SA { target, value: mux });
            }
        } else {
            panic!("IF EXPECTED, GOT {:?}", ifst)
        }
    }
    pub fn lower_case(&mut self, casest: Stmt, ffclocks: Option<Vec<Edge>>) {
        if let Stmt::Case { expr, cases } = casest {


            let case_expr = self.lower_exp(expr);


            let mut default_body: Vec<Stmt> = Vec::new();
            let mut value_cases: Vec<(Expr, Vec<Stmt>)> = Vec::new();

            for case in cases {
                match case {
                    Case::Default { body } => {
                        default_body = body;
                    }
                    Case::Value { value, body } => {
                        value_cases.push((value, body));
                    }
                }
            }


            let mut result_map = self.collect_branch(default_body, ffclocks.clone());


            for (case_val, case_body) in value_cases.into_iter().rev() {


                let val_tmp = self.lower_exp(case_val);
                let eq_tmp  = self.gettmp();
                self.ir.push(SA {
                    target: eq_tmp.clone(),
                    value: A::BinOp {
                        left:  case_expr.clone(),
                        right: val_tmp,
                        op:    BinOp::Eq,
                    }
                });

                let branch_map = self.collect_branch(case_body, ffclocks.clone());

                let all_targets: Vec<String> = result_map.keys()
                    .chain(branch_map.keys())
                    .cloned()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                let mut new_map = std::collections::HashMap::new();

                for target in all_targets {
                    let mux_tmp = self.gettmp();

                    let mux = match (branch_map.get(&target), result_map.get(&target)) {

                        (Some(t), Some(f)) => A::Mux {
                            sel:      eq_tmp.clone(),
                            trueval:  t.clone(),
                            falseval: f.clone(),
                        },

                        (Some(t), None) => A::Mux {
                            sel:      eq_tmp.clone(),
                            trueval:  t.clone(),
                            falseval: target.clone(),
                        },

                        (None, Some(f)) => A::Mux {
                            sel:      eq_tmp.clone(),
                            trueval:  target.clone(),
                            falseval: f.clone(),
                        },
                        (None, None) => unreachable!(),
                    };

                    self.ir.push(SA { target: mux_tmp.clone(), value: mux });
                    new_map.insert(target, mux_tmp);
                }

                result_map = new_map;
            }


            for (target, from) in result_map {
                self.ir.push(SA {
                    target,
                    value: A::Assign { from }
                });
            }

        } else {
            panic!("CASE EXPECTED, GOT {:?}", casest)
        }
    }
    pub fn lower_fixed(&mut self, fx: Stmt) {
        if let Stmt::Fixed { x, y, z, var } = fx {
            self.ir.push(SA { target: var, value: A::Fixed { x, y, z } })
        } else {
            panic!("EXPECTED FIXED, GOT {fx:?}")
        }
    }
    pub fn lower(&mut self, parts: Vec<Part>) {
        for part in parts {
            match part {
                Part::Stmt(stmt) => {
                    match stmt {
                        Stmt::Decl { .. }  => { self.lower_decl(stmt); }
                        Stmt::Fixed { .. } => { self.lower_fixed(stmt); }
                        _ => {}
                    }
                }
                Part::Block(block) => {
                    match block {
                        Block::Module { code, .. } => {
                            self.lower(code);
                        }
                        Block::AlwaysComb { code } => {
                            self.lower_code(code, None);
                        }
                        Block::AlwaysFf { code , clocks} => {
                            self.lower_code(code, Some(clocks));
                        }
                        Block::Initial { code } => {
                            for stmt in code {
                                match stmt {
                                    Stmt::Fixed { .. } => { self.lower_fixed(stmt); }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}