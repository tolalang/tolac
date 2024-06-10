use crate::Compiler;
use crate::lexer::{Lexer, Token};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
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
    USizeType,
    BoolType
}

#[derive(Debug, Clone)]
pub enum NodeChildren {
    Single(Box<AstNode>),
    Pair(Box<AstNode>, Box<AstNode>),
    Triple(Box<AstNode>, Box<AstNode>, Box<AstNode>),
    List(Vec<AstNode>)
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub t: NodeType,
    pub value: Option<String>,
    pub children: NodeChildren
}

pub struct Parser<'c> {
    comp: &'c mut Compiler,
    lexer: Lexer,
    current: Token
}

impl<'c> Parser<'c> {
    pub fn new(comp: &'c mut Compiler, mut lexer: Lexer) -> Parser<'c> {
        let current: Token = lexer.next(comp);
        return Parser { comp, lexer, current };
    }

    pub fn parse_file(&mut self) -> Vec<AstNode> {
        todo!()
    }
}