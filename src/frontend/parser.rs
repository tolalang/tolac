use crate::{Compiler, Error, PathIdx, Source, StringIdx};
use crate::lexer::{Lexer, Token, TokenType};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Invalid,
    IsPublic,
    IsExternal,
    IsConstant,
    Block,
    ArgumentList,
    Argument,
    // statements
    ModuleDecl,
    UsageDecl,
    StructDecl,
    EnumDecl,
    InterfaceDecl,
    FunctionDecl,
    VariableDecl,
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
    IntegerLiteral, FloatLiteral, StringLiteral, 
    CStringLiteral, UnitLiteral, BooleanLiteral,
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

#[derive(Clone)]
pub struct AstNode {
    pub t: NodeType,
    pub source: Source,
    pub value: NodeValue,
    pub children: Vec<AstNode>
}

impl AstNode {
    pub fn new(
        t: NodeType, source: Source, value: NodeValue, children: Vec<AstNode>
    ) -> AstNode {
        return AstNode { t, source, value, children };
    }

    pub fn empty(t: NodeType, source: Source) -> AstNode {
        return AstNode {
            t, source, 
            value: NodeValue::None, children: Vec::new() 
        };
    }
}

impl fmt::Debug for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.t)
            .field(&self.value)
            .field(&self.children)
            .finish()
    }
}


const PREC_NONE: usize = 0;
const PREC_NEGATE: usize = 2;
const PREC_NOT: usize = 2;
const PREC_ADDRESS_OF: usize = 2;
const PREC_DEREF: usize = 2;
const PREC_TYPE_CAST: usize = 3;
const PREC_MULTIPLY: usize = 4;
const PREC_DIVIDE: usize = 4;
const PREC_REMAINDER: usize = 4;
const PREC_ADD: usize = 5;
const PREC_SUBTRACT: usize = 5;
const PREC_LESS_THAN: usize = 6;
const PREC_LESS_THAN_EQUAL: usize = 6;
const PREC_GREATER_THAN: usize = 6;
const PREC_GREATER_THAN_EQUAL: usize = 6;
const PREC_EQUAL: usize = 7;
const PREC_NOT_EQUAL: usize = 7;
const PREC_AND: usize = 8;
const PREC_OR: usize = 8;

fn get_infix_precedence(t: TokenType) -> usize {
    match t {
        TokenType::KeywordAs => PREC_TYPE_CAST,
        TokenType::Asterisk => PREC_MULTIPLY,
        TokenType::Slash => PREC_DIVIDE,
        TokenType::Percent => PREC_REMAINDER,
        TokenType::Plus => PREC_ADD,
        TokenType::Minus => PREC_SUBTRACT,
        TokenType::LessThan => PREC_LESS_THAN,
        TokenType::LessThanEqual => PREC_LESS_THAN_EQUAL,
        TokenType::GreaterThan => PREC_GREATER_THAN,
        TokenType::GreaterThanEqual => PREC_GREATER_THAN_EQUAL,
        TokenType::DoubleEqual => PREC_EQUAL,
        TokenType::NotEqual => PREC_NOT_EQUAL,
        TokenType::DoubleAmpersand => PREC_AND,
        TokenType::DoublePipe => PREC_OR,
        _ => PREC_NONE
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
                    Vec::new()
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
                let mut children: Vec<AstNode> = Vec::new();
                if is_public {
                    children.push(AstNode::empty(
                        NodeType::IsPublic, self.current.source
                    ));
                }
                if is_external {
                    children.push(AstNode::empty(
                        NodeType::IsExternal, self.current.source
                    ));
                }
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let mut t_args: Vec<AstNode> = Vec::new();
                let t_args_start: Source = self.current.source;
                let mut t_args_end: Source = self.current.source;
                if self.current.t == TokenType::BracketOpen {
                    self.next();
                    while self.current.t != TokenType::BracketClose {
                        self.expect(&[TokenType::Identifier])?;
                        t_args.push(AstNode::new(
                            NodeType::Argument, self.current.source,
                            NodeValue::String(self.current.content),
                            Vec::new()
                        ));
                        self.next();
                        self.expect(&[
                            TokenType::Comma, TokenType::BracketClose
                        ])?;
                        if self.current.t == TokenType::Comma {
                            self.next();
                        }
                    }
                    t_args_end = self.current.source;
                    self.next();
                }
                children.push(AstNode::new(
                    NodeType::ArgumentList,
                    Source::across(t_args_start, t_args_end),
                    NodeValue::None,
                    t_args
                ));
                self.expect(&[TokenType::ParenOpen])?;
                self.next();
                let mut args: Vec<AstNode> = Vec::new();
                let args_start: Source = self.current.source;
                while self.current.t != TokenType::ParenClose {
                    let mut arg_children: Vec<AstNode> = Vec::new();
                    let arg_start: Source = self.current.source;
                    if self.current.t == TokenType::KeywordConst {
                        arg_children.push(AstNode::empty(
                            NodeType::IsConstant, self.current.source,
                        ));
                        self.next();
                    }
                    self.expect(&[TokenType::Identifier])?;
                    let arg_name: StringIdx = self.current.content;
                    self.next();
                    let arg_type: AstNode = self.parse_type()?;
                    let arg_end: Source = arg_type.source;
                    arg_children.push(arg_type);
                    args.push(AstNode::new(
                        NodeType::Argument,
                        Source::across(arg_start, arg_end),
                        NodeValue::String(arg_name),
                        arg_children
                    ));
                    self.expect(&[TokenType::Comma, TokenType::ParenClose])?;
                    if self.current.t == TokenType::Comma {
                        self.next();
                    }
                }
                children.push(AstNode::new(
                    NodeType::ArgumentList,
                    Source::across(args_start, self.current.source),
                    NodeValue::None,
                    args
                ));
                self.next();
                if self.current.t == TokenType::Colon {
                    self.next();
                    children.push(self.parse_type()?);
                }
                self.expect(&[TokenType::BraceOpen])?;
                let body_start: Source = self.current.source;
                self.next();
                let body: Vec<AstNode> = self.parse_block(false);
                self.expect(&[TokenType::BraceClose])?;
                children.push(AstNode::new(
                    NodeType::Block, 
                    Source::across(body_start, self.current.source),
                    NodeValue::None,
                    body
                ));
                let end: Source = self.current.source;
                self.next();
                return Ok(AstNode::new(
                    NodeType::FunctionDecl,
                    Source::across(start, end),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordVar => {
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let value_type: AstNode = self.parse_type()?;
                let mut end: Source = value_type.source;
                let mut children: Vec<AstNode> = vec![value_type];
                if self.current.t == TokenType::Equal {
                    self.next();
                    let value: AstNode = self.parse_full_expression()?;
                    end = value.source;
                    children.push(value);
                }
                return Ok(AstNode::new(
                    NodeType::VariableDecl,
                    Source::across(start, end),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordConst => {
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let value_type: AstNode = self.parse_type()?;
                let mut end: Source = value_type.source;
                let mut children: Vec<AstNode> = vec![
                    AstNode::new(
                        NodeType::IsConstant, start,
                        NodeValue::None, Vec::new()
                    ),
                    value_type
                ];
                if self.current.t == TokenType::Equal {
                    self.next();
                    let value: AstNode = self.parse_full_expression()?;
                    end = value.source;
                    children.push(value);
                }
                return Ok(AstNode::new(
                    NodeType::VariableDecl,
                    Source::across(start, end),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordReturn => {
                self.next();
                let returned: AstNode = self.parse_full_expression()?;
                return Ok(AstNode::new(
                    NodeType::Return,
                    Source::across(start, returned.source),
                    NodeValue::None,
                    vec![returned]
                ));
            }
            TokenType::KeywordContinue => {
                self.next();
                return Ok(AstNode::new(
                    NodeType::Continue,
                    start,
                    NodeValue::None, Vec::new()
                ));
            }
            TokenType::KeywordBreak => {
                self.next();
                return Ok(AstNode::new(
                    NodeType::Break,
                    start,
                    NodeValue::None, Vec::new()
                ));
            }
            TokenType::KeywordIf => {
                self.next();
                let mut children: Vec<AstNode> = Vec::new();
                children.push(self.parse_full_expression()?);
                self.expect(&[TokenType::BraceOpen])?;
                let if_start: Source = self.current.source;
                self.next();
                let if_body: Vec<AstNode> = self.parse_block(false);
                self.expect(&[TokenType::BraceClose])?;
                children.push(AstNode::new(
                    NodeType::Block, 
                    Source::across(if_start, self.current.source), 
                    NodeValue::None, 
                    if_body
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
                        else_body
                    ));
                }
                return Ok(AstNode::new(
                    NodeType::If,
                    Source::across(
                        start, 
                        self.last.expect("cannot be the first token").source
                    ),
                    NodeValue::None,
                    children
                ));
            }
            TokenType::KeywordLoop => {
                todo!()
            }
            _ => {
                let expr: AstNode = self.parse_full_expression()?;
                if self.current.t == TokenType::Equal {
                    self.next();
                    let value: AstNode = self.parse_full_expression()?;
                    return Ok(AstNode::new(
                        NodeType::Assign,
                        Source::across(start, value.source),
                        NodeValue::None,
                        vec![expr, value]
                    )); 
                }
                return Ok(expr);
            }
        }
    }

    fn parse_full_expression(&mut self) -> Result<AstNode, ()> {
        return self.parse_expression(usize::MAX);
    }

    fn parse_expression(&mut self, precedence: usize) -> Result<AstNode, ()> {
        let mut previous: Option<AstNode> = None;
        loop {
            let start: Source = self.current.source;
            // infix operators
            if let Some(left) = previous {
                if get_infix_precedence(self.current.t) >= precedence {
                    return Ok(left);
                }
                match self.current.t {
                    TokenType::ParenOpen => {
                        todo!("reassign result to 'previous'")
                    }
                    TokenType::Dot => {
                        todo!("reassign result to 'previous'")
                    }
                    TokenType::KeywordAs => {
                        todo!("reassign result to 'previous'")
                    }
                    TokenType::Plus | TokenType::Minus |
                    TokenType::Asterisk | TokenType::Slash |
                    TokenType::Percent |
                    TokenType::LessThan | TokenType::LessThanEqual |
                    TokenType::GreaterThan | TokenType::GreaterThanEqual |
                    TokenType::DoubleEqual | TokenType::NotEqual |
                    TokenType::DoublePipe | TokenType::DoubleAmpersand => {
                        todo!("reassign result to 'previous'")
                    }
                    _ => return Ok(left)
                }
                continue;
            }
            // prefix operators and literals
            match self.current.t {
                TokenType::Identifier => {
                    todo!("reassign result to 'previous'")
                }
                TokenType::KeywordUnit => {
                    self.next();
                    previous = Some(AstNode::new(
                        NodeType::UnitLiteral, start,
                        NodeValue::None, Vec::new()
                    ));
                }
                TokenType::KeywordSizeof => {
                    todo!("reassign result to 'previous'")
                }
                TokenType::KeywordTrue | TokenType::KeywordFalse |
                TokenType::Integer | TokenType::Float |
                TokenType::String | TokenType::CString => {
                    let nt: NodeType = match self.current.t {
                        TokenType::KeywordTrue | 
                        TokenType::KeywordFalse => NodeType::BooleanLiteral,
                        TokenType::Integer => NodeType::IntegerLiteral,
                        TokenType::Float => NodeType::FloatLiteral,
                        TokenType::String => NodeType::StringLiteral,
                        TokenType::CString => NodeType::CStringLiteral,
                        _ => unreachable!()
                    };
                    let value: StringIdx = self.current.content;
                    self.next();
                    previous = Some(AstNode::new(
                        nt, start, NodeValue::String(value), Vec::new()
                    ));
                }
                TokenType::Ampersand |
                TokenType::Asterisk |
                TokenType::ExclamationMark |
                TokenType::Minus => {
                    todo!("reassign result to 'previous'")
                }
                TokenType::ParenOpen => {
                    todo!("reassign result to 'previous'")
                }
                _ => return Err(self.report_unexpected())
            }
        }
    }

    fn parse_type(&mut self) -> Result<AstNode, ()> {
        let start: Source = self.current.source;
        match self.current.t {
            TokenType::Asterisk => {
                self.next();
                let mut children: Vec<AstNode> = Vec::new();
                if self.current.t == TokenType::KeywordConst {
                    children.push(AstNode::empty(
                        NodeType::IsConstant, self.current.source
                    ));
                    self.next();
                }
                let ptr_type: AstNode = self.parse_type()?;
                let end: Source = ptr_type.source;
                children.push(ptr_type);
                return Ok(AstNode::new(
                    NodeType::PointerType,
                    Source::across(start, end),
                    NodeValue::None,
                    children
                ));
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
                    nt, start, NodeValue::None, Vec::new()
                ));
            }
            TokenType::KeywordFun => {
                todo!()
            }
            TokenType::Identifier => {
                let name: StringIdx = self.current.content;
                let mut t_args: Vec<AstNode> = Vec::new();
                let mut t_args_start: Source = self.current.source;
                let mut t_args_end: Source = self.current.source;
                self.next();
                if self.current.t == TokenType::BracketOpen {
                    t_args_start = self.current.source;
                    self.next();
                    while self.current.t != TokenType::BracketClose {
                        t_args.push(self.parse_type()?);
                        self.expect(&[
                            TokenType::Comma, TokenType::BracketClose
                        ])?;
                        if self.current.t == TokenType::Comma {
                            self.next();
                        }
                    }
                    t_args_end = self.current.source;
                    self.next();
                }
                return Ok(AstNode::new(
                    NodeType::NamespaceAccess, start,
                    NodeValue::String(name),
                    vec![AstNode::new(
                        NodeType::ArgumentList,
                        Source::across(t_args_start, t_args_end),
                        NodeValue::None,
                        t_args
                    )]
                )); 
            }
            _ => return Err(self.report_unexpected())
        }
    }

}