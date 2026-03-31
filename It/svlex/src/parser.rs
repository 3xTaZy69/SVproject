#![allow(dead_code)]

use crate::lex::*;

#[derive(Debug, Clone)]
pub enum Structure {
    Net,
    Reg,
    Const,
    Null,
    Clock(u32),
}

#[derive(Debug, Clone)]
pub enum Vtypes {
    Logic(Structure),
    Wire(Structure),
    Reg(Structure),
    Int(Structure)
}
#[derive(Debug, Clone)]
pub enum BinOps {
    Add,
    Sub,
    BitWiseAnd,
    BitWiseOr,
    LogicAnd,
    LogicOr,
    Xor,
    ShiftL,
    ShiftR,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Neq,
    MUL,
    DIV
}
#[derive(Debug, Clone)]
pub enum UnOps {
    BitWiseNot,
    LogicNot,
    Neg
}
#[derive(Debug, Clone)]
pub enum Expr {
    BinExpr { 
        left: Box<Expr>, 
        op: BinOps, 
        right: Box<Expr>, 
    },
    UnExpr { 
        operand: Box<Expr>, 
        op: UnOps, 
    },
    Imm {
        value: u64,
        width: u32,
    },
    Ident(String),
    Ref {
        base: String,
        h: u64,
        l: u64,
    },
    Parameter {
        name: String,
        value: Box<Expr>,
    },
    Int(u64),
}
#[derive(Debug, Clone)]
pub enum Edge {
    Posedge(String),
    Negedge(String),
}
#[derive(Debug, Clone)]
pub enum Sens {
    Timing(u32),
    Edge(Edge),
}

#[derive(Debug, Clone)]
pub enum Case {
    Value {
        value: Expr, 
        body: Vec<Stmt>
    },
    Default {
        body: Vec<Stmt>
    },
}
#[derive(Debug, Clone)]
pub enum Stmt {
    BlockAssign {
        target: Expr,
        value: Expr,
    },
    NonBlockAssign {
        target: Expr,
        value: Expr,
    },
    ContinuousAssign {
        target: Expr,
        value: Expr,
    },
    Decl {
        target: String,
        length: u64,
        value: Expr,
        vtype: Vtypes,
    },
    IfStmt {
        expr: Expr,
        true_code: Vec<Stmt>,
        false_code: Vec<Stmt>,
    },
    Case {
        expr: Expr,
        cases: Vec<Case>
    },
    Fixed {
        x: i32,
        y: i32,
        z: i32,
        var: String,
    },
    Tick {
        timing: u32,
        dst: String
    }

}
#[derive(Debug, Clone)]
pub enum Part {
    Stmt(Stmt),
    Block(Block),
}
#[derive(Debug, Clone)]
pub enum Inout {
    Input(String, u32),
    Output(String, u32),
}
#[derive(Debug, Clone)]
pub enum Block {
    AlwaysFf {
        clocks: Vec<Edge>,
        code: Vec<Stmt>,
    },
    AlwaysComb {
        code: Vec<Stmt>,
    },
    Module {
        ports: Vec<Inout>,
        name: String,
        code: Vec<Part>
    },
    Initial {
        code: Vec<Stmt>,
    },
}

pub struct Parser {
    pos: usize,
    tokens: Vec<Token>,
    pub ast: Vec<Part>,
    pub vars: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {pos: 0, 
                tokens: tokens, 
                ast: Vec::new(), 
                vars: Vec::new()
        }
    }
    fn advance(&mut self) {
        self.pos += 1;
    }
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }
    fn parse_i(&mut self) -> Expr {
        let a = match self.peek() {
            Some(n) => n,
            None => panic!("EXPECTED NUMBER OR IDENT")
        };
        self.advance();

        match a {
            Token::IDENT(x) => {
                if let Some(Token::LBRACK) = self.peek() {
                    let hl = self.hl();
                    Expr::Ref { base: x, h: hl.0, l: hl.1 }
                } else {
                    Expr::Ident(x)
                }
            }
            Token::INT(x) => Expr::Int(x),
            Token::BININT(x,y ) => Expr::Imm {value: y, width: x},
            Token::HEXINT(x,y ) => Expr::Imm {value: y, width: x},
            Token::DECINT(x,y ) => Expr::Imm {value: y, width: x},
            Token::LPAREN => {
                let expr = self.parse_exp();
                if let Some(Token::RPAREN) = self.peek() {self.advance()} else {
                    panic!("EXPECTED CLOSING PARENTHESIES")
                }
                expr
            },
            _ => panic!("EXPECTED NUMBER OR IDENT OR PARENS, GOT {}", a)
        }
    }
    fn parse_un(&mut self) -> Expr {
        match self.peek() {
            Some(Token::LGNOT) => {
                self.advance();
                Expr::UnExpr {
                    operand: Box::new(self.parse_un()),
                    op: UnOps::LogicNot,
                }
            },
            Some(Token::BWNOT) => {
                self.advance();
                Expr::UnExpr {
                    operand: Box::new(self.parse_un()),
                    op: UnOps::BitWiseNot,
                }
            },
            Some(Token::MINUS) => {
                self.advance();
                Expr::UnExpr {
                    operand: Box::new(self.parse_un()),
                    op: UnOps::Neg,
                }
            },
            _ => self.parse_i(),
        }
    }
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_un();
        while matches!(self.peek(), Some(Token::DIV) | Some(Token::MUL)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::MUL => BinOps::MUL,
                Token::DIV => BinOps::DIV,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_un();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_add(&mut self) -> Expr {
        let mut left = self.parse_term();
        while matches!(self.peek(), Some(Token::PLUS) | Some(Token::MINUS)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::PLUS => BinOps::Add,
                Token::MINUS => BinOps::Sub,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_term();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_shifts(&mut self) -> Expr {
        let mut left = self.parse_add();
        while matches!(self.peek(), Some(Token::SHIFTL) | Some(Token::SHIFTR)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::SHIFTL => BinOps::ShiftL,
                Token::SHIFTR => BinOps::ShiftR,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_add();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_equalities(&mut self) -> Expr {
        let mut left = self.parse_shifts();
        while matches!(self.peek(), Some(Token::BIGGER) | Some(Token::BIGGEREQ) | Some(Token::EQ) | Some(Token::NOTEQ) | Some(Token::LESS) | Some(Token::LESSEQ)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::BIGGER => BinOps::Gt,
                Token::BIGGEREQ => BinOps::Ge,
                Token::LESS => BinOps::Lt,
                Token::LESSEQ => BinOps::Le,
                Token::EQ => BinOps::Eq,
                Token::NOTEQ => BinOps::Neq,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_shifts();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_equalities();
        while matches!(self.peek(), Some(Token::BWAND)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::BWAND => BinOps::BitWiseAnd,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_equalities();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_xor(&mut self) -> Expr {
        let mut left = self.parse_and();
        while matches!(self.peek(), Some(Token::XOR)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::XOR => BinOps::Xor,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_and();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_xor();
        while matches!(self.peek(), Some(Token::BWOR)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::BWOR => BinOps::BitWiseOr,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_xor();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    fn parse_lgand(&mut self) -> Expr {
        let mut left = self.parse_or();
        while matches!(self.peek(), Some(Token::LGAND)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::LGAND => BinOps::LogicAnd,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_or();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    pub fn parse_exp(&mut self) -> Expr {
        let mut left = self.parse_lgand();
        while matches!(self.peek(), Some(Token::LGOR)) {
            let op = self.peek().unwrap();
            let op = match op {
                Token::LGOR => BinOps::LogicOr,
                _ => unreachable!()
            };
            self.advance();
            let right = self.parse_lgand();
            left = Expr::BinExpr { left: Box::new(left), op: op, right: Box::new(right) }
        }
        left
    }
    
}


impl Parser {
    pub fn require(&self, request: Token) {
        match self.peek() {
            Some(token) if token == request => {}
            Some(token) => panic!("EXPECTED {:?}, GOT {:?}", request, token),
            None => panic!("EXPECTED {:?}, GOT NONE", request),
        }
    }
    fn require_int(&self) {
        match self.peek() {
            Some(Token::INT(_)) => {},
            Some(token) => panic!("EXPECTED INT, GOT {:?}", token),
            None => panic!("EXPECTED INT, GOT NONE")
        }
    }
    fn require_ident(&self) {
        match self.peek() {
            Some(Token::IDENT(_)) => {},
            Some(token) => panic!("EXPECTED IDENT, GOT {:?}", token),
            None => panic!("EXPECTED IDENT, GOT NONE")
        }
    }
    fn hl(&mut self) -> (u64, u64) {
        self.require(Token::LBRACK);
        self.advance();
        self.require_int();
        let h = self.peek().unwrap();
        self.advance();
        self.require(Token::COLON);
        self.advance();
        self.require_int();
        let l = self.peek().unwrap();
        self.advance();
        self.require(Token::RBRACK);
        self.advance();
        (h.get_int_val().unwrap(), l.get_int_val().unwrap())
    }
    fn hl_to_w(x: (u64, u64)) -> u64 {
        if x.0 == x.1{
            panic!("LENGTH CANT BE 0")
        }
        if x.0 < x.1 {
            x.1 - x.0 + 1
        } else {
            x.0 - x.1 + 1
        }
    }
    fn require_vec(&self, v: Vec<Token>) {
        let mut i = false;
        for _j in &v {
            if let Some(_j) = self.peek() {
                i = true
            }
        }
        if i == false {
            panic!("EXPECTED ANY OF TYPES: {:?}, GOT {:?}", v, self.peek())
        }
    }
    pub fn parse_decl(&mut self) -> Stmt {
        self.require_vec(vec![Token::INTKW, Token::LOGIC, Token::REG, Token::WIRE]);
        let vtype = match self.peek().unwrap() {
            Token::INTKW => Vtypes::Int,
            Token::REG => Vtypes::Reg,
            Token::WIRE => Vtypes::Wire,
            Token::LOGIC => Vtypes::Logic,
            _ => Vtypes::Int,
        };
        self.advance();
        let mut width = 1;

        if let Some(Token::LBRACK) = self.peek() {
            width = Parser::hl_to_w(self.hl())
        }

        self.require_ident();
        let name = match self.peek() {
            Some(Token::IDENT(x)) => x,
            _ => String::new()
        };

        self.advance();
        let mut expr = Expr::Int(0);

        if let Some(Token::EQUAL) = self.peek() {
            self.advance();
            expr = self.parse_exp();
        }

        self.require(Token::SEMICOLON);
        self.advance();
        
        self.vars.push(name.clone());
        Stmt::Decl { target: name, length: width, value: expr, vtype: vtype(Structure::Null)}

    }
    pub fn parse_assign(&mut self)  -> Stmt {
        let mut continuous = false;
        if let Some(Token::ASSIGN) = self.peek() {
            self.advance();
            continuous = true;
        }

        self.require_ident();
        let name = match self.peek().unwrap() {
            Token::IDENT(x) => x,
            _ => String::new()
        };
        self.advance();

        let mut h: Option<u64> = None;
        let mut l: u64 = 0;


        if let Some(Token::LBRACK) = self.peek() {
            let hl = self.hl();
            h = Some(hl.0);
            l = hl.1;
        }

        self.require_vec(vec![Token::EQUAL, Token::LESSEQ]);
        let mut nonblocking = false;
        if let Some(Token::LESSEQ) = self.peek() {
            nonblocking = true
        }
        self.advance();

        let expr = self.parse_exp();
        self.require(Token::SEMICOLON);
        self.advance();

        if continuous & nonblocking {
            panic!("VARIABLE CANT BE ASSIGNED AS CONTINUOUS AND NONBLOCKING")
        }


        let ret = |trgt| -> Stmt {
            if continuous {
                Stmt::ContinuousAssign { target: trgt, value: expr }
            } else if nonblocking {
                Stmt::NonBlockAssign { target: trgt, value: expr }
            } else {
                Stmt::BlockAssign { target: trgt, value: expr }
            }
        };

        if let Some(x) = h {
            let h = x;
            let trgt = Expr::Ref { base: name, h: h, l: l };
            ret(trgt)
        } else {
            ret(Expr::Ident(name))
        }

        
    }
    fn parse_int(&mut self) -> i32 {
        let sign = if let Some(Token::MINUS) = self.peek() {
            self.advance();
            -1
        } else {
            1
        };

        self.require_int();

        if let Some(Token::INT(x)) = self.peek() {
            self.advance();
            sign * (x as i32)
        } else {
            0
        }
    }
    pub fn xyz(&mut self) -> (i32, i32, i32) {
        self.require(Token::LPAREN);
        self.advance();

        let x = self.parse_int();
        self.require(Token::COMMA);
        self.advance();
        let y = self.parse_int();
        self.require(Token::COMMA);
        self.advance();
        let z = self.parse_int();
        self.require(Token::RPAREN);
        self.advance();

        (x, y, z)
    }
    pub fn parse_fixed(&mut self) -> Stmt {
        self.require(Token::FIXED);
        self.advance();
        let xyz = self.xyz();
        self.require_ident();
        let name = if let Some(Token::IDENT(x)) = self.peek() {
            x
        } else {String::new()};

        self.advance();
        self.require(Token::SEMICOLON);
        self.advance();

        Stmt::Fixed { x: xyz.0, y: xyz.1, z: xyz.2, var: name }
    }
    pub fn parse_if(&mut self)  -> Stmt {
        self.require(Token::IF);
        self.advance();
        self.require(Token::LPAREN);
        self.advance();
        let expr = self.parse_exp();
        self.require(Token::RPAREN);
        self.advance();
        let mut code: Vec<Stmt> = Vec::new();
        if let Some(Token::BEGIN) = self.peek() {
            self.advance();
            code = self.parse_code(Token::END);
            self.advance();
        } else {
            code.push(self.parse_code_once());
        }
        if let Some(Token::ELSE) = self.peek() {
            
            let mut fcode: Vec<Stmt> = Vec::new();
            self.advance();
                if let Some(Token::BEGIN) = self.peek() {
                self.advance();
                fcode = self.parse_code(Token::END);
                self.advance();
            } else {
                fcode.push(self.parse_code_once());
            }
            Stmt::IfStmt { expr, true_code: code, false_code: fcode }
        } else {
            Stmt::IfStmt { expr, true_code: code, false_code: Vec::new() }
        }

    }
    pub fn case(&mut self) -> Case {
        let mut default: bool = false;
        let mut expr = Expr::Int(0);
        if let Some(Token::DEFAULT) = self.peek() {
            default = true;
            self.advance();
        } else {

            expr = self.parse_exp();
        }
        self.require(Token::COLON);
        self.advance();
        if let Some(Token::BEGIN) = self.peek() {
            self.advance();
            let code = self.parse_code(Token::END);
            self.advance();
            if default {
                Case::Default { body: code }
            } else {
                Case::Value { value: expr, body: code }
            }
        } else {
            let mut code: Vec<Stmt> = Vec::new();
            code.push(self.parse_code_once());
            if default {
                Case::Default { body: code }
            } else {
                Case::Value { value: expr, body: code }
            }
        }
    }
    pub fn parse_case(&mut self) -> Stmt {
        self.require(Token::CASE);
        self.advance();
        self.require(Token::LPAREN);
        self.advance();
        let expr = self.parse_exp();
        self.require(Token::RPAREN);
        self.advance();
        let mut cases: Vec<Case> = Vec::new();
        loop {
            if let Some(Token::ENDCASE) = self.peek() {
                break
            }
            cases.push(self.case());
        }
        self.advance();
        Stmt::Case { expr: expr, cases: cases }

    }
    pub fn parse_pn(&mut self) -> Vec<Edge> {
        let mut edges: Vec<Edge> = Vec::new();
        let choose = |x: Token, clock: String| -> Edge {
            match x {
                Token::POSEDGE => Edge::Posedge(clock),
                Token::NEGEDGE => Edge::Negedge(clock),
                _ => panic!("UNEXPECTED TOKEN TYPE {}, MUST BE POSEDGE OR NEGEDGE", x)
            }
        };
        let mut edge = self.peek().unwrap_or_else(|| panic!("EDGE EXPECTED"));
        self.advance();
        let mut clock: String = if let Some(Token::IDENT(x)) = self.peek() {x} else {panic!("CLOCK EXPECTED AFTER EDGE, GOT {:?}", self.peek())};
        edges.push(choose(edge, clock));
        self.advance();
        while let Some(Token::EDGOR) = self.peek() {
            self.advance();
            edge = self.peek().unwrap_or_else(|| panic!("EDGE EXPECTED"));
            self.advance();
            clock = if let Some(Token::IDENT(x)) = self.peek() {x} else {panic!("CLOCK EXPECTED AFTER EDGE, GOT {:?}", self.peek())};
            edges.push(choose(edge, clock));
            self.advance();
        }
        self.require(Token::RPAREN);
        self.advance();

        edges
    }
    pub fn always_ff(&mut self) -> Block {
        self.require(Token::ALWAYSFF);
        self.advance();
        self.require(Token::AT);
        self.advance();
        self.require(Token::LPAREN);
        self.advance();
        let edges = self.parse_pn();
        self.require(Token::BEGIN);
        self.advance();
        let code = self.parse_code(Token::END);
        self.advance();
        Block::AlwaysFf { clocks: edges, code: code }
    }
    pub fn parse_inouts(&mut self) -> Vec<Inout> {
        let mut inouts: Vec<Inout> = Vec::new();
        
        while matches!(self.peek(), Some(Token::OUTPUT) | Some(Token::INPUT)) {
            inouts.extend(self.parse_inout());
        }
        self.require(Token::RPAREN);
        self.advance();
        inouts
    }
    pub fn parse_inout(&mut self) -> Vec<Inout> {
        let itype = self.peek().unwrap_or_else(|| panic!("EXPECTED INPUT OR OUTPUT TYPE, GOT NONE"));
        self.advance();
        let mut width = 1;
        if let Some(Token::LBRACK) = self.peek() {
            let hl = self.hl();
            width = hl.0 - hl.1 + 1;
        }
        let mut inouts: Vec<Inout> = Vec::new();

        if let Some(Token::IDENT(x)) = self.peek() {
            self.vars.push(x.clone());
            if let Token::INPUT = itype {
                inouts.push(Inout::Input(x, width as u32));
            } else if  let Token::OUTPUT = itype {
                inouts.push(Inout::Output(x, width as u32));
            } else {
                panic!("EXPECTED INPUT OR OUTPUT, GOT {}", itype)
            }
        }
        self.advance();
        while let Some(Token::COMMA) = self.peek() {
            self.advance();
                if let Some(Token::IDENT(x)) = self.peek() {
                if let Token::INPUT = itype {
                    println!("parsed {}", x.clone());
                    inouts.push(Inout::Input(x, width as u32));
                    self.advance();
                } else {
                    println!("parsed {}", x.clone());
                    inouts.push(Inout::Output(x, width as u32));
                    self.advance();
                } 
            }
        }
        inouts

    }
    pub fn parse_module(&mut self) -> Block {
        self.require(Token::MODULE);
        self.advance();
        let name;
        if let Some(Token::IDENT(x)) = self.peek() {
            name = x;
            self.advance();
        } else {
            panic!("MODULE NAME EXPECTED");
        }
        let mut ports: Vec<Inout> = Vec::new();
        if let Some(Token::LPAREN) = self.peek() {
            self.advance();
            ports = self.parse_inouts();
        }
        self.require(Token::SEMICOLON);
        self.advance();
        let code = self.parse_code_blockinc(Token::ENDMODULE);
        self.require(Token::ENDMODULE);
        self.advance();

        Block::Module { ports: ports, name: name, code: code }
    }
    pub fn parse_code(&mut self, endpoint: Token) -> Vec<Stmt> {

        let mut parts: Vec<Stmt> = Vec::new();
        loop {
            let current = match self.peek() {
                Some(x) => x,
                None => break,
            };
            if current == endpoint {break}
            if matches!(current, Token::INTKW | Token::LOGIC | Token::WIRE | Token::REG) {
                parts.push(self.parse_decl())
            } else if let Token::IF = current {
                parts.push(self.parse_if())
            } else if let Token::CASE = current {
                parts.push(self.parse_case())
            } else if let Token::IDENT(x) = current {
                if self.vars.contains(&x) {
                    parts.push(self.parse_assign())
                } else {
                    panic!("IDENT {x} SHOULDNT BE HERE")
                }
            } else if let Token::ASSIGN = current {
                parts.push(self.parse_assign());
            } else {
                panic!("UNEXPECTED TOKEN TYPE, {}", current)
            }
        }
        parts
    }
    pub fn parse_code_blockinc(&mut self, endpoint: Token) -> Vec<Part> {
        let mut parts: Vec<Part> = Vec::new();
        loop {
            let current = match self.peek() {
                Some(x) => x,
                None => break,
            };
            if current == endpoint {break}
            if matches!(current, Token::INTKW | Token::LOGIC | Token::WIRE | Token::REG) {
                parts.push(Part::Stmt(self.parse_decl()))
            } else if let Token::IF = current {
                parts.push(Part::Stmt(self.parse_if()))
            } else if let Token::CASE = current {
                parts.push(Part::Stmt(self.parse_case()))
            } else if let Token::MODULE = current {
                parts.push(Part::Block(self.parse_module()))
            } else if let Token::ALWAYSFF = current {
                parts.push(Part::Block(self.always_ff()))
            } else if let Token::ALWAYSCOMB = current {
                parts.push(Part::Block(self.parse_comb()))
            } else if let Token::INITIAL = current {
                parts.push(Part::Block(self.parse_initial()))
            } else if let Token::IDENT(x) = current {
                if self.vars.contains(&x) {
                    parts.push(Part::Stmt(self.parse_assign()))
                } else {
                    panic!("IDENT {x} SHOULDNT BE HERE")
                }
            } else if let Token::ASSIGN = current {
                parts.push(Part::Stmt(self.parse_assign()));
            } else {
                panic!("UNEXPECTED TOKEN TYPE, {}", current)
            }
        }
        parts
    }
    pub fn parse_code_once(&mut self) -> Stmt {
        let current = match self.peek() {
            Some(x) => x,
            None => panic!("UNEXPECTED CODE ENDING")
        };
            if matches!(current, Token::INTKW | Token::LOGIC | Token::WIRE | Token::REG) {
                self.parse_decl()
            } else if let Token::IF = current {
                self.parse_if()
            } else if let Token::CASE = current {
                self.parse_case()
            } else if let Token::IDENT(x) = current {
                if self.vars.contains(&x) {
                    self.parse_assign()
                } else {
                    panic!("IDENT {x} SHOULDNT BE HERE")
                }
            } else if let Token::ASSIGN = current {
                self.parse_assign()
            } else {
                panic!("UNEXPECTED TOKEN TYPE, {}", current)
            }
            
    }
    pub fn parse_initial(&mut self) -> Block {
        self.require(Token::INITIAL);
        self.advance();
        self.require(Token::BEGIN);
        self.advance();
        let mut code: Vec<Stmt> = Vec::new();
        while !matches!(self.peek(), Some(Token::END)) {
            if let Some(Token::FIXED) = self.peek() {
                code.push(self.parse_fixed());
            } else {
                code.push(self.parse_tick());
            }
        }
        self.advance();
        Block::Initial { code: code }
    }
    pub fn parse_tick(&mut self) -> Stmt {
        self.require(Token::TICK);
        self.advance();
        self.require(Token::LPAREN);
        self.advance();
        self.require_int();
        let timing = if let Some(Token::INT(x)) = self.peek() {x} else {0};
        self.advance();
        self.require(Token::RPAREN);
        self.advance();
        self.require_ident();
        let name = if let Some(Token::IDENT(x)) = self.peek() {x} else {String::new()};
        self.advance();
        self.require(Token::SEMICOLON);
        self.advance();
        Stmt::Tick { timing: timing as u32, dst: name }
    }
    pub fn parse_comb(&mut self) -> Block {
        self.require(Token::ALWAYSCOMB);
        self.advance();
        self.require(Token::BEGIN);
        self.advance();
        let code = self.parse_code(Token::END);
        self.advance();
        Block::AlwaysComb { code: code }
    }
    pub fn parse(&mut self) {
        let mut parts: Vec<Part> = Vec::new();

        loop {
            let current = match self.peek() {
                Some(x) => x,
                None => break,
            };
            if matches!(current, Token::INTKW | Token::LOGIC | Token::WIRE | Token::REG) {
                parts.push(Part::Stmt(self.parse_decl()))
            } else if let Token::IF = current {
                parts.push(Part::Stmt(self.parse_if()))
            } else if let Token::CASE = current {
                parts.push(Part::Stmt(self.parse_case()))
            } else if let Token::MODULE = current {
                parts.push(Part::Block(self.parse_module()))
            } else if let Token::ALWAYSFF = current {
                parts.push(Part::Block(self.always_ff()))
            } else if let Token::ALWAYSCOMB = current {
                parts.push(Part::Block(self.parse_comb()))
            } else if let Token::INITIAL = current {
                parts.push(Part::Block(self.parse_initial()))
            } else if let Token::IDENT(x) = current {
                if self.vars.contains(&x) {
                    parts.push(Part::Stmt(self.parse_assign()))
                } else {
                    panic!("IDENT {x} SHOULDNT BE HERE")
                }
            } else if let Token::ASSIGN = current {
                parts.push(Part::Stmt(self.parse_assign()));
            } else {
                panic!("UNEXPECTED TOKEN TYPE, {}", current)
            }
        }
        



        self.ast.extend(parts);
    }

    


}








use core::panic;
use std::{fmt, future::pending};



pub fn show_expr(expr: &Expr) -> String {
    match expr {
        Expr::BinExpr { left, op, right } => {
            format!(
                "({} {:?} {})",
                show_expr(left),
                op,
                show_expr(right)
            )
        }
        Expr::Int(x) => x.to_string(),
        Expr::Ident(x) => x.clone(),
        Expr::UnExpr { operand, op } => {
            format!(
                "({:?}{})",
                op,
                show_expr(operand)
            )
        }
        Expr::Ref { base, h, l } => {
            format!(
                "{}[{}:{}]",
                base,
                h,
                l
            )
        }
        Expr::Imm { value, width } => {
            format!(
                "{}'{}",
                width,
                value
            )
        }
        _ => "expr".to_string(),
    }
}