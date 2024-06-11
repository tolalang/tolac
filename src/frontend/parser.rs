use crate::{Compiler, Error, PathIdx, Source, StringIdx};
use crate::lexer::{Lexer, Token, TokenType};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Invalid,
    Block,
    // statements
    ModuleDecl,
    UsageDecl,
    StructDecl,
    EnumDecl,
    InterfaceDecl,
    FunctionDecl,
    VariableDecl,
    ConstDecl,
    Return,
    Continue,
    Break,
    If,
    Loop,
    Assign, AssignAdd, AssignSubtract, AssignMultiply, AssignDivide, AssignRemainder,
    // expressions
    NamespaceAccess,
    VariableAccess,
    Call,
    IntegerLiteral, NumberLiteral, StringLiteral, CStringLiteral, UnitLiteral,
    MemberAccess,
    TypeCast, SizeOf,
    AddressOf, Deref,
    Add, Subtract, Multiply, Divide, Remainder, Negate,
    LessThan, GreaterThan, LessThanEqual, GreaterThanEqual, Equal, NotEqual,
    LogicalNot, LogicalAnd, LogicalOr,
    // types
    PointerType, FunctionPointerType,
    U8Type, U16Type, U32Type, U64Type,
    S8Type, S16Type, S32Type, S64Type,
    F32Type, F64Type, 
    UnitType, 
    UsizeType,
    BoolType
}

#[derive(Debug, Clone)]
pub enum NodeValue {
    None,
    String(StringIdx),
    Path(PathIdx)
}

#[derive(Debug, Clone)]
pub enum NodeChildren {
    None,
    Single(Box<AstNode>),
    Pair(Box<AstNode>, Box<AstNode>),
    List(Vec<AstNode>)
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub t: NodeType,
    pub source: Source,
    pub value: NodeValue,
    pub children: NodeChildren
}

impl AstNode {
    pub fn new(
        t: NodeType, source: Source, value: NodeValue, children: NodeChildren
    ) -> AstNode {
        return AstNode { t, source, value, children };
    }
}


pub struct Parser<'c> {
    comp: &'c mut Compiler,
    lexer: Lexer,
    last: Option<Token>,
    current: Token
}

impl<'c> Parser<'c> {
    pub fn new(comp: &'c mut Compiler, mut lexer: Lexer) -> Parser<'c> {
        let current: Token = lexer.next(comp);
        return Parser { comp, lexer, last: None, current };
    }

    fn next(&mut self) {
        self.last = Some(self.current);
        self.current = self.lexer.next(self.comp);
    }

    fn report_unexpected(&mut self) {
        let e: String = format!(
            "unexpected {}{}",
            self.current.display(self.comp),
            self.last.map(|t| 
                format!(" after {}", t.display(self.comp))
            ).unwrap_or_else(|| String::from(""))
        );
        let s: Source = if self.current.t == TokenType::EndOfFile {
            self.last.unwrap_or(self.current).source
        } else { self.current.source };
        self.comp.errors.push(Error::dynamic(e, s));
        loop {
            let recovered: bool = match self.current.t {
                TokenType::Semicolon |
                TokenType::BraceClose |
                TokenType::EndOfFile => true,
                _ => false
            };
            if recovered { break; }
            self.next();
        }
    }

    fn expect(&mut self, tt: &[TokenType]) -> Result<(), ()> {
        if tt.contains(&self.current.t) {
            return Ok(());
        }
        self.report_unexpected();
        return Err(());
    }

    fn expect_not(&mut self, tt: &[TokenType]) -> Result<(), ()> {
        if !tt.contains(&self.current.t) {
            return Ok(());
        }
        self.report_unexpected();
        return Err(());
    }

    pub fn parse_file(&mut self) -> Vec<AstNode> {
        return self.parse_block(true);
    }

    fn parse_block(&mut self, global: bool) -> Vec<AstNode> {
        let mut nodes: Vec<AstNode> = Vec::new();
        loop {
            while self.current.t == TokenType::Semicolon {
                self.next();
            }
            let end: bool = match self.current.t {
                TokenType::BraceClose |
                TokenType::EndOfFile => true,
                _ => false
            };
            if end { break; }
            if let Ok(n) = self.parse_statement(global) {
                nodes.push(n);
            }
        }
        return nodes;
    }

    fn parse_path(&mut self) -> Result<PathIdx, ()> {
        let mut p: Vec<StringIdx> = Vec::new();
        self.expect(&[TokenType::Identifier])?;
        p.push(self.current.content);
        self.next();
        while self.current.t == TokenType::DoubleColon {
            self.next();
            self.expect(&[TokenType::Identifier])?;
            p.push(self.current.content);
            self.next();
        }
        return Ok(self.comp.paths.insert(&p));
    }

    fn parse_statement(&mut self, global: bool) -> Result<AstNode, ()> {
        let start: Source = self.current.source;
        let is_public: bool = self.current.t == TokenType::KeywordPub;
        if is_public {
            self.next();
            self.expect(&[
                TokenType::KeywordStruct, TokenType::KeywordEnum,
                TokenType::KeywordInterface, TokenType::KeywordFun,
                TokenType::KeywordVar, TokenType::KeywordConst,
                TokenType::KeywordExt
            ])?;
        }
        let is_external: bool = self.current.t == TokenType::KeywordExt;
        if is_external {
            self.next();
            self.expect(&[
                TokenType::KeywordFun,
                TokenType::KeywordVar, TokenType::KeywordConst
            ])?;
        }
        if global {
            self.expect(&[
                TokenType::KeywordMod, TokenType::KeywordUse,
                TokenType::KeywordStruct, TokenType::KeywordEnum,
                TokenType::KeywordInterface, TokenType::KeywordFun,
                TokenType::KeywordVar, TokenType::KeywordConst
            ])?;
        } else {
            self.expect_not(&[
                TokenType::KeywordMod, TokenType::KeywordUse,
                TokenType::KeywordStruct, TokenType::KeywordEnum,
                TokenType::KeywordInterface, TokenType::KeywordFun
            ])?;
        }
        match self.current.t {
            TokenType::KeywordMod => {
                self.next();
                let name: PathIdx = self.parse_path()?;
                return Ok(AstNode::new(
                    NodeType::ModuleDecl,
                    Source::across(
                        start, 
                        self.last.expect("cannot be the first token").source
                    ),
                    NodeValue::Path(name),
                    NodeChildren::None
                ));
            }
            TokenType::KeywordUse => {
                todo!()
            }
            TokenType::KeywordStruct => {
                todo!()
            }
            TokenType::KeywordEnum => {
                todo!()
            }
            TokenType::KeywordInterface => {
                todo!()
            }
            TokenType::KeywordFun => {
                todo!()
            }
            TokenType::KeywordVar => {
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let value_type: AstNode = self.parse_type()?;
                self.expect(&[TokenType::Equal])?;
                self.next();
                let value: AstNode = self.parse_expression()?;
                return Ok(AstNode::new(
                    NodeType::VariableDecl,
                    Source::across(start, value.source),
                    NodeValue::String(name),
                    NodeChildren::Pair(value_type.into(), value.into())
                ));
            }
            TokenType::KeywordConst => {
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let value_type: AstNode = self.parse_type()?;
                self.expect(&[TokenType::Equal])?;
                self.next();
                let value: AstNode = self.parse_expression()?;
                return Ok(AstNode::new(
                    NodeType::ConstDecl,
                    Source::across(start, value.source),
                    NodeValue::String(name),
                    NodeChildren::Pair(value_type.into(), value.into())
                ));
            }
            TokenType::KeywordReturn => {
                self.next();
                let returned: AstNode = self.parse_expression()?;
                return Ok(AstNode::new(
                    NodeType::Return,
                    Source::across(start, returned.source),
                    NodeValue::None,
                    NodeChildren::Single(returned.into())
                ));
            }
            TokenType::KeywordContinue => {
                self.next();
                return Ok(AstNode::new(
                    NodeType::Continue,
                    start,
                    NodeValue::None, NodeChildren::None
                ));
            }
            TokenType::KeywordBreak => {
                self.next();
                return Ok(AstNode::new(
                    NodeType::Break,
                    start,
                    NodeValue::None, NodeChildren::None
                ));
            }
            TokenType::KeywordIf => {
                self.next();
                let mut children: Vec<AstNode> = Vec::new();
                children.push(self.parse_expression()?);
                self.expect(&[TokenType::BraceOpen])?;
                let if_start: Source = self.current.source;
                self.next();
                let if_body: Vec<AstNode> = self.parse_block(false);
                self.expect(&[TokenType::BraceClose])?;
                children.push(AstNode::new(
                    NodeType::Block, 
                    Source::across(if_start, self.current.source), 
                    NodeValue::None, 
                    NodeChildren::List(if_body)
                ));
                self.next();
                if self.current.t == TokenType::KeywordElse {
                    self.next();
                    self.expect(&[TokenType::KeywordIf, TokenType::BraceOpen])?;
                    let else_start: Source;
                    let else_body: Vec<AstNode>;
                    let else_end: Source;
                    if self.current.t == TokenType::KeywordIf {
                        else_start = self.current.source;
                        else_body = vec![self.parse_statement(false)?];
                        else_end = self.last
                            .expect("cannot be the first token").source;
                    } else {
                        else_start = self.current.source;
                        self.next();
                        else_body = self.parse_block(false);
                        self.expect(&[TokenType::BraceClose])?;
                        else_end = self.current.source;
                        self.next();
                    }
                    children.push(AstNode::new(
                        NodeType::Block, 
                        Source::across(else_start, else_end), 
                        NodeValue::None, 
                        NodeChildren::List(else_body)
                    ));
                }
                return Ok(AstNode::new(
                    NodeType::If,
                    Source::across(
                        start, 
                        self.last.expect("cannot be the first token").source
                    ),
                    NodeValue::None,
                    NodeChildren::List(children)
                ));
            }
            TokenType::KeywordLoop => {
                todo!()
            }
            _ => {
                todo!()
            }
        }
    }

    fn parse_expression(&mut self) -> Result<AstNode, ()> {
        todo!()
    }

    fn parse_type(&mut self) -> Result<AstNode, ()> {
        let start: Source = self.current.source;
        match self.current.t {
            TokenType::Asterisk => {
                todo!()
            }
            TokenType::KeywordU8 | TokenType::KeywordU16 |
            TokenType::KeywordU32 | TokenType::KeywordU64 |
            TokenType::KeywordS8 | TokenType::KeywordS16 |
            TokenType::KeywordS32 | TokenType::KeywordS64 |
            TokenType::KeywordF32 | TokenType::KeywordF64 |
            TokenType::KeywordUsize | TokenType::KeywordUnit |
            TokenType::KeywordBool => {
                let nt: NodeType = match self.current.t {
                    TokenType::KeywordU8 => NodeType::U8Type,
                    TokenType::KeywordU16 => NodeType::U16Type,
                    TokenType::KeywordU32 => NodeType::U32Type,
                    TokenType::KeywordU64 => NodeType::U64Type,
                    TokenType::KeywordS8 => NodeType::S8Type,
                    TokenType::KeywordS16 => NodeType::S16Type,
                    TokenType::KeywordS32 => NodeType::S32Type,
                    TokenType::KeywordS64 => NodeType::S64Type,
                    TokenType::KeywordF32 => NodeType::F32Type,
                    TokenType::KeywordF64 => NodeType::F64Type,
                    TokenType::KeywordUsize => NodeType::UsizeType,
                    TokenType::KeywordUnit => NodeType::UnitType,
                    TokenType::KeywordBool => NodeType::BoolType,
                    _ => unreachable!()
                };
                self.next();
                return Ok(AstNode::new(
                    nt, start, NodeValue::None, NodeChildren::None
                ));
            }
            TokenType::KeywordFun => {
                todo!()
            }
            TokenType::Identifier => {
                todo!()
            }
            _ => {
                todo!()
            }
        }
    }

}