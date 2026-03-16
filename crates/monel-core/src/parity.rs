/// Structural parity checker (Stage 2).
///
/// Verifies one-to-one correspondence between intent declarations
/// and implementation definitions.

use std::collections::{HashMap, HashSet};

use crate::ast::{Effect, Type, TypeParam};
use crate::impl_ast::{ImplFile, ImplFn, ImplType, ImplTypeKind, IntentAnnotation};
use crate::intent_ast::{IntentDecl, IntentFile, IntentFn, IntentType};

/// A parity error with error code and message.
#[derive(Debug, Clone)]
pub struct ParityError {
    pub code: &'static str,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for ParityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{}[{}]: {}", sev, self.code, self.message)
    }
}

/// Result of parity checking a module.
#[derive(Debug)]
pub struct ParityReport {
    pub module: String,
    pub matched_fns: Vec<FnMatch>,
    pub matched_types: Vec<TypeMatch>,
    pub errors: Vec<ParityError>,
}

#[derive(Debug)]
pub struct FnMatch {
    pub intent_name: String,
    pub impl_name: String,
}

#[derive(Debug)]
pub struct TypeMatch {
    pub intent_name: String,
    pub impl_name: String,
}

impl ParityReport {
    pub fn is_ok(&self) -> bool {
        self.errors.iter().all(|e| e.severity != Severity::Error)
    }
}

/// Check structural parity between an intent file and an implementation file.
pub fn check_parity(intent: &IntentFile, imp: &ImplFile) -> ParityReport {
    let mut report = ParityReport {
        module: intent.module.clone(),
        matched_fns: vec![],
        matched_types: vec![],
        errors: vec![],
    };

    // Collect intent declarations by kind
    let intent_fns: HashMap<&str, &IntentFn> = intent
        .declarations
        .iter()
        .filter_map(|d| match d {
            IntentDecl::Fn(f) => Some((f.name.as_str(), f)),
            _ => None,
        })
        .collect();

    let intent_types: HashMap<&str, &IntentType> = intent
        .declarations
        .iter()
        .filter_map(|d| match d {
            IntentDecl::Type(t) => Some((t.name.as_str(), t)),
            _ => None,
        })
        .collect();

    // Collect impl declarations
    let mut impl_fns: HashMap<&str, &ImplFn> = HashMap::new();
    let mut impl_types: HashMap<&str, &ImplType> = HashMap::new();
    let mut no_intent_fns: HashSet<&str> = HashSet::new();

    for decl in &imp.declarations {
        match decl {
            crate::impl_ast::ImplDecl::Fn(f) => {
                match &f.annotation {
                    IntentAnnotation::Intent(name) => {
                        impl_fns.insert(name.as_str(), f);
                    }
                    IntentAnnotation::NoIntent => {
                        no_intent_fns.insert(f.name.as_str());
                    }
                }
            }
            crate::impl_ast::ImplDecl::Type(t) => {
                impl_types.insert(t.name.as_str(), t);
            }
        }
    }

    // --- Check functions ---

    // Every intent fn must have a matching impl fn
    for (name, intent_fn) in &intent_fns {
        match impl_fns.get(name) {
            None => {
                report.errors.push(ParityError {
                    code: "P0001",
                    message: format!("missing implementation for intent fn `{}`", name),
                    severity: Severity::Error,
                });
            }
            Some(impl_fn) => {
                check_fn_parity(intent_fn, impl_fn, &mut report);
                report.matched_fns.push(FnMatch {
                    intent_name: name.to_string(),
                    impl_name: impl_fn.name.clone(),
                });
            }
        }
    }

    // Every impl fn with @intent must have a matching intent fn
    for (name, _) in &impl_fns {
        if !intent_fns.contains_key(name) {
            report.errors.push(ParityError {
                code: "P0002",
                message: format!(
                    "no intent found for fn `{}` — add intent or mark `@no_intent`",
                    name
                ),
                severity: Severity::Error,
            });
        }
    }

    // --- Check types ---

    for (name, intent_type) in &intent_types {
        match impl_types.get(name) {
            None => {
                report.errors.push(ParityError {
                    code: "P0010",
                    message: format!("missing implementation for intent type `{}`", name),
                    severity: Severity::Error,
                });
            }
            Some(impl_type) => {
                check_type_parity(intent_type, impl_type, &mut report);
                report.matched_types.push(TypeMatch {
                    intent_name: name.to_string(),
                    impl_name: impl_type.name.clone(),
                });
            }
        }
    }

    report
}

/// Check that a function's signature matches between intent and impl.
fn check_fn_parity(intent: &IntentFn, imp: &ImplFn, report: &mut ParityReport) {
    let name = &intent.name;

    // Check type parameter count and names
    check_type_params(&intent.type_params, &imp.type_params, name, report);

    // Build a type parameter mapping for generic unification
    let type_map: HashMap<&str, &str> = intent
        .type_params
        .iter()
        .zip(imp.type_params.iter())
        .map(|(i, m)| (i.name.as_str(), m.name.as_str()))
        .collect();

    // Check parameter count
    if intent.params.len() != imp.params.len() {
        report.errors.push(ParityError {
            code: "P0101",
            message: format!(
                "fn `{}`: intent has {} params, impl has {}",
                name,
                intent.params.len(),
                imp.params.len()
            ),
            severity: Severity::Error,
        });
        return; // Can't check individual params if counts differ
    }

    // Check each parameter
    for (i, (ip, mp)) in intent.params.iter().zip(imp.params.iter()).enumerate() {
        if ip.name != mp.name {
            report.errors.push(ParityError {
                code: "P0102",
                message: format!(
                    "fn `{}`: param {} name mismatch: intent `{}` vs impl `{}`",
                    name, i, ip.name, mp.name
                ),
                severity: Severity::Error,
            });
        }
        if !types_match(&ip.ty, &mp.ty, &type_map) {
            report.errors.push(ParityError {
                code: "P0103",
                message: format!(
                    "fn `{}`: param `{}` type mismatch: intent `{:?}` vs impl `{:?}`",
                    name, ip.name, ip.ty, mp.ty
                ),
                severity: Severity::Error,
            });
        }
    }

    // Check return type
    if !types_match(&intent.return_type, &imp.return_type, &type_map) {
        report.errors.push(ParityError {
            code: "P0104",
            message: format!(
                "fn `{}`: return type mismatch: intent `{:?}` vs impl `{:?}`",
                name, intent.return_type, imp.return_type
            ),
            severity: Severity::Error,
        });
    }

    // Check effects (impl effects must be subset of declared)
    check_effects(&intent.effects, &imp.actual_effects, name, report);

    // Check error exhaustiveness
    check_error_exhaustiveness(intent, imp, report);
}

/// Check type parameters match.
fn check_type_params(
    intent: &[TypeParam],
    imp: &[TypeParam],
    fn_name: &str,
    report: &mut ParityReport,
) {
    if intent.len() != imp.len() {
        report.errors.push(ParityError {
            code: "P0105",
            message: format!(
                "fn `{}`: type param count mismatch: intent has {}, impl has {}",
                fn_name,
                intent.len(),
                imp.len()
            ),
            severity: Severity::Error,
        });
        return;
    }

    for (ip, mp) in intent.iter().zip(imp.iter()) {
        // Names must match
        if ip.name != mp.name {
            report.errors.push(ParityError {
                code: "P0106",
                message: format!(
                    "fn `{}`: type param name mismatch: intent `{}` vs impl `{}`",
                    fn_name, ip.name, mp.name
                ),
                severity: Severity::Error,
            });
        }
        // Bounds must match
        let mut ib: Vec<&str> = ip.bounds.iter().map(|s| s.as_str()).collect();
        let mut mb: Vec<&str> = mp.bounds.iter().map(|s| s.as_str()).collect();
        ib.sort();
        mb.sort();
        if ib != mb {
            report.errors.push(ParityError {
                code: "P0107",
                message: format!(
                    "fn `{}`: type param `{}` bounds mismatch: intent {:?} vs impl {:?}",
                    fn_name, ip.name, ip.bounds, mp.bounds
                ),
                severity: Severity::Error,
            });
        }
    }
}

/// Check that two types match, accounting for generic type parameter mapping.
fn types_match(intent: &Type, imp: &Type, type_map: &HashMap<&str, &str>) -> bool {
    match (intent, imp) {
        (Type::Unit, Type::Unit) => true,
        (Type::Named(a), Type::Named(b)) => a == b,
        (Type::Param(a), Type::Param(b)) => {
            // Check via type_map: intent's T maps to impl's T
            match type_map.get(a.as_str()) {
                Some(mapped) => *mapped == b.as_str(),
                None => a == b,
            }
        }
        // Param can match Named if the type_map says so
        // (this handles cases where intent uses T and impl resolves it)
        (Type::Param(a), Type::Named(b)) | (Type::Named(b), Type::Param(a)) => {
            type_map.get(a.as_str()) == Some(&b.as_str())
        }
        (Type::Generic(a_name, a_args), Type::Generic(b_name, b_args)) => {
            a_name == b_name
                && a_args.len() == b_args.len()
                && a_args
                    .iter()
                    .zip(b_args.iter())
                    .all(|(a, b)| types_match(a, b, type_map))
        }
        (Type::Tuple(a), Type::Tuple(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|(a, b)| types_match(a, b, type_map))
        }
        (Type::Func(a_params, a_ret), Type::Func(b_params, b_ret)) => {
            a_params.len() == b_params.len()
                && a_params
                    .iter()
                    .zip(b_params.iter())
                    .all(|(a, b)| types_match(a, b, type_map))
                && types_match(a_ret, b_ret, type_map)
        }
        (Type::Mut(a), Type::Mut(b)) => types_match(a, b, type_map),
        _ => false,
    }
}

/// Effect implication rules. Returns all effects implied by a given effect.
fn implied_effects(effect: &Effect) -> HashSet<Effect> {
    let mut implied = HashSet::new();
    implied.insert(effect.clone());

    match (effect.category.as_str(), effect.operation.as_str()) {
        ("Db", "write") => {
            implied.insert(Effect::new("Db", "read"));
        }
        ("Fs", "write") => {
            implied.insert(Effect::new("Fs", "read"));
        }
        ("Http", "send") => {
            implied.insert(Effect::new("Net", "connect"));
            implied.insert(Effect::new("Net", "send"));
        }
        ("Http", "receive") => {
            implied.insert(Effect::new("Net", "receive"));
        }
        ("Db", "*") => {
            implied.insert(Effect::new("Db", "read"));
            implied.insert(Effect::new("Db", "write"));
        }
        ("Fs", "*") => {
            implied.insert(Effect::new("Fs", "read"));
            implied.insert(Effect::new("Fs", "write"));
        }
        ("Net", "*") => {
            implied.insert(Effect::new("Net", "connect"));
            implied.insert(Effect::new("Net", "listen"));
            implied.insert(Effect::new("Net", "send"));
            implied.insert(Effect::new("Net", "receive"));
        }
        ("Crypto", "*") => {
            for op in &["hash", "encrypt", "decrypt", "sign", "verify", "random"] {
                implied.insert(Effect::new("Crypto", op));
            }
        }
        ("Auth", "*") => {
            for op in &["check", "grant", "revoke", "session"] {
                implied.insert(Effect::new("Auth", op));
            }
        }
        _ => {}
    }

    implied
}

/// Expand a declared effect set to include all implied effects.
fn expand_effects(declared: &[Effect]) -> HashSet<Effect> {
    let mut expanded = HashSet::new();
    for effect in declared {
        expanded.extend(implied_effects(effect));
    }
    expanded
}

/// Check that actual effects are a subset of declared effects.
fn check_effects(
    declared: &[Effect],
    actual: &[Effect],
    fn_name: &str,
    report: &mut ParityReport,
) {
    // Pure means no effects allowed
    if declared.iter().any(|e| e.is_pure()) {
        if actual.iter().any(|e| !e.is_pure()) {
            let violations: Vec<String> = actual
                .iter()
                .filter(|e| !e.is_pure())
                .map(|e| format!("{}.{}", e.category, e.operation))
                .collect();
            report.errors.push(ParityError {
                code: "P0201",
                message: format!(
                    "fn `{}`: declared pure but body has effects: [{}]",
                    fn_name,
                    violations.join(", ")
                ),
                severity: Severity::Error,
            });
        }
        return;
    }

    let allowed = expand_effects(declared);

    for effect in actual {
        if effect.is_pure() {
            continue;
        }
        if !allowed.contains(effect) {
            report.errors.push(ParityError {
                code: "P0202",
                message: format!(
                    "fn `{}`: undeclared effect `{}.{}` — add to intent effects or remove from implementation",
                    fn_name, effect.category, effect.operation
                ),
                severity: Severity::Error,
            });
        }
    }
}

/// Check error variant exhaustiveness:
/// 1. Every declared `fails:` variant must be reachable in the implementation
/// 2. The implementation must not produce undeclared error variants
fn check_error_exhaustiveness(intent: &IntentFn, imp: &ImplFn, report: &mut ParityReport) {
    let name = &intent.name;

    // Collect declared error variants from intent
    let declared: HashSet<&str> = if !intent.errors.is_empty() {
        // @strict: errors: section
        intent.errors.iter().map(|(name, _)| name.as_str()).collect()
    } else {
        // lightweight: fails: section
        intent.fails.iter().map(|s| s.as_str()).collect()
    };

    let actual: HashSet<&str> = imp.actual_errors.iter().map(|s| s.as_str()).collect();

    // Declared but not produced → warning (unreachable error variant)
    for variant in &declared {
        if !actual.contains(variant) {
            report.errors.push(ParityError {
                code: "P0301",
                message: format!(
                    "fn `{}`: declared error variant `{}` is never produced in implementation",
                    name, variant
                ),
                severity: Severity::Warning,
            });
        }
    }

    // Produced but not declared → error (undeclared error variant)
    for variant in &actual {
        if !declared.contains(variant) {
            report.errors.push(ParityError {
                code: "P0302",
                message: format!(
                    "fn `{}`: implementation produces undeclared error variant `{}`",
                    name, variant
                ),
                severity: Severity::Error,
            });
        }
    }
}

/// Check type parity between intent type and impl type.
fn check_type_parity(intent: &IntentType, imp: &ImplType, report: &mut ParityReport) {
    let name = &intent.name;

    // Check type params
    check_type_params(&intent.type_params, &imp.type_params, name, report);

    // Check variant correspondence for enums
    if !intent.variants.is_empty() {
        match &imp.kind {
            ImplTypeKind::Enum(impl_variants) => {
                let intent_names: HashSet<&str> =
                    intent.variants.iter().map(|v| v.name.as_str()).collect();
                let impl_names: HashSet<&str> =
                    impl_variants.iter().map(|v| v.name.as_str()).collect();

                for v in &intent_names {
                    if !impl_names.contains(v) {
                        report.errors.push(ParityError {
                            code: "P0401",
                            message: format!(
                                "type `{}`: intent declares variant `{}` not found in implementation",
                                name, v
                            ),
                            severity: Severity::Error,
                        });
                    }
                }
                for v in &impl_names {
                    if !intent_names.contains(v) {
                        report.errors.push(ParityError {
                            code: "P0402",
                            message: format!(
                                "type `{}`: implementation has variant `{}` not declared in intent",
                                name, v
                            ),
                            severity: Severity::Error,
                        });
                    }
                }

                // Check payload correspondence
                for iv in &intent.variants {
                    if let Some(mv) = impl_variants.iter().find(|v| v.name == iv.name) {
                        match (&iv.payload, &mv.payload) {
                            (Some(_), None) => {
                                report.errors.push(ParityError {
                                    code: "P0403",
                                    message: format!(
                                        "type `{}`: variant `{}` has payload in intent but not in implementation",
                                        name, iv.name
                                    ),
                                    severity: Severity::Error,
                                });
                            }
                            (None, Some(_)) => {
                                report.errors.push(ParityError {
                                    code: "P0404",
                                    message: format!(
                                        "type `{}`: variant `{}` has payload in implementation but not in intent",
                                        name, iv.name
                                    ),
                                    severity: Severity::Error,
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            ImplTypeKind::Struct(_) => {
                report.errors.push(ParityError {
                    code: "P0405",
                    message: format!(
                        "type `{}`: intent declares variants (enum) but implementation is a struct",
                        name
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }

    // Check field correspondence for structs
    if !intent.fields.is_empty() {
        match &imp.kind {
            ImplTypeKind::Struct(impl_fields) => {
                let type_map: HashMap<&str, &str> = intent
                    .type_params
                    .iter()
                    .zip(imp.type_params.iter())
                    .map(|(i, m)| (i.name.as_str(), m.name.as_str()))
                    .collect();

                let intent_names: HashSet<&str> =
                    intent.fields.iter().map(|f| f.name.as_str()).collect();
                let impl_names: HashSet<&str> =
                    impl_fields.iter().map(|f| f.name.as_str()).collect();

                for f in &intent_names {
                    if !impl_names.contains(f) {
                        report.errors.push(ParityError {
                            code: "P0411",
                            message: format!(
                                "type `{}`: intent declares field `{}` not found in implementation",
                                name, f
                            ),
                            severity: Severity::Error,
                        });
                    }
                }

                // Check field types match
                for ifield in &intent.fields {
                    if let Some(mfield) = impl_fields.iter().find(|f| f.name == ifield.name) {
                        if !types_match(&ifield.ty, &mfield.ty, &type_map) {
                            report.errors.push(ParityError {
                                code: "P0412",
                                message: format!(
                                    "type `{}`: field `{}` type mismatch: intent `{:?}` vs impl `{:?}`",
                                    name, ifield.name, ifield.ty, mfield.ty
                                ),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
            ImplTypeKind::Enum(_) => {
                report.errors.push(ParityError {
                    code: "P0413",
                    message: format!(
                        "type `{}`: intent declares fields (struct) but implementation is an enum",
                        name
                    ),
                    severity: Severity::Error,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests;
