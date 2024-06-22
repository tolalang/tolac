use std::collections::HashMap;
use crate::{
    Compiler, AstNode, NodeType, TypeIdx, StringIdx, ScopeIdx, NodeValue
};

#[derive(Debug)]
pub struct TypeChecker<'c> {
    comp: &'c mut Compiler,
    templs: Vec<HashMap<StringIdx, TypeIdx>>,
    scopes: Vec<ScopeIdx>
}

impl<'c> TypeChecker<'c> {
    pub fn new(comp: &mut Compiler) -> TypeChecker {
        return TypeChecker {
            comp,
            templs: Vec::new(),
            scopes: Vec::new()
        };
    }

    pub fn check_types(&mut self) {
        // TODO!
    }

    pub fn check_function(&mut self) {
        // TODO!
    }

    pub fn check_node(&mut self, n: &AstNode, e: Option<TypeIdx>) -> AstNode {
        match (n.t, n.value) {
            (NodeType::Add | NodeType::Subtract | NodeType::Multiply
                    | NodeType::Divide | NodeType::Remainder, _) => {
                let left: AstNode = self.check_node(&n.children[0], e);
                let right: AstNode = self.check_node(&n.children[1], e);
                let rtype: TypeIdx = self.match_types(left.rtype, right.rtype);
                return AstNode::new(
                    n.t, n.source, n.value, vec!(left, right), rtype
                );
            }
            (NodeType::Call, _) => {
                match (n.children[0].t, n.children[0].value) {
                    (NodeType::PathAccess, NodeValue::Path(called)) => {
                        
                    } 
                    _ => {}
                }
                todo!("function pointer calls")
            }
            _ => unreachable!("node must be valid")
        }
    }

    pub fn match_types(&self, left: TypeIdx, right: TypeIdx) -> TypeIdx {
        // TODO: actual implementation
        //       if any type is unknown, assume it is correct
        //       return Unknown type if types don't match
        return left;
    }
}