use crate::{Compiler, Error, Source, StringIdx};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    EndOfFile,
    Invalid,
    Whitespace, Comment,
    Integer, Float,
    String, CString,
    Identifier,
    BraceOpen, BraceClose,
    BracketOpen, BracketClose,
    ParenOpen, ParenClose,
    Equal,
    Plus, Minus, Asterisk, Slash, Percent,
    PlusEqual, MinusEqual, AsteriskEqual, SlashEqual, PercentEqual,
    LessThan, GreaterThan, LessThanEqual, GreaterThanEqual,
    DoubleEqual, NotEqual,
    ExclamationMark,
    DoubleAmpersand,
    DoublePipe,
    Colon, DoubleColon,
    Comma, Semicolon,
    Dot,
    Ampersand,
    KeywordPub, KeywordExt, KeywordMod, KeywordUse,
    KeywordStruct, KeywordFun, KeywordVar, KeywordEnum, KeywordInterface,
    KeywordIf, KeywordElse, KeywordLoop, 
    KeywordReturn, KeywordContinue, KeywordBreak,
    KeywordAs, KeywordSizeof, 
    KeywordConst,
    KeywordTrue, KeywordFalse, KeywordUnit,
    KeywordU8, KeywordU16, KeywordU32, KeywordU64,
    KeywordS8, KeywordS16, KeywordS32, KeywordS64,
    KeywordF32, KeywordF64, KeywordUsize
}

#[derive(Debug, Copy, Clone)]
pub struct Token {
    pub t: TokenType,
    pub content: StringIdx,
    pub source: Source
}

#[derive(Debug)]
pub struct Lexer {
    path: StringIdx,
    source: String,
    pos: usize,
    start: usize,
    buffer: String,
}

impl Lexer {
    pub fn new(path: StringIdx, content: String) -> Lexer {
        return Lexer {
            path, source: content, 
            pos: 0,
            start: 0,
            buffer: String::new()
        };  
    }

    fn has(&self) -> bool {
        return self.pos < self.source.len();
    }

    fn char_at(&self, idx: usize) -> char {
        if let Some(c) = self.source[idx..].chars().next() {
            return c;
        }
        return '\0';
    }

    fn current(&self) -> char {
        return self.char_at(self.pos);
    } 

    fn peek(&self) -> char {
        let cw: usize = self.current_width();
        return self.char_at(self.pos + cw);
    }

    fn current_width(&self) -> usize {
        let cp: u32 = self.current() as u32;
        if cp <= 0x007F { return 1; }
        if cp <= 0x07FF { return 2; }
        if cp <= 0xFFFF { return 3; }
        return 4;
    }

    fn consume(&mut self) {
        self.buffer.push(self.current());
        self.skip();
    }

    fn skip(&mut self) {
        let cw: usize = self.current_width();
        self.pos += cw;
    }

    fn build(&mut self, c: &mut Compiler, tt: TokenType) -> Token {
        let token: Token = Token {
            t: tt,
            content: c.strings.insert(&self.buffer),
            source: Source::new(self.path, self.start, self.pos)
        };
        self.buffer.clear();
        return token;
    }
    
    fn build_s(
        &mut self, c: &mut Compiler, content: &str, tt: TokenType
    ) -> Token {
        self.pos += content.len();
        self.buffer.clear();
        return Token {
            t: tt,
            content: c.strings.insert(content),
            source: Source::new(self.path, self.start, self.pos)
        };
    }

    fn next_raw(&mut self, c: &mut Compiler) -> Token {
        self.start = self.pos;
        if !self.has() {
            return self.build(c, TokenType::EndOfFile);
        }
        if self.current().is_whitespace() {
            while self.has() && self.current().is_whitespace() {
                self.consume();
            }
            return self.build(c, TokenType::Whitespace);
        }
        if self.current() == 'c' && self.peek() == '"' {
            self.skip();
            return self.lex_string(c, TokenType::CString);
        }
        if self.current() == '"' {
            return self.lex_string(c, TokenType::String);
        }
        if self.current().is_ascii_digit() {
            while self.has() && self.current().is_ascii_digit() {
                self.consume();
            }
            if self.has() && self.current() == '.' {
                self.consume();
                while self.has() && self.current().is_ascii_digit() {
                    self.consume();
                }
                return self.build(c, TokenType::Float);
            } else {
                return self.build(c, TokenType::Integer);
            }
        }
        if self.current().is_ascii_alphanumeric() || self.current() == '_' {
            while self.has() 
                && self.current().is_ascii_alphanumeric() 
                || self.current() == '_' 
            {
                self.consume();
            }
            match self.buffer.as_str() {
                "pub" => return self.build(c, TokenType::KeywordPub),
                "ext" => return self.build(c, TokenType::KeywordExt),
                "mod" => return self.build(c, TokenType::KeywordMod),
                "use" => return self.build(c, TokenType::KeywordUse),
                "struct" => return self.build(c, TokenType::KeywordStruct),
                "fun" => return self.build(c, TokenType::KeywordFun),
                "var" => return self.build(c, TokenType::KeywordVar),
                "enum" => return self.build(c, TokenType::KeywordEnum),
                "interface" => return self.build(c, TokenType::KeywordInterface),
                "if" => return self.build(c, TokenType::KeywordIf),
                "else" => return self.build(c, TokenType::KeywordElse),
                "loop" => return self.build(c, TokenType::KeywordLoop),
                "return" => return self.build(c, TokenType::KeywordReturn),
                "continue" => return self.build(c, TokenType::KeywordContinue),
                "break" => return self.build(c, TokenType::KeywordBreak),
                "as" => return self.build(c, TokenType::KeywordAs),
                "sizeof" => return self.build(c, TokenType::KeywordSizeof),
                "const" => return self.build(c, TokenType::KeywordConst),
                "true" => return self.build(c, TokenType::KeywordTrue),
                "false" => return self.build(c, TokenType::KeywordFalse),
                "unit" => return self.build(c, TokenType::KeywordUnit),
                "u8" => return self.build(c, TokenType::KeywordU8),
                "u16" => return self.build(c, TokenType::KeywordU16),
                "u32" => return self.build(c, TokenType::KeywordU32),
                "u64" => return self.build(c, TokenType::KeywordU64),
                "s8" => return self.build(c, TokenType::KeywordS8),
                "s16" => return self.build(c, TokenType::KeywordS16),
                "s32" => return self.build(c, TokenType::KeywordS32),
                "s64" => return self.build(c, TokenType::KeywordS64),
                "f32" => return self.build(c, TokenType::KeywordF32),
                "f64" => return self.build(c, TokenType::KeywordF64),
                "usize" => return self.build(c, TokenType::KeywordUsize),
                _ => return self.build(c, TokenType::Identifier)
            }
        }
        if self.current() == '#' {
            while self.has() && self.current() != '\n' {
                self.consume();
            }
            return self.build(c, TokenType::Comment);
        }
        match (self.current(), self.peek()) {
            ('+', '=') => return self.build_s(c, "+=", TokenType::PlusEqual),
            ('-', '=') => return self.build_s(c, "-=", TokenType::MinusEqual),
            ('*', '=') => return self.build_s(c, "*=", TokenType::AsteriskEqual),
            ('/', '=') => return self.build_s(c, "/=", TokenType::SlashEqual),
            ('%', '=') => return self.build_s(c, "%=", TokenType::PercentEqual),
            ('<', '=') => return self.build_s(c, "<=", TokenType::LessThanEqual),
            ('>', '=') => return self.build_s(c, ">=", TokenType::GreaterThanEqual),
            ('=', '=') => return self.build_s(c, "==", TokenType::DoubleEqual),
            ('!', '=') => return self.build_s(c, "!=", TokenType::NotEqual),
            ('&', '&') => return self.build_s(c, "&&", TokenType::DoubleAmpersand),
            ('|', '|') => return self.build_s(c, "||", TokenType::DoublePipe),
            (':', ':') => return self.build_s(c, "::", TokenType::DoubleColon),
            _ => {}
        }
        match self.current() {
            '{' => return self.build_s(c, "{", TokenType::BraceOpen),
            '}' => return self.build_s(c, "}", TokenType::BraceClose),
            '[' => return self.build_s(c, "[", TokenType::BracketOpen),
            ']' => return self.build_s(c, "]", TokenType::BracketClose),
            '(' => return self.build_s(c, "(", TokenType::ParenOpen),
            ')' => return self.build_s(c, ")", TokenType::ParenClose),
            '=' => return self.build_s(c, "=", TokenType::Equal),
            '+' => return self.build_s(c, "+", TokenType::Plus),
            '-' => return self.build_s(c, "-", TokenType::Minus),
            '*' => return self.build_s(c, "*", TokenType::Asterisk),
            '/' => return self.build_s(c, "/", TokenType::Slash),
            '%' => return self.build_s(c, "%", TokenType::Percent),
            '<' => return self.build_s(c, "<", TokenType::LessThan),
            '>' => return self.build_s(c, ">", TokenType::GreaterThan),
            '!' => return self.build_s(c, "!", TokenType::ExclamationMark),
            ':' => return self.build_s(c, ":", TokenType::Colon),
            ',' => return self.build_s(c, ",", TokenType::Comma),
            ';' => return self.build_s(c, ";", TokenType::Semicolon),
            '.' => return self.build_s(c, ".", TokenType::Dot),
            '&' => return self.build_s(c, "&", TokenType::Ampersand),
            _ => {}
        }
        self.consume();
        c.errors.push(Error::dynamic(
            String::from("invalid character"),
            Source::new(self.path, self.start, self.pos),
            format!("'{}' is not a valid character", self.buffer)
        ));
        return self.build(c, TokenType::Invalid);
    }

    fn lex_string(&mut self, c: &mut Compiler, tt: TokenType) -> Token {
        self.skip();
        let mut escaped: bool = false;
        while self.has() && (self.current() != '"' || escaped) {
            if escaped {
                let seq_start: usize = self.pos - 1;
                match self.current() {
                    '\n' => {
                        self.skip();
                    }
                    '0' => {
                        self.skip();
                        self.buffer.push('\0');
                    }
                    'n' => {
                        self.skip();
                        self.buffer.push('\n');
                    }
                    'r' => {
                        self.skip();
                        self.buffer.push('\r');
                    }
                    't' => {
                        self.skip();
                        self.buffer.push('\t');
                    }
                    'u' if self.peek() == '{' => {
                        self.skip();
                        self.skip();
                        let cp_start: usize = self.pos;
                        while self.has() && self.current() != '}' {
                            self.skip();
                        }
                        if self.current() != '}' {
                            c.errors.push(Error::fixed(
                                "unclosed unicode codepoint sequence",
                                Source::new(self.path, seq_start, self.pos),
                                "this literal is opened here, but never closed"
                            ));
                        }
                        self.skip();
                        let mut valid: bool = true;
                        if let Ok(cp) = u32::from_str_radix(
                            &self.source[cp_start..self.pos - 1], 
                            16
                        ) {
                            if let Some(c) = char::from_u32(cp) {
                                self.buffer.push(c);
                            } else { valid = false; }
                        } else { valid = false; }
                        if !valid {
                            c.errors.push(Error::fixed(
                                "invalid unicode codepoint sequence",
                                Source::new(
                                    self.path, cp_start, self.pos - 1
                                ),
                                concat!(
                                    "these characters must only be hex digits",
                                    " that represent a valid unicode codepoint"
                                )
                            ));
                            self.buffer.push('�');
                        }
                    }
                    _ => self.consume()
                }
                escaped = false;
            } else {
                escaped = self.current() == '\\';
                self.consume();
            }
        }
        if self.current() != '"' {
            c.errors.push(Error::fixed(
                "unclosed string literal",
                Source::new(self.path, self.start, self.start + 1),
                "this literal is opened here, but never closed"
            ));
        }
        self.skip();
        return self.build(c, tt);
    }

    pub fn next(&mut self, c: &mut Compiler) -> Token {
        let mut current: Token = self.next_raw(c);
        while match current.t {
            TokenType::Invalid | TokenType::Comment | TokenType::Whitespace => true,
            _ => false
        } {
            current = self.next_raw(c); 
        }
        return current;
    }
}