use std::{collections::HashMap, rc::Rc};

use crate::{PathIdx, Source, StringIdx};


#[derive(Debug, Clone, PartialEq, Eq, Hash)] 
pub enum Type {
    Unknown,
    Integer,
    U8, U16, U32, U64, Usize,
    S8, S16, S32, S64,
    Float,
    F32, F64,
    Unit, 
    Boolean,
    Pointer(bool, TypeIdx),
    Struct(PathIdx),
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeIdx(usize);

#[derive(Debug, Clone)]
pub struct TypeMap {
    indices: HashMap<Rc<Type>, TypeIdx>,
    types: Vec<Rc<Type>>
}

impl TypeMap {
    pub fn new() -> TypeMap {
        return TypeMap {
            indices: HashMap::new(),
            types: Vec::new()
        };
    }

    pub fn insert(&mut self, t: Type) -> TypeIdx {
        if let Some(idx) = self.indices.get(&t) { return *idx; }
        let idx = TypeIdx(self.types.len());
        let t: Rc<Type> = t.into();
        self.indices.insert(Rc::clone(&t), idx);
        self.types.push(t);
        return idx;
    }

    pub fn get<'s>(&'s self, idx: TypeIdx) -> &'s Type {
        return &self.types[idx.0];
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VariableState {
    Uninitialized,
    Initialized
}

impl VariableState {
    fn is_accessible(&self) -> bool {
        match self {
            VariableState::Uninitialized => false,
            VariableState::Initialized => true
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Variable {
    pub source: Source, 
    pub state: VariableState,
    pub value_type: TypeIdx
}

impl Variable {
    pub fn new(
        source: Source, state: VariableState, value_type: TypeIdx
    ) -> Variable {
        return Variable { source, state, value_type };
    }
}


#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<StringIdx, Vec<Variable>>
}

impl Scope {
    pub fn new() -> Scope {
        return Scope {
            variables: HashMap::new()
        }
    }

    pub fn insert(
        &mut self, 
        name: StringIdx, 
        source: Source, state: VariableState, value_type: TypeIdx
    ) {
        if let Some(v) = self.variables.get_mut(&name) {
            v.push(Variable::new(source, state, value_type));
        } else {
            self.variables.insert(
                name, vec!(Variable::new(source, state, value_type))
            );
        }
    }

    pub fn get_last<'s>(
        &'s self, name: StringIdx
    ) -> Option<&'s Variable> {
        return self.variables.get(&name).map(|v| v.last()).and_then(|v| v);
    }

    pub fn get_all<'s>(
        &'s self, name: StringIdx
    ) -> Option<&'s Vec<Variable>> {
        return self.variables.get(&name);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScopeIdx(usize);

#[derive(Debug, Clone)]
pub struct ScopeMap {
    scopes: Vec<Scope>
}

impl ScopeMap {
    pub fn new() -> ScopeMap {
        return ScopeMap {
            scopes: Vec::new()
        }
    }
}