#[allow(dead_code)]
#[derive(Clone)]
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    MODULE,
    IDENT(String),
    LPAREN,
    INPUT,
    OUTPUT,
    COMMA,
    RPAREN,
    SEMICOLON,
    ENDMODULE,
    ALWAYS,
    SHARP,
    INT(u64),
    WIRE,
    REG,
    EQUAL,
    PLUS,
    MINUS,
    XOR,
    ALWAYSFF,
    AT,
    POSEDGE,
    NEGEDGE,
    BEGIN,
    END,
    EDGOR,
    LESS,
    BIGGER,
    EQ,
    LESSEQ,
    BIGGEREQ,
    NOTEQ,
    IF,
    ELSE,
    CASE,
    ENDCASE,
    LBRACK,
    RBRACK,
    COLON,
    FOR,
    INTEGER,
    DOT,
    ASSIGN,
    ALWAYSCOMB,
    PARAMETER,
    LOCALPARAM,
    BININT(u32, u64),
    HEXINT(u32, u64),
    DECINT(u32, u64),
    INTKW,
    DIV,
    MUL,
    LOGIC,
    BIT,
    LONGINT,
    SHORTINT,
    BYTE,
    SHIFTL,
    SHIFTR,
    BWAND,
    BWNOT,
    BWOR,
    LGAND,
    LGNOT,
    LGOR,
    INITIAL,
    FIXED,
    DEFAULT,
    TICK
}

impl Token {
    pub fn get_int_val(&self) -> Option<u64> {
        if let Token::INT(x) = self {
            Some(*x)
        } else {
            None
        }
    }
}


pub struct Lexer {
    pos: usize,
    text: String,
    line: u32,
    pub tokens: Vec<Token>
} impl Lexer {
    pub fn new(text: String) -> Lexer {
        Lexer { pos: 0, text: text, line: 0, tokens: Vec::new() }
    }
    fn peek(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }
    fn advance(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }
    fn number(&mut self) {
        let mut n: String = String::new();

        while self.peek().unwrap().is_ascii_digit() {
            n.push(self.peek().unwrap());
            self.advance();
        }

        if self.peek().unwrap() == '\'' {
            self.advance();
            let base = self.peek().unwrap();
            self.advance();


            let mut y: String = String::new();
            
            if "bd".contains(base) {
                while self.peek().unwrap().is_ascii_digit() {
                y.push(self.peek().unwrap());
                self.advance();
            }} else {
                while self.peek().unwrap().is_ascii_alphanumeric() {
                y.push(self.peek().unwrap());
                self.advance();
            }}
            

            let s = match base {
                'b' => u64::from_str_radix(&y, 2).expect("couldnt turn binary into number"),
                'h' => u64::from_str_radix(&y, 16).expect("coulnt turn hex into number"),
                'd' => y.parse().expect("couldnt parse decimal into number on line"),
                _ => panic!("unknown base")
            };

            let n: u32 = n.parse().expect("couldnt turn size into int");

            match base {
                'b' => self.tokens.push(Token::BININT(n, s)),
                'h' => self.tokens.push(Token::HEXINT(n, s)),
                'd' => self.tokens.push(Token::DECINT(n, s)),
                _ => panic!("unknown base")
            }
        
        } else {
        
        self.tokens.push(Token::INT(n.parse().expect("Couldn`t parse number into int")))

        }
    }
    fn word(&mut self) {
        let mut w: String = String::new();

        while self.peek().unwrap().is_alphanumeric() || self.peek().unwrap() == '_'  {
            w.push(self.peek().unwrap());
            self.advance();
        }

        self.tokens.push(
            match w.as_str() {
                "module" => Token::MODULE,
                "endmodule" => Token::ENDMODULE,
                "input" => Token::INPUT,
                "output" => Token::OUTPUT,
                "always" => Token::ALWAYS,
                "wire" => Token::WIRE,
                "reg" => Token::REG,
                "always_ff" => Token::ALWAYSFF,
                "posedge" => Token::POSEDGE,
                "negedge" => Token::NEGEDGE,
                "begin" => Token::BEGIN,
                "end" => Token::END,
                "or" => Token::EDGOR,
                "if" => Token::IF,
                "else" => Token::ELSE,
                "case" => Token::CASE,
                "endcase" => Token::ENDCASE,
                "for" => Token::FOR,
                "integer" => Token::INTEGER,
                "assign" => Token::ASSIGN,
                "always_comb" => Token::ALWAYSCOMB,
                "parameter" => Token::PARAMETER,
                "localparam" => Token::LOCALPARAM,
                "int" => Token::INTKW,
                "logic" => Token::LOGIC,
                "bit" => Token::BIT,
                "while" => panic!("WHILE LOOP ON LINE {} NOT SUPPORTED", self.line),
                "longint" => Token::LONGINT,
                "shortint" => Token::SHORTINT,
                "byte" => Token::BYTE,
                "initial" => Token::INITIAL,
                "fixed" => Token::FIXED,
                "default" => Token::DEFAULT,
                "tick" => Token::TICK,
                _ => Token::IDENT(w),
            }
        );
    }
    fn pushadv(&mut self, t: Token) {
        self.advance();
        self.tokens.push(t)
    }

    fn comparsion(&mut self) {
        let mut s = String::new();
        s.push(self.peek().unwrap());

        self.advance();

        if "><=".contains(self.peek().unwrap()) {
            s.push(self.peek().unwrap());
            self.advance();
        }

        self.tokens.push(
            match s.as_str() {
                ">>" => Token::SHIFTR,
                "<<" => Token::SHIFTL,
                "<" => Token::LESS,
                ">" => Token::BIGGER,
                "<=" => Token::LESSEQ,
                ">=" => Token::BIGGEREQ,
                "==" => Token::EQ,
                "!=" => Token::NOTEQ,
                "=" => Token::EQUAL,
                _ => panic!("UNKNOWN COMPARSION OPERATOR '{}' ON LINE {}", s, self.line)
            }
        )
        
    }

    fn opers(&mut self) {
        let mut f: String = String::new();
        f.push(self.peek().unwrap());
        self.advance();
        let c = self.peek().unwrap();
        if "&|".contains(c) {
            f.push(c);
            self.advance();
        }

        let t = match f.as_str() {
            "&" => Token::BWAND,
            "~" => Token::BWNOT,
            "|" => Token::BWOR,
            "&&" => Token::LGAND,
            "||" => Token::LGOR,
            "!" => Token::LGNOT,
            "^" => Token::XOR,
            _ => panic!("UNKNOWN LOGIC OPERATOR '{f}'")
        };

        self.tokens.push(t);
    }

    fn ops(&mut self) {
        let mut s = String::new();
        s.push(self.peek().unwrap());

        self.advance();

        self.tokens.push(
            match s.as_str() {
                "+" => Token::PLUS,
                "-" => Token::MINUS,
                _ => Token::PLUS,
            }
        )
        
    }

    fn parsediv(&mut self) {
        self.advance();
        if let Some('*') = self.peek() {
            loop {
                let s: char = self.peek().unwrap_or_else(|| panic!("EXPECTED CLOSED COMMENT"));
                self.advance();
                let s2:char = self.peek().unwrap_or_else(|| panic!("EXPECTED CLOSED COMMENT"));
                let mut s3: String = String::new();
                s3.push(s);
                s3.push(s2);
                if s3.as_str() == "*/" {
                    self.advance();
                    self.advance();
                    break
                }
        }} else {
            self.tokens.push(Token::DIV);
        }
    }

    pub fn lex(&mut self) {
        while let Some(_) = self.peek() {
            let current = self.peek().unwrap();
            if current.is_ascii_digit() {
                self.number();
            } else if current.is_alphanumeric() || current == '_' {
                self.word()
            } else if current == '(' {
                self.pushadv(Token::LPAREN);
            } else if current == ')' {
                self.pushadv(Token::RPAREN);
            } else if current == '#' {
                self.pushadv(Token::SHARP);
            } else if current.is_whitespace() {
                self.advance();
            } else if current == ';' {
                self.pushadv(Token::SEMICOLON);
            } else if current == ',' {
                self.pushadv(Token::COMMA);
            } else if "+-".contains(current) {
                self.ops();
            } else if "&|!~^".contains(current) {
                self.opers();
            } else if current == '^' {
                self.pushadv(Token::XOR);
            } else if current == '@' {
                self.pushadv(Token::AT);
            } else if "<>=!".contains(current) {
                self.comparsion();
            } else if current == '[' {
                self.pushadv(Token::LBRACK);
            } else if current == ']' {
                self.pushadv(Token::RBRACK);
            } else if current == ':' {
                self.pushadv(Token::COLON);
            } else if current == '.' {
                self.pushadv(Token::DOT);
            } else if current == '/' {
                self.parsediv();
            } else if current == '*' {
                self.pushadv(Token::MUL);
            } else if current == '\n' {
                self.advance();
                self.line += 1;
            } else {
                panic!("ERR: {} on {}", current, self.line)
            } 
        }
    }
}

use std::fmt;

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Token::ALWAYS => "ALWAYS".to_string(),
            Token::MODULE => "MODULE".to_string(),
            Token::ENDMODULE => "ENDMODULE".to_string(),
            Token::COMMA => "COMMA".to_string(),
            Token::EQUAL => "EQUAL".to_string(),
            Token::INPUT => "INPUT".to_string(),
            Token::OUTPUT => "OUTPUT".to_string(),
            Token::LPAREN => "LPAREN".to_string(),
            Token::RPAREN => "RPAREN".to_string(),
            Token::SHARP => "SHARP".to_string(),
            Token::REG => "REG".to_string(),
            Token::WIRE => "WIRE".to_string(),

            Token::IDENT(x) => format!("IDENT({})", x),
            Token::INT(x) => format!("INT({})", x),

            Token::PLUS => "PLUS".to_string(),
            Token::MINUS => "MINUS".to_string(),
            Token::XOR => "XOR".to_string(),
            Token::SEMICOLON => "SEMICOLON".to_string(),
            Token::ALWAYSFF => "ALWAYSFF".to_string(),
            Token::AT => "AT".to_string(),
            Token::POSEDGE => "POSEDGE".to_string(),
            Token::NEGEDGE => "NEGEDGE".to_string(),
            Token::BEGIN => "BEGIN".to_string(),
            Token::END => "END".to_string(),
            Token::EDGOR => "EDGOR".to_string(),
            Token::LESS => "LESS".to_string(),
            Token::LESSEQ => "LESSEQ".to_string(),
            Token::BIGGER => "BIGGER".to_string(),
            Token::BIGGEREQ => "BIGGEREQ".to_string(),
            Token::NOTEQ => "NOTEQ".to_string(),
            Token::EQ => "EQ".to_string(),
            Token::IF => "IF".to_string(),
            Token::ELSE => "ELSE".to_string(),
            Token::CASE => "CASE".to_string(),
            Token::ENDCASE => "ENDCASE".to_string(),
            Token::LBRACK => "LBRACK".to_string(),
            Token::RBRACK => "RBRACK".to_string(),
            Token::INTEGER => "INTEGER".to_string(),
            Token::FOR => "FOR".to_string(),
            Token::COLON => "COLON".to_string(),
            Token::DOT => "DOT".to_string(),
            Token::ASSIGN => "ASSIGN".to_string(),
            Token::ALWAYSCOMB => "ALWAYSCOMB".to_string(),
            Token::PARAMETER => "PARAMETER".to_string(),
            Token::LOCALPARAM => "LOCALPARAM".to_string(),

            Token::BININT(s, i) => format!("BININT({}, {})", s, i),
            Token::DECINT(s, i) => format!("DECINT({}, {})", s, i),
            Token::HEXINT(s, i) => format!("HEXINT({}, {})", s, i),

            Token::INTKW => "INTKW".to_string(),
            Token::MUL => "MUL".to_string(),
            Token::DIV => "DIV".to_string(),
            Token::LOGIC => "LOGIC".to_string(),
            Token::BIT => "BIT".to_string(),
            Token::SHORTINT => "SHORTINT".to_string(),
            Token::LONGINT => "LONGINT".to_string(),
            Token::BYTE => "BYTE".to_string(),
            Token::SHIFTL => "SHIFTL".to_string(),
            Token::SHIFTR => "SHIFTR".to_string(),
            Token::BWAND => "BWAND".to_string(),
            Token::BWOR => "BWOR".to_string(),
            Token::BWNOT => "BWNOT".to_string(),
            Token::LGAND => "LGAND".to_string(),
            Token::LGOR => "LGOR".to_string(),
            Token::LGNOT => "LGNOT".to_string(),
            Token::INITIAL => "INITIAL".to_string(),
            Token::FIXED => "FIXED".to_string(),
            Token::DEFAULT => "DEFAULT".to_string(),
            Token::TICK => "TICK".to_string(),
        };

        write!(f, "{}", s)
    }
}