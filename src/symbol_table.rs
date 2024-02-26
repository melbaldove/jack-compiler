use std::{collections::HashMap, fmt};

pub struct SymbolTable {
    map: HashMap<String, Symbol>,
    index_map: HashMap<SegmentKind, usize>,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum SegmentKind {
    Static,
    Field,
    Arg,
    Var,
}

impl fmt::Display for SegmentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentKind::Static => write!(f, "static"),
            SegmentKind::Field => write!(f, "field"),
            SegmentKind::Arg => write!(f, "arg"),
            SegmentKind::Var => write!(f, "var"),
        }
    }
}

struct Symbol {
    _type: String,
    kind: SegmentKind,
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
        self.index_map.insert(SegmentKind::Static, 0);
        self.index_map.insert(SegmentKind::Field, 0);
        self.index_map.insert(SegmentKind::Arg, 0);
        self.index_map.insert(SegmentKind::Var, 0);
    }

    pub fn define(&mut self, name: &str, _type: &str, kind: SegmentKind) {
        let index = *self.index_map.get(&kind).unwrap();
        let symbol = Symbol {
            _type: String::from(_type),
            index,
            kind,
        };
        self.map.insert(String::from(name), symbol);
        self.index_map.insert(kind, index + 1);
    }

    pub fn kind_of(&self, name: &str) -> Option<SegmentKind> {
        self.map.get(name).map(|s| s.kind)
    }

    pub fn type_of(&self, name: &str) -> Option<String> {
        self.map.get(name).map(|s| s._type.to_owned())
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.map.get(name).map(|s| s.index)
    }
}
