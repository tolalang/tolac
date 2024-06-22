use std::collections::{HashMap, HashSet};

use crate::{AstNode, Compiler, NodeType, NodeValue, PathIdx, StringIdx, Type};

fn expand_wildcards(c: &mut Compiler, path: PathIdx) -> Vec<PathIdx> {
    let wildcard: StringIdx = c.strings.insert("*");
    let segs: &[StringIdx] = c.paths.get(path);
    if !segs.contains(&wildcard) {
        return vec!(path); 
    }
    return c.symbols.symbols().keys()
        .map(|p| *p)
        .filter(|symbol| {
            let symbol_segs = c.paths.get(*symbol);
            if symbol_segs.len() != segs.len() { return false; }
            for seg_i in 0..segs.len() {
                if segs[seg_i] == wildcard { continue; }
                if segs[seg_i] != symbol_segs[seg_i] { return false; }
            }
            return true;
        })
        .collect();
}

pub fn expand_paths(c: &mut Compiler) {
    let mut files: HashMap<StringIdx, Vec<AstNode>> = c.parsed_files.clone();
    for file in files.values_mut() {
        expand_file_paths(c, file);
    }
    c.parsed_files = files;
}

fn expand_file_paths(c: &mut Compiler, file: &[AstNode]) {
    let mut curr_mod: PathIdx = c.paths.insert(&[]);
    let mut curr_use: HashMap<StringIdx, PathIdx> = HashMap::new();
    for node in file {
        match (node.t, node.value) {
            (NodeType::ModuleDecl, NodeValue::Path(p)) => {
                curr_mod = p;
            }
            (NodeType::UsageDecl, _) => {
                for usage in &node.children {
                    let raw_path: PathIdx = if let NodeValue::Path(p)
                        = usage.value { p } 
                        else { unreachable!("node must be valid") };
                    for usage_path in expand_wildcards(c, raw_path) {
                        let alias: StringIdx = *c.paths.get(usage_path)
                            .last().expect("must have segment");
                        curr_use.insert(alias, usage_path);
                    }
                }
            }
            (_, NodeValue::String(name)) => {
                let mut full_path_segs: Vec<StringIdx> = c.paths
                    .get(curr_mod).into();
                full_path_segs.push(name);
                let full_path: PathIdx = c.paths.insert(&full_path_segs);
                let mut decl_node: AstNode = AstNode::new(
                    NodeType::Invalid, node.source, NodeValue::None,
                    Vec::new(), c.types.insert(Type::Unknown)
                );
                std::mem::swap(
                    &mut decl_node,
                    &mut c.symbols.symbols_mut()
                        .get_mut(&full_path).expect("should still exist!")
                        .decl_node
                );
                expand_node_paths(
                    c, &curr_use, &mut HashSet::new(), &mut decl_node
                );
                std::mem::swap(
                    &mut decl_node,
                    &mut c.symbols.symbols_mut()
                        .get_mut(&full_path).expect("should still exist!")
                        .decl_node
                );
            }
            _ => unreachable!("node must be valid")
        }
    }
}

fn expand_node_paths(
    c: &mut Compiler, 
    u: &HashMap<StringIdx, PathIdx>,
    v: &mut HashSet<StringIdx>,
    n: &mut AstNode
) {
    let mut cv: HashSet<StringIdx> = v.clone();
    match (n.t, n.value) {
        (NodeType::FunctionDecl, _) => {
            n.children
                .iter().filter(|c| c.t == NodeType::ArgumentList)
                .skip(1).next().expect("should have arg list")
                .children
                .iter().map(|a| {
                    if let NodeValue::String(name) = a.value { name }
                    else { unreachable!("should have a value") }
                })
                .for_each(|n| {
                    cv.insert(n);
                });
        }
        _ => {}
    }
    for child in &mut n.children {
        expand_node_paths(c, u, &mut cv, child);
    }
    match (n.t, n.value) {
        (NodeType::PathAccess, NodeValue::Path(rel_accessed_path)) => {
            let rel_accessed_segs: &[StringIdx] = c.paths
                .get(rel_accessed_path);
            let is_local_var: bool = rel_accessed_segs.len() == 1
                && v.contains(&rel_accessed_segs[0]);
            if !is_local_var {
                let alias: StringIdx = *rel_accessed_segs
                    .last().expect("has at least one segment");
                let mut accessed_path_segs: Vec<StringIdx> = Vec::new();
                accessed_path_segs.extend_from_slice(
                    u.get(&alias).map(|p| c.paths.get(*p)).unwrap_or(&[alias])
                );
                accessed_path_segs.extend_from_slice(
                    &rel_accessed_segs[1..]
                );
                let accessed_path: PathIdx = c.paths
                    .insert(&accessed_path_segs);
                n.value = NodeValue::Path(accessed_path);
            }
        }
        (NodeType::VariableDecl, NodeValue::String(name)) => {
            v.insert(name);
        }
        _ => {}
    }
}