use std::fmt;
use crate::{Compiler, PathIdx, ScopeIdx, Source, StringIdx, Type, TypeIdx};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    // meta or used in structure of other nodes
    Invalid,
    IsPublic,
    IsExternal,
    IsExported,
    IsConstant,
    Block,
    ArgumentList,
    ArgumentDecl,
    UsedPath,
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
    While,
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
    PointerType, ReferenceType, 
    U8Type, U16Type, U32Type, U64Type, UsizeType,
    S8Type, S16Type, S32Type, S64Type,
    F32Type, F64Type, 
    UnitType, 
    BoolType,
    FunctionType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeValue {
    None,
    String(StringIdx),
    Path(PathIdx),
    Scope(ScopeIdx)
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AstNode {
    pub t: NodeType,
    pub source: Source,
    pub value: NodeValue,
    pub children: Vec<AstNode>,
    pub result_type: TypeIdx
}

impl AstNode {
    pub fn new(
        t: NodeType, source: Source, value: NodeValue, children: Vec<AstNode>,
        result_type: TypeIdx
    ) -> AstNode {
        return AstNode { t, source, value, children, result_type };
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