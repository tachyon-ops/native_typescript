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
    Block(BlockStmt<'a>),
    Var(VarDecl<'a>),
    Expr(ExprStmt<'a>),
    If(IfStmt<'a>),
    Switch(SwitchStmt<'a>),
    For(ForStmt<'a>),
    ForIn(ForInStmt<'a>),
    ForOf(ForOfStmt<'a>),
    While(WhileStmt<'a>),
    DoWhile(DoWhileStmt<'a>),
    Return(ReturnStmt<'a>),
    Throw(ThrowStmt<'a>),
    Try(TryStmt<'a>),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Labeled(LabeledStmt<'a>),
    Function(FunctionDecl<'a>),
    Class(ClassDecl<'a>),
    Import(ImportDecl<'a>),
    Export(ExportDecl<'a>),
    NativeImport(NativeImportDecl),
    NativeFunction(NativeFunctionDecl<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeImportDecl {
    pub name: Symbol,
    pub source: Symbol,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeFunctionDecl<'a> {
    pub name: Symbol,
    pub params: NodeList<'a, super::ast::Param<'a>>,
    pub return_type: Option<&'a super::ast::TypeNode<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockStmt<'a> {
    pub stmts: NodeList<'a, Stmt<'a>>,
    pub span: Span,
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
    pub ty: Option<TypeNode<'a>>,
    pub init: Option<&'a Expr<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprStmt<'a> {
    pub expr: &'a Expr<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IfStmt<'a> {
    pub test: &'a Expr<'a>,
    pub consequent: &'a Stmt<'a>,
    pub alternate: Option<&'a Stmt<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchStmt<'a> {
    pub discriminant: &'a Expr<'a>,
    pub cases: NodeList<'a, SwitchCase<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchCase<'a> {
    pub test: Option<&'a Expr<'a>>, // None for default
    pub consecutive: NodeList<'a, Stmt<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForStmt<'a> {
    pub init: Option<ForInit<'a>>,
    pub test: Option<&'a Expr<'a>>,
    pub update: Option<&'a Expr<'a>>,
    pub body: &'a Stmt<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ForInit<'a> {
    Var(VarDecl<'a>),
    Expr(&'a Expr<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForInStmt<'a> {
    pub left: ForInit<'a>,
    pub right: &'a Expr<'a>,
    pub body: &'a Stmt<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForOfStmt<'a> {
    pub is_await: bool,
    pub left: ForInit<'a>,
    pub right: &'a Expr<'a>,
    pub body: &'a Stmt<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhileStmt<'a> {
    pub test: &'a Expr<'a>,
    pub body: &'a Stmt<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoWhileStmt<'a> {
    pub body: &'a Stmt<'a>,
    pub test: &'a Expr<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReturnStmt<'a> {
    pub value: Option<&'a Expr<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrowStmt<'a> {
    pub argument: &'a Expr<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TryStmt<'a> {
    pub block: BlockStmt<'a>,
    pub handler: Option<CatchHandler<'a>>,
    pub finalizer: Option<BlockStmt<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatchHandler<'a> {
    pub param: Option<Symbol>, // simplified for Phase 1
    pub body: BlockStmt<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakStmt {
    pub label: Option<Symbol>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContinueStmt {
    pub label: Option<Symbol>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabeledStmt<'a> {
    pub label: Symbol,
    pub body: &'a Stmt<'a>,
    pub span: Span,
}

// ── Functions ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FunctionDecl<'a> {
    pub id: Option<Symbol>,
    pub params: NodeList<'a, Param<'a>>,
    pub body: Option<BlockStmt<'a>>,
    pub return_ty: Option<TypeNode<'a>>,
    pub is_async: bool,
    pub is_generator: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Param<'a> {
    pub name: Symbol,
    pub ty: Option<TypeNode<'a>>,
    pub init: Option<&'a Expr<'a>>,
    pub is_rest: bool,
    pub span: Span,
}

// ── Classes ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassDecl<'a> {
    pub id: Option<Symbol>,
    pub super_class: Option<&'a Expr<'a>>,
    pub body: NodeList<'a, ClassMember<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClassMember<'a> {
    Constructor(FunctionDecl<'a>),
    Method(MethodDecl<'a>),
    Property(PropertyDecl<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MethodDecl<'a> {
    pub key: Symbol,
    pub func: FunctionDecl<'a>,
    pub is_static: bool,
    pub access: Option<AccessModifier>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyDecl<'a> {
    pub key: Symbol,
    pub ty: Option<TypeNode<'a>>,
    pub init: Option<&'a Expr<'a>>,
    pub is_static: bool,
    pub access: Option<AccessModifier>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
}

// ── Imports & Exports ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportDecl<'a> {
    pub specifiers: NodeList<'a, ImportSpecifier>,
    pub source: Symbol,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSpecifier {
    Named(Symbol, Option<Symbol>), // (local, imported)
    Default(Symbol),
    Namespace(Symbol),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExportDecl<'a> {
    pub decl: Option<&'a Stmt<'a>>,
    pub specifiers: NodeList<'a, ExportSpecifier>,
    pub source: Option<Symbol>,
    pub is_default: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportSpecifier {
    pub local: Symbol,
    pub exported: Option<Symbol>,
}

// ── Expressions ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr<'a> {
    Number(f64, Span),
    String(Symbol, Span),
    Bool(bool, Span),
    Null(Span),
    Undefined(Span),
    This(Span),
    Ident(Symbol, Span),
    Member(MemberExpr<'a>),
    Index(IndexExpr<'a>),
    OptChain(OptChainExpr<'a>),
    Unary(UnaryExpr<'a>),
    Binary(BinaryExpr<'a>),
    Conditional(ConditionalExpr<'a>),
    Assign(AssignExpr<'a>),
    Call(CallExpr<'a>),
    New(NewExpr<'a>),
    Arrow(ArrowExpr<'a>),
    Function(FunctionDecl<'a>),
    Template(TemplateExpr<'a>),
    Spread(SpreadExpr<'a>),
    Array(ArrayExpr<'a>),
    Object(ObjectExpr<'a>),
    Paren(&'a Expr<'a>, Span),
    As(AsExpr<'a>),
    JSXElement(JSXElement<'a>),
    JSXText(Symbol, Span),
    JSXExpressionContainer(&'a Expr<'a>, Span),
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpr<'a> {
    pub test: &'a Expr<'a>,
    pub consequent: &'a Expr<'a>,
    pub alternate: &'a Expr<'a>,
    pub span: Span,
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody<'a> {
    Expr(&'a Expr<'a>),
    Block(BlockStmt<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrowExpr<'a> {
    pub params: NodeList<'a, Param<'a>>,
    pub body: ArrowBody<'a>,
    pub is_async: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateExpr<'a> {
    pub quasis: NodeList<'a, Symbol>,
    pub exprs: NodeList<'a, &'a Expr<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpreadExpr<'a> {
    pub argument: &'a Expr<'a>,
    pub span: Span,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct AsExpr<'a> {
    pub expr: &'a Expr<'a>,
    pub ty: TypeNode<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JSXElement<'a> {
    pub tag: Symbol,
    pub props: NodeList<'a, ObjProp<'a>>,
    pub children: NodeList<'a, &'a Expr<'a>>,
    pub span: Span,
}

// ── Types ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeNode<'a> {
    Number(Span),
    String(Span),
    Boolean(Span),
    BigInt(Span),
    Symbol(Span),
    Null(Span),
    Undefined(Span),
    Void(Span),
    Never(Span),
    Unknown(Span),
    Any(Span),
    Object(Span),
    LiteralNumber(f64, Span),
    LiteralString(Symbol, Span),
    LiteralBool(bool, Span),
    TypeRef(TypeRefNode<'a>),
    Array(AstArenaRef<'a, TypeNode<'a>>),
    Tuple(NodeList<'a, TypeNode<'a>>, Span),
    Function(FunctionTypeNode<'a>),
    Union(NodeList<'a, TypeNode<'a>>, Span),
    Intersection(NodeList<'a, TypeNode<'a>>, Span),
    Paren(AstArenaRef<'a, TypeNode<'a>>, Span),
    // Simplified for Phase 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstArenaRef<'a, T>(pub &'a T);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeRefNode<'a> {
    pub name: Symbol,
    pub type_args: Option<NodeList<'a, TypeNode<'a>>>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FunctionTypeNode<'a> {
    pub params: NodeList<'a, Param<'a>>,
    pub return_ty: AstArenaRef<'a, TypeNode<'a>>,
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
            Expr::Function(e) => e.span,
            Expr::Template(e) => e.span,
            Expr::Spread(e) => e.span,
            Expr::Array(e) => e.span,
            Expr::Object(e) => e.span,
            Expr::As(e) => e.span,
            Expr::JSXElement(e) => e.span,
            Expr::JSXText(_, s) => *s,
            Expr::JSXExpressionContainer(_, s) => *s,
        }
    }
}

impl<'a> Stmt<'a> {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Block(s) => s.span,
            Stmt::Var(s) => s.span,
            Stmt::Expr(s) => s.span,
            Stmt::If(s) => s.span,
            Stmt::Switch(s) => s.span,
            Stmt::For(s) => s.span,
            Stmt::ForIn(s) => s.span,
            Stmt::ForOf(s) => s.span,
            Stmt::While(s) => s.span,
            Stmt::DoWhile(s) => s.span,
            Stmt::Return(s) => s.span,
            Stmt::Throw(s) => s.span,
            Stmt::Try(s) => s.span,
            Stmt::Break(s) => s.span,
            Stmt::Continue(s) => s.span,
            Stmt::Labeled(s) => s.span,
            Stmt::Function(s) => s.span,
            Stmt::Class(s) => s.span,
            Stmt::Import(s) => s.span,
            Stmt::Export(s) => s.span,
            Stmt::NativeImport(s) => s.span,
            Stmt::NativeFunction(s) => s.span,
        }
    }
}

impl<'a> TypeNode<'a> {
    pub fn span(&self) -> Span {
        match self {
            TypeNode::Number(s) | TypeNode::String(s) | TypeNode::Boolean(s)
            | TypeNode::BigInt(s) | TypeNode::Symbol(s) | TypeNode::Null(s)
            | TypeNode::Undefined(s) | TypeNode::Void(s) | TypeNode::Never(s)
            | TypeNode::Unknown(s) | TypeNode::Any(s) | TypeNode::Object(s)
            | TypeNode::LiteralNumber(_, s) | TypeNode::LiteralString(_, s)
            | TypeNode::LiteralBool(_, s) | TypeNode::Tuple(_, s)
            | TypeNode::Union(_, s) | TypeNode::Intersection(_, s)
            | TypeNode::Paren(_, s) => *s,
            TypeNode::TypeRef(t) => t.span,
            TypeNode::Array(t) => t.0.span(),
            TypeNode::Function(t) => t.span,
        }
    }
}
