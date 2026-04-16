use rustc_hash::FxHashMap;

/// Interned string. Equality is pointer equality after interning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(u32);

pub const SYM_EMPTY: Symbol = Symbol(0);
pub const SYM_CONSTRUCTOR: Symbol = Symbol(1);
pub const SYM_PROTOTYPE: Symbol = Symbol(2);
pub const SYM_LENGTH: Symbol = Symbol(3);
pub const SYM_UNDEFINED: Symbol = Symbol(4);
pub const SYM_NULL: Symbol = Symbol(5);
pub const SYM_NUMBER: Symbol = Symbol(6);
pub const SYM_STRING: Symbol = Symbol(7);
pub const SYM_BOOLEAN: Symbol = Symbol(8);
pub const SYM_OBJECT: Symbol = Symbol(9);
pub const SYM_FUNCTION: Symbol = Symbol(10);
pub const SYM_SYMBOL: Symbol = Symbol(11);
pub const SYM_BIGINT: Symbol = Symbol(12);

pub struct Interner {
    map: FxHashMap<String, Symbol>,
    strings: Vec<String>,
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl Interner {
    pub fn new() -> Self {
        let mut interner = Self {
            map: FxHashMap::default(),
            strings: Vec::new(),
        };

        // Pre-intern common symbols
        interner.intern_static("");             // 0
        interner.intern_static("constructor");  // 1
        interner.intern_static("prototype");    // 2
        interner.intern_static("length");       // 3
        interner.intern_static("undefined");    // 4
        interner.intern_static("null");         // 5
        interner.intern_static("number");       // 6
        interner.intern_static("string");       // 7
        interner.intern_static("boolean");      // 8
        interner.intern_static("object");       // 9
        interner.intern_static("function");     // 10
        interner.intern_static("symbol");       // 11
        interner.intern_static("bigint");       // 12

        interner
    }

    fn intern_static(&mut self, s: &str) -> Symbol {
        let sym = Symbol(self.strings.len() as u32);
        self.strings.push(s.to_string());
        self.map.insert(s.to_string(), sym);
        sym
    }

    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }

        let sym = Symbol(self.strings.len() as u32);
        let s_owned = s.to_string();
        self.strings.push(s_owned.clone());
        self.map.insert(s_owned, sym);
        sym
    }

    pub fn get(&self, sym: Symbol) -> &str {
        &self.strings[sym.0 as usize]
    }
}
