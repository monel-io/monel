/// Unified AST for `.mn` files.
/// Contracts and implementation coexist in the same tree.

/// A parsed `.mn` source file.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub imports: Vec<Import>,
    pub declarations: Vec<Decl>,
}

/// `use std/io {println, readln}`
#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub names: Vec<String>,
}

/// Top-level declaration.
#[derive(Debug, Clone)]
pub enum Decl {
    Fn(FnDecl),
    Type(TypeDecl),
    Trait(TraitDecl),
    Impl(ImplBlock),
    Const(ConstDecl),
    StateMachine(StateMachineDecl),
}

/// Visibility: `pub` or private (default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
}

/// A function declaration with contracts and body.
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub visibility: Visibility,
    pub is_async: bool,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    // Contracts (all optional)
    pub doc: Option<String>,
    pub requires: Vec<Expr>,
    pub ensures: Vec<EnsuresClause>,
    pub effects: Vec<EffectRef>,
    pub panics: PanicsDecl,
    pub complexity: Option<ComplexityDecl>,
    pub fails: Vec<FailsEntry>,
    // Body
    pub body: Option<FnBody>,
}

/// Function body: either compact (`= expr`) or block (indented statements).
#[derive(Debug, Clone)]
pub enum FnBody {
    Compact(Expr),
    Block(Vec<Stmt>),
}

/// An ensures clause: unconditional, `ok =>`, or `err(Variant) =>`.
#[derive(Debug, Clone)]
pub enum EnsuresClause {
    Always(Expr),
    Ok(Expr),
    Err(String, Expr),
    ResultIs(Pattern, Expr),
}

/// Panics declaration.
#[derive(Debug, Clone)]
pub enum PanicsDecl {
    Unspecified,
    Never,
    Condition(Expr),
}

/// `complexity: O(n) where n = self.history.len`
#[derive(Debug, Clone)]
pub struct ComplexityDecl {
    pub bound: String,
    pub where_var: Option<String>,
    pub where_expr: Option<Expr>,
}

/// `Overflow: "push on full stack"`
#[derive(Debug, Clone)]
pub struct FailsEntry {
    pub variant: String,
    pub description: String,
}

/// An effect reference: `Db.read`, `Clock.read`, `pure`
#[derive(Debug, Clone)]
pub struct EffectRef {
    pub category: String,
    pub operation: String,
}

/// A type declaration: struct, enum, alias, or distinct.
#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub visibility: Visibility,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub kind: TypeDeclKind,
    pub invariants: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum TypeDeclKind {
    Struct(Vec<Field>),
    Enum(Vec<Variant>),
    Alias(TypeExpr),
    Distinct(TypeExpr),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Field>,
    pub invariants: Vec<Expr>,
}

/// A trait declaration.
#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub visibility: Visibility,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub associated_types: Vec<String>,
    pub methods: Vec<FnDecl>,
}

/// `impl Stack<T>` or `impl Display for Stack<T>`
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub type_params: Vec<TypeParam>,
    pub trait_name: Option<String>,
    pub target_type: TypeExpr,
    pub where_clauses: Vec<WhereClause>,
    pub methods: Vec<FnDecl>,
    pub associated_types: Vec<(String, TypeExpr)>,
}

#[derive(Debug, Clone)]
pub struct WhereClause {
    pub param: String,
    pub bounds: Vec<String>,
}

/// `pub const DEFAULT_CAPACITY: Int = 16`
#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub visibility: Visibility,
    pub name: String,
    pub ty: TypeExpr,
    pub value: Expr,
}

/// State machine declaration.
#[derive(Debug, Clone)]
pub struct StateMachineDecl {
    pub name: String,
    pub doc: Option<String>,
    pub states: Vec<String>,
    pub transitions: Vec<TransitionDecl>,
    pub initial: String,
    pub terminal: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TransitionDecl {
    pub from: String,
    pub event: String,
    pub to: String,
}

/// Type parameter: `T`, `T: Display + Clone`
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<String>,
}

/// Function parameter: `self: mut Stack<T>`, `x: Int`
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
}

/// A type expression.
#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// `Int`, `String`, `T`
    Named(String),
    /// `Vec<T>`, `Result<Session, AuthError>`
    Generic(String, Vec<TypeExpr>),
    /// `Option<T>` (sugar, same as Generic)
    /// `(Int, String)`
    Tuple(Vec<TypeExpr>),
    /// `Fn(Int) -> Bool`
    Func(Vec<TypeExpr>, Box<TypeExpr>),
    /// `mut T`
    Mut(Box<TypeExpr>),
    /// `owned T`
    Owned(Box<TypeExpr>),
    /// `&T`
    Ref(Box<TypeExpr>),
    /// `()`
    Unit,
}

/// An expression.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer literal: `42`, `1_000_000`
    Int(i64),
    /// Float literal: `3.14`
    Float(f64),
    /// String literal (may contain interpolation segments)
    String(Vec<StringPart>),
    /// Boolean: `true`, `false`
    Bool(bool),
    /// `None`
    None_,
    /// Identifier: `x`, `self`, `result`
    Ident(String),
    /// Field access: `self.name`, `expr.len`
    Field(Box<Expr>, String),
    /// Method/function call: `f(x, y)`, `self.push(v)`
    Call(Box<Expr>, Vec<Expr>),
    /// Index: `arr[i]`
    Index(Box<Expr>, Box<Expr>),
    /// Slice: `arr[0..i]`
    Slice(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Binary op: `x + 1`, `a == b`, `x and y`
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    /// Unary op: `not x`, `-x`
    UnaryOp(UnaryOp, Box<Expr>),
    /// `old(expr)`
    Old(Box<Expr>),
    /// `try expr`
    Try(Box<Expr>),
    /// `return expr`
    Return(Box<Expr>),
    /// `if cond ... else ...`
    If(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>),
    /// `match expr | pat => body`
    Match(Box<Expr>, Vec<MatchArm>),
    /// `for var in expr body`
    For(String, Box<Expr>, Vec<Stmt>, Vec<Expr>),
    /// Block of statements (last is the value)
    Block(Vec<Stmt>),
    /// Struct construction: `Foo { field: val, ... }` or indented form
    Construct(String, Vec<(String, Expr)>),
    /// Enum variant construction: `GreetError.EmptyMessage`, `Ok(x)`
    EnumVariant(String, String, Vec<Expr>),
    /// Closure: `fn(x) => x + 1` or `fn() -> T { ... }`
    Closure(Vec<Param>, Box<Expr>),
    /// Range: `0..n`
    Range(Box<Expr>, Box<Expr>),
    /// Await: `expr.await`
    Await(Box<Expr>),
    /// `result is Some(s)` pattern test
    Is(Box<Expr>, Box<Pattern>),
    /// Assignment: `self.x = 5`
    Assign(Box<Expr>, Box<Expr>),
    /// Compound assignment: `self.x += 1`
    CompoundAssign(Box<Expr>, BinOp, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Interpolation(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// A statement.
#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let(String, Option<TypeExpr>, Expr),
}

/// A match arm: `| pattern => body`
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Stmt>,
}

/// A pattern for match/is.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// `_`
    Wildcard,
    /// `x`
    Binding(String),
    /// `Some(x)`, `GreetError.TooLong(n)`
    Constructor(String, Vec<Pattern>),
    /// `None`
    Ident(String),
    /// Literal: `42`, `"hello"`
    Literal(Expr),
}
