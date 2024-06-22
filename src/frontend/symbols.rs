use std::collections::{HashMap, HashSet};

use crate::{AstNode, Compiler, Error, NodeType, NodeValue, PathIdx, StringIdx};


#[derive(Debug, Clone)]
pub struct Symbol {
    pub is_public: bool,
    pub decl_node: AstNode,
    pub template_args: Vec<StringIdx>,
    pub monomorphized_nodes: HashMap<Vec<AstNode>, AstNode>
}

impl Symbol {
    pub fn from_decl_node(node: AstNode) -> Symbol {
        let template_args: Vec<StringIdx> = node.children.iter()
            .find(|c| c.t == NodeType::ArgumentList)
            .map(|l| l.children.iter().map(|a|
                if let NodeValue::String(n) = a.value { n }
                else { unreachable!("must have correct template args!") }
            ).collect())
            .unwrap_or(Vec::new());
        return Symbol {
            is_public: node.children.iter()
                .find(|c| c.t == NodeType::IsPublic)
                .is_some(), 
            decl_node: node,
            template_args,
            monomorphized_nodes: HashMap::new()
        }
    }
}


#[derive(Debug, Clone)]
pub struct SymbolTable {
    modules: HashSet<PathIdx>,
    symbols: HashMap<PathIdx, Symbol>,
    unmangled: HashMap<StringIdx, PathIdx>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        return SymbolTable {
            modules: HashSet::new(),
            symbols: HashMap::new(),
            unmangled: HashMap::new()
        };
    }

    pub fn modules(&self) -> &HashSet<PathIdx> { &self.modules }
    pub fn symbols(&self) -> &HashMap<PathIdx, Symbol> { &self.symbols }
    pub(crate) fn symbols_mut(&mut self) -> &mut HashMap<PathIdx, Symbol> { 
        &mut self.symbols
    }
    pub fn unmangled(&self) -> &HashMap<StringIdx, PathIdx> { &self.unmangled }

    pub fn insert_file(&mut self, nodes: &[AstNode], c: &mut Compiler) {
        let mut curr_mod: PathIdx = c.paths.insert(&[]);
        for node in nodes {
            match (node.t, node.value) {
                (NodeType::ModuleDecl, NodeValue::Path(p)) => {
                    if self.modules.contains(&p) {
                        c.errors.push(Error::dynamic(
                            format!(
                                "the module '{}' is declared more than once",
                                p.display(c)
                            ), 
                            node.source
                        ));
                    }
                    self.modules.insert(p);
                    curr_mod = p;
                }
                (NodeType::UsageDecl, _) => {}
                (_, NodeValue::String(name)) => {
                    let exported: bool = node.children.iter()
                        .find(|c| c.t == NodeType::IsExported).is_some();
                    let mut full_path_segs: Vec<StringIdx> = c.paths
                        .get(curr_mod).into();
                    full_path_segs.push(name);
                    let full_path: PathIdx = c.paths.insert(&full_path_segs);
                    if exported {
                        if self.unmangled.contains_key(&name) {
                            c.errors.push(Error::dynamic(
                                format!(
                                    "the name '{}' is exported more than once",
                                    c.strings.get(name)
                                ), 
                                node.source
                            ));
                        }
                        self.unmangled.insert(name, full_path);
                    }
                    if self.symbols.contains_key(&full_path) {
                        c.errors.push(Error::dynamic(
                            format!(
                                "the symbol '{}' is declared more than once",
                                full_path.display(c)
                            ), 
                            node.source
                        ));
                    }
                    self.symbols.insert(
                        full_path, Symbol::from_decl_node(node.clone())
                    );
                }
                _ => unreachable!("node must be valid")
            }
        }
    }
}