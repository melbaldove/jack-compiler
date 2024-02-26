use std::{collections::HashMap, fmt};

pub struct SymbolTable {
    map: HashMap<String, Symbol>,
    index_map: HashMap<Category, usize>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Category {
    Static,
    Field,
    Arg,
    Var,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Static => write!(f, "static"),
            Category::Field => write!(f, "field"),
            Category::Arg => write!(f, "arg"),
            Category::Var => write!(f, "var"),
        }
    }
}

struct Symbol {
    _type: String,
    kind: Category,
    index: usize,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let index_map = HashMap::new();

        let mut table = SymbolTable {
            map: HashMap::new(),
            index_map,
        };
        table.reset();
        table
    }

    pub fn reset(&mut self) {
        self.index_map.insert(Category::Static, 0);
        self.index_map.insert(Category::Field, 0);
        self.index_map.insert(Category::Arg, 0);
        self.index_map.insert(Category::Var, 0);
    }

    pub fn define(&mut self, name: &str, _type: &str, kind: Category) {
        let index = self.var_count(kind);
        let symbol = Symbol {
            _type: String::from(_type),
            index,
            kind,
        };
        self.map.insert(String::from(name), symbol);
        self.index_map.insert(kind, index + 1);
    }

    pub fn var_count(&self, kind: Category) -> usize {
        *self.index_map.get(&kind).unwrap()
    }

    pub fn kind_of(&self, name: &str) -> Option<Category> {
        self.map.get(name).map(|s| s.kind)
    }

    pub fn type_of(&self, name: &str) -> Option<String> {
        self.map.get(name).map(|s| s._type.to_owned())
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.map.get(name).map(|s| s.index)
    }
}
