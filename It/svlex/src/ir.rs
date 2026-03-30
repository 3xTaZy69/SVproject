/*

Static Assignment IR ( Not single assignment )

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
    target: String,
    value: A
}

#[derive(Debug)]
pub enum A {
    Reg { len: u32, value: u64, signed: bool },
    Net { len: u32, signed: bool },
    Clock { timing: u32 },
    Const { len: u32, value: u64, signed: bool },
    UnOp { operation: UnOp, operand: String },
    BinOp { left: String, right: String, op: BinOp },
    Dff { clock: String, input: String, len: u32 }, /*
        not real "d flip flop", just specification for register input
     */
    Mux { sel: String, trueval: String, falseval: String },
    Block { from: String },
    Assign { from: String },
    Fixed { x: i32, y: i32, z: i32 },
    Ref { base: String, h: u32, l: u32},
    NonBlock { from: String, clocks: Vec<Edge> }  
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
                SA {target: name, value: A::NonBlock { from: value, clocks: ffclocks.unwrap() }}
            }
            _ => panic!("EXPECTED ASSIGN, GOT {assign:?}")
        };

        String::new()
    }
}