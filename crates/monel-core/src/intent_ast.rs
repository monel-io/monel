/// AST for `.mn.intent` files.

use crate::ast::{Effect, Param, Type, TypeParam};

/// A complete intent file.
#[derive(Debug, Clone)]
pub struct IntentFile {
    pub module: String,
    pub declarations: Vec<IntentDecl>,
}

/// A top-level intent declaration.
#[derive(Debug, Clone)]
pub enum IntentDecl {
    Fn(IntentFn),
    Type(IntentType),
    StateMachine(IntentStateMachine),
    Layout(IntentLayout),
}

/// `intent fn name(...) -> ReturnType`
#[derive(Debug, Clone)]
pub struct IntentFn {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub effects: Vec<Effect>,
    pub is_strict: bool,
    // Lightweight tier
    pub does: Option<String>,
    pub fails: Vec<String>,
    pub edge_cases: Vec<String>,
    // Strict tier
    pub requires: Vec<String>,
    pub ensures: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub panics_never: bool,
}

/// `intent type Name`
#[derive(Debug, Clone)]
pub struct IntentType {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub does: Option<String>,
    pub variants: Vec<IntentVariant>,
    pub fields: Vec<IntentField>,
    pub invariants: Vec<String>,
}

/// An enum variant in intent: `Overflow: "push attempted on a full stack"`
#[derive(Debug, Clone)]
pub struct IntentVariant {
    pub name: String,
    pub description: String,
    pub payload: Option<Type>,
}

/// A struct field in intent: `port: Int`
#[derive(Debug, Clone)]
pub struct IntentField {
    pub name: String,
    pub ty: Type,
}

/// `intent state_machine Name`
#[derive(Debug, Clone)]
pub struct IntentStateMachine {
    pub name: String,
    pub does: Option<String>,
    pub states: Vec<String>,
    pub initial: String,
    pub terminal: Vec<String>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub trigger: String,
}

/// `intent layout Name` (placeholder)
#[derive(Debug, Clone)]
pub struct IntentLayout {
    pub name: String,
    pub does: Option<String>,
}
