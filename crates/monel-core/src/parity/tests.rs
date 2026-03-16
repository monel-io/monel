use super::*;
use crate::ast::*;
use crate::impl_ast::*;
use crate::intent_ast::*;

// ─── Helpers ───────────────────────────────────────────────────

fn t(name: &str) -> Type {
    Type::Named(name.to_string())
}

fn tp(name: &str) -> Type {
    Type::Param(name.to_string())
}

fn generic(name: &str, args: Vec<Type>) -> Type {
    Type::Generic(name.to_string(), args)
}

fn mut_ty(inner: Type) -> Type {
    Type::Mut(Box::new(inner))
}

fn param(name: &str, ty: Type) -> Param {
    Param {
        name: name.to_string(),
        ty,
    }
}

fn eff(cat: &str, op: &str) -> Effect {
    Effect::new(cat, op)
}

fn tparam(name: &str) -> TypeParam {
    TypeParam::new(name)
}

// ─── Stack example ─────────────────────────────────────────────

fn stack_intent() -> IntentFile {
    IntentFile {
        module: "stack".to_string(),
        declarations: vec![
            // intent type Stack<T>
            IntentDecl::Type(IntentType {
                name: "Stack".to_string(),
                type_params: vec![tparam("T")],
                does: Some("a bounded, generic stack using a contiguous buffer".to_string()),
                variants: vec![],
                fields: vec![],
                invariants: vec!["self.len <= self.capacity".to_string()],
            }),
            // intent type StackError
            IntentDecl::Type(IntentType {
                name: "StackError".to_string(),
                type_params: vec![],
                does: None,
                variants: vec![
                    IntentVariant {
                        name: "Overflow".to_string(),
                        description: "push attempted on a full stack".to_string(),
                        payload: None,
                    },
                    IntentVariant {
                        name: "Underflow".to_string(),
                        description: "pop attempted on an empty stack".to_string(),
                        payload: None,
                    },
                ],
                fields: vec![],
                invariants: vec![],
            }),
            // intent fn new(capacity: Int) -> Stack<T> @strict
            IntentDecl::Fn(IntentFn {
                name: "new".to_string(),
                type_params: vec![],
                params: vec![param("capacity", t("Int"))],
                return_type: generic("Stack", vec![tp("T")]),
                effects: vec![Effect::pure()],
                is_strict: true,
                does: None,
                fails: vec![],
                edge_cases: vec![],
                requires: vec!["capacity > 0".to_string()],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            // intent fn push(self: mut Stack<T>, val: T) -> Result<(), StackError> @strict
            IntentDecl::Fn(IntentFn {
                name: "push".to_string(),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(generic("Stack", vec![tp("T")]))),
                    param("val", tp("T")),
                ],
                return_type: generic("Result", vec![Type::Unit, t("StackError")]),
                effects: vec![Effect::pure()],
                is_strict: true,
                does: None,
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![("Overflow".to_string(), "stack is at capacity".to_string())],
                panics_never: false,
            }),
            // intent fn pop(self: mut Stack<T>) -> Result<T, StackError> @strict
            IntentDecl::Fn(IntentFn {
                name: "pop".to_string(),
                type_params: vec![],
                params: vec![param("self", mut_ty(generic("Stack", vec![tp("T")])))],
                return_type: generic("Result", vec![tp("T"), t("StackError")]),
                effects: vec![Effect::pure()],
                is_strict: true,
                does: None,
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![("Underflow".to_string(), "stack is empty".to_string())],
                panics_never: false,
            }),
            // intent fn peek(self: Stack<T>) -> Option<T>
            IntentDecl::Fn(IntentFn {
                name: "peek".to_string(),
                type_params: vec![],
                params: vec![param("self", generic("Stack", vec![tp("T")]))],
                return_type: generic("Option", vec![tp("T")]),
                effects: vec![Effect::pure()],
                is_strict: false,
                does: Some("returns the top element without removing it".to_string()),
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            // intent fn is_empty(self: Stack<T>) -> Bool
            IntentDecl::Fn(IntentFn {
                name: "is_empty".to_string(),
                type_params: vec![],
                params: vec![param("self", generic("Stack", vec![tp("T")]))],
                return_type: t("Bool"),
                effects: vec![Effect::pure()],
                is_strict: false,
                does: Some("returns true if the stack contains no elements".to_string()),
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
        ],
    }
}

fn stack_impl() -> ImplFile {
    ImplFile {
        module: Some("stack".to_string()),
        declarations: vec![
            // type Stack<T> — struct with data, len, capacity
            ImplDecl::Type(ImplType {
                name: "Stack".to_string(),
                type_params: vec![tparam("T")],
                kind: ImplTypeKind::Struct(vec![
                    ImplField { name: "data".to_string(), ty: generic("Array", vec![tp("T")]) },
                    ImplField { name: "len".to_string(), ty: t("Int") },
                    ImplField { name: "capacity".to_string(), ty: t("Int") },
                ]),
            }),
            // type StackError — enum
            ImplDecl::Type(ImplType {
                name: "StackError".to_string(),
                type_params: vec![],
                kind: ImplTypeKind::Enum(vec![
                    ImplVariant { name: "Overflow".to_string(), payload: None },
                    ImplVariant { name: "Underflow".to_string(), payload: None },
                ]),
            }),
            // fn new @intent("new")
            ImplDecl::Fn(ImplFn {
                name: "new".to_string(),
                annotation: IntentAnnotation::Intent("new".to_string()),
                type_params: vec![],
                params: vec![param("capacity", t("Int"))],
                return_type: generic("Stack", vec![tp("T")]),
                effects: vec![Effect::pure()],
                actual_effects: vec![], // pure — no effects in body
                actual_errors: vec![],
            }),
            // fn push @intent("push")
            ImplDecl::Fn(ImplFn {
                name: "push".to_string(),
                annotation: IntentAnnotation::Intent("push".to_string()),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(generic("Stack", vec![tp("T")]))),
                    param("val", tp("T")),
                ],
                return_type: generic("Result", vec![Type::Unit, t("StackError")]),
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec!["Overflow".to_string()],
            }),
            // fn pop @intent("pop")
            ImplDecl::Fn(ImplFn {
                name: "pop".to_string(),
                annotation: IntentAnnotation::Intent("pop".to_string()),
                type_params: vec![],
                params: vec![param("self", mut_ty(generic("Stack", vec![tp("T")])))],
                return_type: generic("Result", vec![tp("T"), t("StackError")]),
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec!["Underflow".to_string()],
            }),
            // fn peek @intent("peek")
            ImplDecl::Fn(ImplFn {
                name: "peek".to_string(),
                annotation: IntentAnnotation::Intent("peek".to_string()),
                type_params: vec![],
                params: vec![param("self", generic("Stack", vec![tp("T")]))],
                return_type: generic("Option", vec![tp("T")]),
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
            // fn is_empty @intent("is_empty")
            ImplDecl::Fn(ImplFn {
                name: "is_empty".to_string(),
                annotation: IntentAnnotation::Intent("is_empty".to_string()),
                type_params: vec![],
                params: vec![param("self", generic("Stack", vec![tp("T")]))],
                return_type: t("Bool"),
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
            // fn advance_cursor @no_intent — should be ignored
            ImplDecl::Fn(ImplFn {
                name: "internal_helper".to_string(),
                annotation: IntentAnnotation::NoIntent,
                type_params: vec![],
                params: vec![],
                return_type: Type::Unit,
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
        ],
    }
}

#[test]
fn stack_parity_passes() {
    let report = check_parity(&stack_intent(), &stack_impl());
    for e in &report.errors {
        eprintln!("  {}", e);
    }
    assert!(report.is_ok(), "stack parity should pass — errors: {:?}", report.errors);
    assert_eq!(report.matched_fns.len(), 5);
    assert_eq!(report.matched_types.len(), 2);
}

// ─── HTTP Handler example ──────────────────────────────────────

fn http_intent() -> IntentFile {
    IntentFile {
        module: "http_handler".to_string(),
        declarations: vec![
            // intent type Config
            IntentDecl::Type(IntentType {
                name: "Config".to_string(),
                type_params: vec![],
                does: Some("server configuration with sensible defaults".to_string()),
                variants: vec![],
                fields: vec![
                    IntentField { name: "port".to_string(), ty: t("Int") },
                    IntentField { name: "host".to_string(), ty: t("String") },
                    IntentField {
                        name: "tls".to_string(),
                        ty: generic("Option", vec![t("TlsConfig")]),
                    },
                    IntentField { name: "max_connections".to_string(), ty: t("Int") },
                    IntentField { name: "request_timeout".to_string(), ty: t("Duration") },
                ],
                invariants: vec![],
            }),
            // intent type ServerError
            IntentDecl::Type(IntentType {
                name: "ServerError".to_string(),
                type_params: vec![],
                does: None,
                variants: vec![
                    IntentVariant { name: "PortInUse".to_string(), description: "".to_string(), payload: None },
                    IntentVariant { name: "PermissionDenied".to_string(), description: "".to_string(), payload: None },
                    IntentVariant {
                        name: "TlsError".to_string(),
                        description: "".to_string(),
                        payload: Some(t("String")),
                    },
                ],
                fields: vec![],
                invariants: vec![],
            }),
            // intent type AuthError
            IntentDecl::Type(IntentType {
                name: "AuthError".to_string(),
                type_params: vec![],
                does: None,
                variants: vec![
                    IntentVariant { name: "InvalidCreds".to_string(), description: "".to_string(), payload: None },
                    IntentVariant { name: "Locked".to_string(), description: "".to_string(), payload: None },
                    IntentVariant { name: "Expired".to_string(), description: "".to_string(), payload: None },
                ],
                fields: vec![],
                invariants: vec![],
            }),
            // intent fn serve
            IntentDecl::Fn(IntentFn {
                name: "serve".to_string(),
                type_params: vec![],
                params: vec![
                    param("config", t("Config")),
                    param("handler", t("Handler")),
                ],
                return_type: generic("Result", vec![t("Server"), t("ServerError")]),
                effects: vec![eff("Net", "listen"), eff("Log", "write")],
                is_strict: false,
                does: Some("starts an HTTP server".to_string()),
                fails: vec!["PortInUse".to_string(), "PermissionDenied".to_string(), "TlsError".to_string()],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            // intent fn authenticate @strict
            IntentDecl::Fn(IntentFn {
                name: "authenticate".to_string(),
                type_params: vec![],
                params: vec![param("creds", t("Credentials"))],
                return_type: generic("Result", vec![t("Session"), t("AuthError")]),
                effects: vec![
                    eff("Db", "read"),
                    eff("Db", "write"),
                    eff("Crypto", "verify"),
                    eff("Log", "write"),
                ],
                is_strict: true,
                does: None,
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![
                    ("InvalidCreds".to_string(), "invalid username or password".to_string()),
                    ("Locked".to_string(), "account locked".to_string()),
                    ("Expired".to_string(), "session has expired".to_string()),
                ],
                panics_never: true,
            }),
            // intent fn handle_request
            IntentDecl::Fn(IntentFn {
                name: "handle_request".to_string(),
                type_params: vec![],
                params: vec![param("req", t("Request"))],
                return_type: t("Response"),
                effects: vec![
                    eff("Db", "read"),
                    eff("Db", "write"),
                    eff("Log", "write"),
                    eff("Http", "send"),
                ],
                is_strict: false,
                does: Some("routes and processes an incoming HTTP request".to_string()),
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
        ],
    }
}

fn http_impl() -> ImplFile {
    ImplFile {
        module: Some("http_handler".to_string()),
        declarations: vec![
            ImplDecl::Type(ImplType {
                name: "Config".to_string(),
                type_params: vec![],
                kind: ImplTypeKind::Struct(vec![
                    ImplField { name: "port".to_string(), ty: t("Int") },
                    ImplField { name: "host".to_string(), ty: t("String") },
                    ImplField { name: "tls".to_string(), ty: generic("Option", vec![t("TlsConfig")]) },
                    ImplField { name: "max_connections".to_string(), ty: t("Int") },
                    ImplField { name: "request_timeout".to_string(), ty: t("Duration") },
                ]),
            }),
            ImplDecl::Type(ImplType {
                name: "ServerError".to_string(),
                type_params: vec![],
                kind: ImplTypeKind::Enum(vec![
                    ImplVariant { name: "PortInUse".to_string(), payload: None },
                    ImplVariant { name: "PermissionDenied".to_string(), payload: None },
                    ImplVariant { name: "TlsError".to_string(), payload: Some(t("String")) },
                ]),
            }),
            ImplDecl::Type(ImplType {
                name: "AuthError".to_string(),
                type_params: vec![],
                kind: ImplTypeKind::Enum(vec![
                    ImplVariant { name: "InvalidCreds".to_string(), payload: None },
                    ImplVariant { name: "Locked".to_string(), payload: None },
                    ImplVariant { name: "Expired".to_string(), payload: None },
                ]),
            }),
            ImplDecl::Fn(ImplFn {
                name: "serve".to_string(),
                annotation: IntentAnnotation::Intent("serve".to_string()),
                type_params: vec![],
                params: vec![
                    param("config", t("Config")),
                    param("handler", t("Handler")),
                ],
                return_type: generic("Result", vec![t("Server"), t("ServerError")]),
                effects: vec![eff("Net", "listen"), eff("Log", "write")],
                actual_effects: vec![eff("Net", "listen"), eff("Log", "write")],
                actual_errors: vec![
                    "PortInUse".to_string(),
                    "PermissionDenied".to_string(),
                    "TlsError".to_string(),
                ],
            }),
            ImplDecl::Fn(ImplFn {
                name: "authenticate".to_string(),
                annotation: IntentAnnotation::Intent("authenticate".to_string()),
                type_params: vec![],
                params: vec![param("creds", t("Credentials"))],
                return_type: generic("Result", vec![t("Session"), t("AuthError")]),
                effects: vec![
                    eff("Db", "read"),
                    eff("Db", "write"),
                    eff("Crypto", "verify"),
                    eff("Log", "write"),
                ],
                actual_effects: vec![
                    eff("Db", "read"),
                    eff("Db", "write"),
                    eff("Crypto", "verify"),
                    eff("Log", "write"),
                ],
                actual_errors: vec![
                    "InvalidCreds".to_string(),
                    "Locked".to_string(),
                    "Expired".to_string(),
                ],
            }),
            ImplDecl::Fn(ImplFn {
                name: "handle_request".to_string(),
                annotation: IntentAnnotation::Intent("handle_request".to_string()),
                type_params: vec![],
                params: vec![param("req", t("Request"))],
                return_type: t("Response"),
                effects: vec![
                    eff("Db", "read"),
                    eff("Db", "write"),
                    eff("Log", "write"),
                    eff("Http", "send"),
                ],
                actual_effects: vec![
                    eff("Db", "read"),
                    eff("Db", "write"),
                    eff("Log", "write"),
                    eff("Http", "send"),
                ],
                actual_errors: vec![],
            }),
        ],
    }
}

#[test]
fn http_handler_parity_passes() {
    let report = check_parity(&http_intent(), &http_impl());
    for e in &report.errors {
        eprintln!("  {}", e);
    }
    assert!(report.is_ok(), "http_handler parity should pass — errors: {:?}", report.errors);
    assert_eq!(report.matched_fns.len(), 3);
    assert_eq!(report.matched_types.len(), 3);
}

// ─── Terminal example ──────────────────────────────────────────

fn terminal_intent() -> IntentFile {
    IntentFile {
        module: "terminal".to_string(),
        declarations: vec![
            IntentDecl::Type(IntentType {
                name: "TerminalError".to_string(),
                type_params: vec![],
                does: None,
                variants: vec![
                    IntentVariant { name: "PtyFailed".to_string(), description: "".to_string(), payload: None },
                    IntentVariant {
                        name: "IoError".to_string(),
                        description: "".to_string(),
                        payload: Some(t("String")),
                    },
                    IntentVariant { name: "InvalidSize".to_string(), description: "".to_string(), payload: None },
                ],
                fields: vec![],
                invariants: vec![],
            }),
            // intent fn init — with unsafe + Fs effects
            IntentDecl::Fn(IntentFn {
                name: "init".to_string(),
                type_params: vec![],
                params: vec![param("config", t("TerminalConfig"))],
                return_type: generic("Result", vec![t("Terminal"), t("TerminalError")]),
                effects: vec![eff("unsafe", "unsafe"), eff("Fs", "read"), eff("Fs", "write")],
                is_strict: true,
                does: None,
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![
                    ("PtyFailed".to_string(), "PTY allocation failed".to_string()),
                    ("InvalidSize".to_string(), "rows or cols must be positive".to_string()),
                ],
                panics_never: true,
            }),
            // intent fn write_input — lightweight with fails:
            IntentDecl::Fn(IntentFn {
                name: "write_input".to_string(),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(t("Terminal"))),
                    param("data", generic("Array", vec![t("Byte")])),
                ],
                return_type: generic("Result", vec![Type::Unit, t("TerminalError")]),
                effects: vec![eff("unsafe", "unsafe"), eff("Fs", "write")],
                is_strict: false,
                does: Some("writes user input bytes to the PTY".to_string()),
                fails: vec!["IoError".to_string()],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            // intent fn process_output — pure, no errors
            IntentDecl::Fn(IntentFn {
                name: "process_output".to_string(),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(t("Terminal"))),
                    param("data", generic("Array", vec![t("Byte")])),
                ],
                return_type: Type::Unit,
                effects: vec![Effect::pure()],
                is_strict: false,
                does: Some("parses ANSI escape sequences and updates the character grid".to_string()),
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            // intent fn resize — unsafe + Signal
            IntentDecl::Fn(IntentFn {
                name: "resize".to_string(),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(t("Terminal"))),
                    param("rows", t("Int")),
                    param("cols", t("Int")),
                ],
                return_type: generic("Result", vec![Type::Unit, t("TerminalError")]),
                effects: vec![eff("unsafe", "unsafe"), eff("Signal", "signal")],
                is_strict: true,
                does: None,
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![("IoError".to_string(), "failed to send SIGWINCH".to_string())],
                panics_never: false,
            }),
        ],
    }
}

fn terminal_impl() -> ImplFile {
    ImplFile {
        module: Some("terminal".to_string()),
        declarations: vec![
            ImplDecl::Type(ImplType {
                name: "TerminalError".to_string(),
                type_params: vec![],
                kind: ImplTypeKind::Enum(vec![
                    ImplVariant { name: "PtyFailed".to_string(), payload: None },
                    ImplVariant { name: "IoError".to_string(), payload: Some(t("String")) },
                    ImplVariant { name: "InvalidSize".to_string(), payload: None },
                ]),
            }),
            ImplDecl::Fn(ImplFn {
                name: "init".to_string(),
                annotation: IntentAnnotation::Intent("init".to_string()),
                type_params: vec![],
                params: vec![param("config", t("TerminalConfig"))],
                return_type: generic("Result", vec![t("Terminal"), t("TerminalError")]),
                effects: vec![eff("unsafe", "unsafe"), eff("Fs", "read"), eff("Fs", "write")],
                actual_effects: vec![eff("unsafe", "unsafe"), eff("Fs", "read"), eff("Fs", "write")],
                actual_errors: vec!["PtyFailed".to_string(), "InvalidSize".to_string()],
            }),
            ImplDecl::Fn(ImplFn {
                name: "write_input".to_string(),
                annotation: IntentAnnotation::Intent("write_input".to_string()),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(t("Terminal"))),
                    param("data", generic("Array", vec![t("Byte")])),
                ],
                return_type: generic("Result", vec![Type::Unit, t("TerminalError")]),
                effects: vec![eff("unsafe", "unsafe"), eff("Fs", "write")],
                actual_effects: vec![eff("unsafe", "unsafe"), eff("Fs", "write")],
                actual_errors: vec!["IoError".to_string()],
            }),
            ImplDecl::Fn(ImplFn {
                name: "process_output".to_string(),
                annotation: IntentAnnotation::Intent("process_output".to_string()),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(t("Terminal"))),
                    param("data", generic("Array", vec![t("Byte")])),
                ],
                return_type: Type::Unit,
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
            ImplDecl::Fn(ImplFn {
                name: "resize".to_string(),
                annotation: IntentAnnotation::Intent("resize".to_string()),
                type_params: vec![],
                params: vec![
                    param("self", mut_ty(t("Terminal"))),
                    param("rows", t("Int")),
                    param("cols", t("Int")),
                ],
                return_type: generic("Result", vec![Type::Unit, t("TerminalError")]),
                effects: vec![eff("unsafe", "unsafe"), eff("Signal", "signal")],
                actual_effects: vec![eff("unsafe", "unsafe"), eff("Signal", "signal")],
                actual_errors: vec!["IoError".to_string()],
            }),
            // @no_intent helpers
            ImplDecl::Fn(ImplFn {
                name: "advance_cursor".to_string(),
                annotation: IntentAnnotation::NoIntent,
                type_params: vec![],
                params: vec![param("self", mut_ty(t("Terminal")))],
                return_type: Type::Unit,
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
            ImplDecl::Fn(ImplFn {
                name: "scroll_up".to_string(),
                annotation: IntentAnnotation::NoIntent,
                type_params: vec![],
                params: vec![param("self", mut_ty(t("Terminal")))],
                return_type: Type::Unit,
                effects: vec![Effect::pure()],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
        ],
    }
}

#[test]
fn terminal_parity_passes() {
    let report = check_parity(&terminal_intent(), &terminal_impl());
    for e in &report.errors {
        eprintln!("  {}", e);
    }
    assert!(report.is_ok(), "terminal parity should pass — errors: {:?}", report.errors);
    assert_eq!(report.matched_fns.len(), 4);
    assert_eq!(report.matched_types.len(), 1);
}

// ─── Failure cases ─────────────────────────────────────────────
// These test that parity CORRECTLY catches mismatches.

#[test]
fn missing_impl_is_error() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Fn(IntentFn {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![param("x", t("Int"))],
            return_type: t("Int"),
            effects: vec![Effect::pure()],
            is_strict: false,
            does: Some("does something".to_string()),
            fails: vec![],
            edge_cases: vec![],
            requires: vec![],
            ensures: vec![],
            errors: vec![],
            panics_never: false,
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![],
    };
    let report = check_parity(&intent, &imp);
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.code == "P0001"));
}

#[test]
fn orphan_impl_is_error() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "bar".to_string(),
            annotation: IntentAnnotation::Intent("bar".to_string()),
            type_params: vec![],
            params: vec![],
            return_type: Type::Unit,
            effects: vec![Effect::pure()],
            actual_effects: vec![],
            actual_errors: vec![],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.code == "P0002"));
}

#[test]
fn no_intent_fn_is_not_orphan() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "helper".to_string(),
            annotation: IntentAnnotation::NoIntent,
            type_params: vec![],
            params: vec![],
            return_type: Type::Unit,
            effects: vec![Effect::pure()],
            actual_effects: vec![],
            actual_errors: vec![],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(report.is_ok());
}

#[test]
fn param_type_mismatch_is_error() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Fn(IntentFn {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![param("x", t("Int"))],
            return_type: t("Int"),
            effects: vec![Effect::pure()],
            is_strict: false,
            does: None,
            fails: vec![],
            edge_cases: vec![],
            requires: vec![],
            ensures: vec![],
            errors: vec![],
            panics_never: false,
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "foo".to_string(),
            annotation: IntentAnnotation::Intent("foo".to_string()),
            type_params: vec![],
            params: vec![param("x", t("String"))], // WRONG TYPE
            return_type: t("Int"),
            effects: vec![Effect::pure()],
            actual_effects: vec![],
            actual_errors: vec![],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.code == "P0103"));
}

#[test]
fn undeclared_effect_is_error() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Fn(IntentFn {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: Type::Unit,
            effects: vec![Effect::pure()],
            is_strict: false,
            does: None,
            fails: vec![],
            edge_cases: vec![],
            requires: vec![],
            ensures: vec![],
            errors: vec![],
            panics_never: false,
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "foo".to_string(),
            annotation: IntentAnnotation::Intent("foo".to_string()),
            type_params: vec![],
            params: vec![],
            return_type: Type::Unit,
            effects: vec![Effect::pure()],
            actual_effects: vec![eff("Db", "write")], // UNDECLARED EFFECT
            actual_errors: vec![],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.code == "P0201"));
}

#[test]
fn effect_implication_allows_db_read_under_db_write() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Fn(IntentFn {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: Type::Unit,
            effects: vec![eff("Db", "write")], // declares Db.write
            is_strict: false,
            does: None,
            fails: vec![],
            edge_cases: vec![],
            requires: vec![],
            ensures: vec![],
            errors: vec![],
            panics_never: false,
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "foo".to_string(),
            annotation: IntentAnnotation::Intent("foo".to_string()),
            type_params: vec![],
            params: vec![],
            return_type: Type::Unit,
            effects: vec![eff("Db", "write")],
            actual_effects: vec![eff("Db", "read"), eff("Db", "write")], // uses both
            actual_errors: vec![],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(report.is_ok(), "Db.write should imply Db.read — errors: {:?}", report.errors);
}

#[test]
fn undeclared_error_variant_is_error() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Fn(IntentFn {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: generic("Result", vec![Type::Unit, t("MyError")]),
            effects: vec![Effect::pure()],
            is_strict: false,
            does: None,
            fails: vec!["NotFound".to_string()],
            edge_cases: vec![],
            requires: vec![],
            ensures: vec![],
            errors: vec![],
            panics_never: false,
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "foo".to_string(),
            annotation: IntentAnnotation::Intent("foo".to_string()),
            type_params: vec![],
            params: vec![],
            return_type: generic("Result", vec![Type::Unit, t("MyError")]),
            effects: vec![Effect::pure()],
            actual_effects: vec![],
            actual_errors: vec![
                "NotFound".to_string(),
                "Unauthorized".to_string(), // NOT DECLARED
            ],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.code == "P0302"));
}

#[test]
fn enum_variant_mismatch_is_error() {
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Type(IntentType {
            name: "MyError".to_string(),
            type_params: vec![],
            does: None,
            variants: vec![
                IntentVariant { name: "A".to_string(), description: "".to_string(), payload: None },
                IntentVariant { name: "B".to_string(), description: "".to_string(), payload: None },
            ],
            fields: vec![],
            invariants: vec![],
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Type(ImplType {
            name: "MyError".to_string(),
            type_params: vec![],
            kind: ImplTypeKind::Enum(vec![
                ImplVariant { name: "A".to_string(), payload: None },
                ImplVariant { name: "C".to_string(), payload: None }, // B missing, C extra
            ]),
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(!report.is_ok());
    assert!(report.errors.iter().any(|e| e.code == "P0401")); // B missing
    assert!(report.errors.iter().any(|e| e.code == "P0402")); // C extra
}

#[test]
fn generic_type_matching_works() {
    // intent fn foo<T>(x: Vec<T>) -> Option<T>
    // impl   fn foo<T>(x: Vec<T>) -> Option<T>
    let intent = IntentFile {
        module: "test".to_string(),
        declarations: vec![IntentDecl::Fn(IntentFn {
            name: "foo".to_string(),
            type_params: vec![tparam("T")],
            params: vec![param("x", generic("Vec", vec![tp("T")]))],
            return_type: generic("Option", vec![tp("T")]),
            effects: vec![Effect::pure()],
            is_strict: false,
            does: None,
            fails: vec![],
            edge_cases: vec![],
            requires: vec![],
            ensures: vec![],
            errors: vec![],
            panics_never: false,
        })],
    };
    let imp = ImplFile {
        module: Some("test".to_string()),
        declarations: vec![ImplDecl::Fn(ImplFn {
            name: "foo".to_string(),
            annotation: IntentAnnotation::Intent("foo".to_string()),
            type_params: vec![tparam("T")],
            params: vec![param("x", generic("Vec", vec![tp("T")]))],
            return_type: generic("Option", vec![tp("T")]),
            effects: vec![Effect::pure()],
            actual_effects: vec![],
            actual_errors: vec![],
        })],
    };
    let report = check_parity(&intent, &imp);
    assert!(report.is_ok(), "generic matching should work — errors: {:?}", report.errors);
}
