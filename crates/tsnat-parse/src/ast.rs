use tsnat_common::span::Span;
use tsnat_common::interner::Symbol;
use crate::arena::NodeList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Module,
    Script,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Program<'a> {
    pub stmts: NodeList<'a, Stmt<'a>>,
    pub span: Span,
    pub source_type: SourceType,
}

// ── Statements ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stmt<'a> {
    Var(VarDecl<'a>),
    Expr(ExprStmt<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Const,
    Let,
    Var,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VarDecl<'a> {
    pub kind: VarKind,
    pub decls: NodeList<'a, VarDeclarator<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VarDeclarator<'a> {
    pub name: Symbol,
    pub init: Option<&'a Expr<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprStmt<'a> {
    pub expr: &'a Expr<'a>,
    pub span: Span,
}

// ── Expressions ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    // Literals
    Number(f64, Span),
    String(Symbol, Span),
    Bool(bool, Span),
    Null(Span),
    Undefined(Span),
    This(Span),

    // Identifiers & member access
    Ident(Symbol, Span),
    Member(MemberExpr<'a>),
    Index(IndexExpr<'a>),
    OptChain(OptChainExpr<'a>),

    // Operations
    Unary(UnaryExpr<'a>),
    Binary(BinaryExpr<'a>),
    Conditional(ConditionalExpr<'a>),
    Assign(AssignExpr<'a>),

    // Calls & construction
    Call(CallExpr<'a>),
    New(NewExpr<'a>),

    // Functions
    Arrow(ArrowExpr<'a>),

    // Template literals
    Template(TemplateExpr<'a>),

    // Spread
    Spread(SpreadExpr<'a>),

    // Array & Object literals
    Array(ArrayExpr<'a>),
    Object(ObjectExpr<'a>),

    // Grouping (preserved for span accuracy)
    Paren(&'a Expr<'a>, Span),

    // Type assertions (parsed but ignored in Phase 1)
    As(AsExpr<'a>),
}

// ── Unary ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, Plus, Not, BitNot,
    Typeof, Void, Delete,
    PreInc, PreDec, PostInc, PostDec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr<'a> {
    pub op: UnaryOp,
    pub operand: &'a Expr<'a>,
    pub span: Span,
}

// ── Binary ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Exp,
    EqEq, EqEqEq, BangEq, BangEqEq,
    Lt, Gt, LtEq, GtEq,
    And, Or, NullishCoalesce,
    BitAnd, BitOr, BitXor,
    Shl, Shr, UShr,
    In, Instanceof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr<'a> {
    pub op: BinaryOp,
    pub left: &'a Expr<'a>,
    pub right: &'a Expr<'a>,
    pub span: Span,
}

// ── Assignment ────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Eq,
    AddEq, SubEq, MulEq, DivEq, ModEq, ExpEq,
    AndEq, OrEq, NullishEq,
    BitAndEq, BitOrEq, BitXorEq,
    ShlEq, ShrEq, UShrEq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignExpr<'a> {
    pub op: AssignOp,
    pub left: &'a Expr<'a>,
    pub right: &'a Expr<'a>,
    pub span: Span,
}

// ── Conditional ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpr<'a> {
    pub test: &'a Expr<'a>,
    pub consequent: &'a Expr<'a>,
    pub alternate: &'a Expr<'a>,
    pub span: Span,
}

// ── Member access ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpr<'a> {
    pub object: &'a Expr<'a>,
    pub property: Symbol,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr<'a> {
    pub object: &'a Expr<'a>,
    pub index: &'a Expr<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptChainExpr<'a> {
    pub object: &'a Expr<'a>,
    pub property: Symbol,
    pub span: Span,
}

// ── Call & New ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr<'a> {
    pub callee: &'a Expr<'a>,
    pub args: NodeList<'a, &'a Expr<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr<'a> {
    pub callee: &'a Expr<'a>,
    pub args: NodeList<'a, &'a Expr<'a>>,
    pub span: Span,
}

// ── Arrow ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ArrowParam {
    pub name: Symbol,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody<'a> {
    Expr(&'a Expr<'a>),
    // Block body will be added in TASK-009
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrowExpr<'a> {
    pub params: NodeList<'a, ArrowParam>,
    pub body: ArrowBody<'a>,
    pub is_async: bool,
    pub span: Span,
}

// ── Template ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateExpr<'a> {
    pub quasis: NodeList<'a, Symbol>,
    pub exprs: NodeList<'a, &'a Expr<'a>>,
    pub span: Span,
}

// ── Spread ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SpreadExpr<'a> {
    pub argument: &'a Expr<'a>,
    pub span: Span,
}

// ── Array & Object literals ───────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpr<'a> {
    pub elements: NodeList<'a, &'a Expr<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpr<'a> {
    pub properties: NodeList<'a, ObjProp<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjProp<'a> {
    pub key: Symbol,
    pub value: &'a Expr<'a>,
    pub span: Span,
}

// ── As ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct AsExpr<'a> {
    pub expr: &'a Expr<'a>,
    /// Type annotation is skipped in Phase 1; we just record the span
    pub span: Span,
}

// ── Expr helpers ──────────────────────────────────────────────

impl<'a> Expr<'a> {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(_, s) | Expr::String(_, s) | Expr::Bool(_, s)
            | Expr::Null(s) | Expr::Undefined(s) | Expr::This(s)
            | Expr::Ident(_, s) | Expr::Paren(_, s) => *s,
            Expr::Member(e) => e.span,
            Expr::Index(e) => e.span,
            Expr::OptChain(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Conditional(e) => e.span,
            Expr::Assign(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::New(e) => e.span,
            Expr::Arrow(e) => e.span,
            Expr::Template(e) => e.span,
            Expr::Spread(e) => e.span,
            Expr::Array(e) => e.span,
            Expr::Object(e) => e.span,
            Expr::As(e) => e.span,
        }
    }
}
