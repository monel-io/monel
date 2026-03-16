/// AST for `.mn` implementation files.

use crate::ast::{Effect, Param, Type, TypeParam};

/// A complete implementation file.
#[derive(Debug, Clone)]
pub struct ImplFile {
    pub module: Option<String>,
    pub declarations: Vec<ImplDecl>,
}

/// A top-level implementation declaration.
#[derive(Debug, Clone)]
pub enum ImplDecl {
    Fn(ImplFn),
    Type(ImplType),
}

/// Annotation linking impl to intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentAnnotation {
    /// `@intent("name")`
    Intent(String),
    /// `@no_intent`
    NoIntent,
}

/// `fn name @intent("name")`
#[derive(Debug, Clone)]
pub struct ImplFn {
    pub name: String,
    pub annotation: IntentAnnotation,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub effects: Vec<Effect>,
    /// The actual effects found in the function body (computed by effect analysis).
    /// For now, we mock this — in a real compiler, this comes from analyzing the body.
    pub actual_effects: Vec<Effect>,
    /// Error variants actually produced in the body (computed by analysis).
    /// For now, mocked.
    pub actual_errors: Vec<String>,
}

/// `type Name` — struct or enum
#[derive(Debug, Clone)]
pub struct ImplType {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub kind: ImplTypeKind,
}

#[derive(Debug, Clone)]
pub enum ImplTypeKind {
    Struct(Vec<ImplField>),
    Enum(Vec<ImplVariant>),
}

#[derive(Debug, Clone)]
pub struct ImplField {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct ImplVariant {
    pub name: String,
    pub payload: Option<Type>,
}
