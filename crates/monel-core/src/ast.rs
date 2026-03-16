/// Shared AST types used by both intent and implementation layers.

/// A type expression — `Int`, `Stack<T>`, `Result<Session, AuthError>`, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Simple named type: `Int`, `String`, `Bool`
    Named(String),
    /// Generic type: `Stack<T>`, `Result<T, E>`, `Vec<Int>`
    Generic(String, Vec<Type>),
    /// Tuple type: `(Int, String)`
    Tuple(Vec<Type>),
    /// Function type: `Fn(Int) -> Bool`
    Func(Vec<Type>, Box<Type>),
    /// Unit type
    Unit,
    /// Type parameter (unresolved generic): `T`, `E`
    Param(String),
    /// Mutable reference: `mut Self`
    Mut(Box<Type>),
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// An effect like `Db.read`, `Fs.write`, `pure`, `unsafe`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Effect {
    pub category: String,
    pub operation: String,
}

impl Effect {
    pub fn new(category: &str, operation: &str) -> Self {
        Self {
            category: category.to_string(),
            operation: operation.to_string(),
        }
    }

    pub fn pure() -> Self {
        Self::new("pure", "pure")
    }

    pub fn is_pure(&self) -> bool {
        self.category == "pure"
    }
}

/// A type parameter with optional bounds: `T`, `T: Display + Clone`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<String>,
}

impl TypeParam {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bounds: vec![],
        }
    }

    pub fn with_bounds(name: &str, bounds: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            bounds: bounds.iter().map(|s| s.to_string()).collect(),
        }
    }
}
