use indexmap::IndexMap;
use tsnat_common::interner::Symbol;

/// A cheap handle to a type. Types are stored in the TypeArena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

pub const TYPE_NEVER: TypeId = TypeId(0);
pub const TYPE_UNKNOWN: TypeId = TypeId(1);
pub const TYPE_ANY: TypeId = TypeId(2);
pub const TYPE_NULL: TypeId = TypeId(3);
pub const TYPE_UNDEFINED: TypeId = TypeId(4);
pub const TYPE_VOID: TypeId = TypeId(5);
pub const TYPE_NUMBER: TypeId = TypeId(6);
pub const TYPE_STRING: TypeId = TypeId(7);
pub const TYPE_BOOLEAN: TypeId = TypeId(8);
pub const TYPE_BIGINT: TypeId = TypeId(9);
pub const TYPE_SYMBOL: TypeId = TypeId(10);

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitives
    Number,
    String,
    Boolean,
    BigInt,
    Symbol,
    Null,
    Undefined,
    Void,
    Never,
    Unknown,
    Any,

    // Literals
    LiteralNumber(f64),
    LiteralString(Symbol),
    LiteralBool(bool),

    // Composite
    Object(ObjectType),
    Array(TypeId),
    Function(FunctionType),

    // Combinators
    Union(Vec<TypeId>),
    Intersection(Vec<TypeId>),

    // Generics & Inference
    TypeParam(TypeParamDecl),
    Generic(GenericType),

    // Conditionals & Mapping
    Conditional(ConditionalType),
    Mapped(MappedType),
    IndexedAccess(TypeId, TypeId), // T[K]
    TemplateLiteral(TemplateLiteralType),

    // Intrinsic helpers
    Keyof(TypeId),
    Typeof(TypeId),
    Infer(Symbol),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType {
    pub properties: IndexMap<Symbol, PropertyType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyType {
    pub ty: TypeId,
    pub optional: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params: Vec<ParamType>,
    pub return_ty: TypeId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamType {
    pub name: Symbol,
    pub ty: TypeId,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParamDecl {
    pub name: Symbol,
    pub constraint: Option<TypeId>,
    pub default: Option<TypeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericType {
    pub target: TypeId,
    pub args: Vec<TypeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalType {
    pub check_type: TypeId,
    pub extends_type: TypeId,
    pub true_type: TypeId,
    pub false_type: TypeId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedType {
    pub type_param: TypeParamDecl, // The K in keyof T
    pub type_def: TypeId,          // The T[K] part
    pub readonly_mod: Option<bool>, // true = +readonly, false = -readonly
    pub optional_mod: Option<bool>, // true = +?, false = -?
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateLiteralType {
    pub quasis: Vec<Symbol>, // String parts
    pub exprs: Vec<TypeId>,  // Interpolated types
}

#[derive(Default)]
pub struct TypeArena {
    types: Vec<Type>,
}

impl TypeArena {
    pub fn new() -> Self {
        let mut arena = Self { types: Vec::new() };
        // Pre-populate built-in types to match constants
        arena.alloc(Type::Never);     // 0
        arena.alloc(Type::Unknown);   // 1
        arena.alloc(Type::Any);       // 2
        arena.alloc(Type::Null);      // 3
        arena.alloc(Type::Undefined); // 4
        arena.alloc(Type::Void);      // 5
        arena.alloc(Type::Number);    // 6
        arena.alloc(Type::String);    // 7
        arena.alloc(Type::Boolean);   // 8
        arena.alloc(Type::BigInt);    // 9
        arena.alloc(Type::Symbol);    // 10
        arena
    }

    pub fn alloc(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        id
    }

    pub fn get(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }
}
