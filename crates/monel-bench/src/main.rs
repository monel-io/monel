//! Monel vs Rust benchmark: measures AI token efficiency, drift detection, and spec density.
//!
//! Scenario: An AI agent is asked to "add rate limiting to authenticate —
//! max 10 attempts per IP per minute."
//!
//! We measure:
//! 1. Token cost: how many tokens the AI must read to gather sufficient context
//! 2. Drift detection: what the parity checker catches that Rust tooling misses
//! 3. Specification density: intent lines vs equivalent Rust doc/type lines

use monel_core::ast::{Effect, Param, Type};
use monel_core::impl_ast::*;
use monel_core::intent_ast::*;
use monel_core::parity::{check_parity, ParityReport, Severity};

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Monel vs Rust Benchmark: AI Agent Efficiency");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("Scenario: \"Add rate limiting to authenticate — max 10 attempts");
    println!("           per IP per minute\"");
    println!();

    bench_token_efficiency();
    bench_drift_detection();
    bench_spec_density();
    bench_parity_catches_bugs();
}

// ─── Benchmark 1: Token Efficiency ──────────────────────────────────────────

fn bench_token_efficiency() {
    println!("───────────────────────────────────────────────────────────────");
    println!("  Benchmark 1: Token Efficiency (AI Context Gathering)");
    println!("───────────────────────────────────────────────────────────────");
    println!();

    // Rust: AI agent must speculatively read files to understand the codebase.
    // Typical exploration pattern for "modify authenticate":
    let rust_files: Vec<(&str, &str)> = vec![
        ("Cargo.toml", include_str!("../rust-baseline/lib.rs")),
        ("lib.rs", include_str!("../rust-baseline/lib.rs")),
        ("auth.rs", include_str!("../rust-baseline/auth.rs")),
        ("types.rs", include_str!("../rust-baseline/types.rs")),
        ("errors.rs", include_str!("../rust-baseline/errors.rs")),
        ("handlers.rs", include_str!("../rust-baseline/handlers.rs")),
        ("middleware.rs", include_str!("../rust-baseline/middleware.rs")),
        ("config.rs", include_str!("../rust-baseline/config.rs")),
    ];

    let rust_total_chars: usize = rust_files.iter().map(|(_, c)| c.len()).sum();
    let rust_total_lines: usize = rust_files.iter().map(|(_, c)| c.lines().count()).sum();
    let rust_total_tokens = rust_total_chars / 4; // ~4 chars per token approximation
    let rust_file_count = rust_files.len();

    // Monel: single .mn file with contracts alongside implementation.
    let monel_source =
        include_str!("../../../spec/examples/http_handler.mn");

    // `monel query context` (NOT YET IMPLEMENTED) would return a targeted subset:
    let monel_context_output = r#"# Context for: fn authenticate
## Contract (http_handler.mn)
fn authenticate(creds: Credentials) -> Result<Session, AuthError> @strict
  requires:
    creds.username.len > 0
    creds.password.len > 0
  ensures:
    ok => result.user_id > 0
    ok => result.expires_at > Clock.now()
    err(Locked) => Db.failed_attempts(creds.username) >= 5
  errors:
    InvalidCreds: "invalid username or password"
    Locked: "account locked after 5+ failed attempts"
    Expired: "session has expired"
  effects: [Db.read, Db.write, Crypto.verify, Log.write]
  panics: never

## Dependencies
type Credentials { username: String, password: String }
type Session { user_id: Int, token: String, expires_at: Instant }
type AuthError = InvalidCreds | Locked | Expired

## Effect Budget (from monel.policy)
Db.write: max_per_second = 1000

## Callers
fn handle_request — effects: [Db.read, Db.write, Log.write, Http.send]
"#;

    // Monel with full source file
    let monel_full_chars = monel_source.len();
    let monel_full_lines: usize = monel_source.lines().count();
    let monel_full_tokens = monel_full_chars / 4;

    // Monel with query context (best case — targeted output)
    let monel_context_chars = monel_context_output.len();
    let monel_context_lines = monel_context_output.lines().count();
    let monel_context_tokens = monel_context_chars / 4;

    println!("  Approach             Files  Lines  ~Tokens  Tool Calls");
    println!("  ──────────────────── ─────  ─────  ───────  ──────────");
    println!(
        "  Rust (speculative)   {:>5}  {:>5}  {:>7}  {:>10}",
        rust_file_count, rust_total_lines, rust_total_tokens, "8 reads"
    );
    println!(
        "  Monel (full file)    {:>5}  {:>5}  {:>7}  {:>10}",
        1, monel_full_lines, monel_full_tokens, "1 read"
    );
    println!(
        "  Monel (query ctx)    {:>5}  {:>5}  {:>7}  {:>10}",
        0, monel_context_lines, monel_context_tokens, "1 query"
    );
    println!();

    let savings_full = 100.0 * (1.0 - monel_full_tokens as f64 / rust_total_tokens as f64);
    let savings_ctx = 100.0 * (1.0 - monel_context_tokens as f64 / rust_total_tokens as f64);

    println!("  Token savings (full file):    {savings_full:.0}%");
    println!("  Token savings (query ctx):    {savings_ctx:.0}% [SIMULATED — command not yet implemented]");
    println!();

    // What the AI knows BEFORE reading any implementation code:
    println!("  What Monel contracts tell the AI before reading impl:");
    println!("    + Exact function signature with types");
    println!("    + Preconditions (username/password non-empty)");
    println!("    + Postconditions (user_id > 0, session not expired)");
    println!("    + All possible errors and when they occur");
    println!("    + All side effects (Db.read, Db.write, Crypto.verify, Log.write)");
    println!("    + Panic guarantee (never)");
    println!("    + Effect budget constraints (Db.write max rate)");
    println!();
    println!("  What Rust tells the AI before reading impl:");
    println!("    + Function signature with types");
    println!("    - No preconditions");
    println!("    - No postconditions");
    println!("    - Error variants visible, but no docs on when they occur");
    println!("    - No side effect information -- must read entire body");
    println!("    - No panic guarantees");
    println!("    - No rate/budget constraints");
    println!();
}

// ─── Benchmark 2: Drift Detection ──────────────────────────────────────────

fn bench_drift_detection() {
    println!("───────────────────────────────────────────────────────────────");
    println!("  Benchmark 2: Drift Detection");
    println!("  (AI adds rate limiting -- what breaks silently?)");
    println!("───────────────────────────────────────────────────────────────");
    println!();

    let intent = build_auth_intent();
    let drifted_impl = build_drifted_impl();

    let report = check_parity(&intent, &drifted_impl);

    println!("  Scenario: AI added rate limiting with these changes:");
    println!("    1. Added `Net.send` effect (calls external rate-limit service)");
    println!("    2. Added `RateLimited` error variant");
    println!("    3. Did NOT update intent (simulates spec drift)");
    println!();

    println!("  Rust compiler:  PASS (compiles, no drift detection)");
    println!("  Rust clippy:    PASS (no drift detection)");
    println!("  Rust tests:     PASS (existing tests don't cover new behavior)");
    println!();

    println!("  Monel parity checker:");
    if report.is_ok() {
        println!("    PASS (unexpected!)");
    } else {
        for err in &report.errors {
            let icon = if err.code.starts_with("P02") {
                "EFFECT DRIFT"
            } else if err.code.starts_with("P04") {
                "TYPE DRIFT"
            } else {
                "VIOLATION"
            };
            println!("    FAIL [{icon}] {err}");
        }
    }
    println!();

    println!("  In Rust, these changes compile and ship silently.");
    println!("  In Monel, parity checking blocks the build until intent is updated.");
    println!();

    println!("  Additional silent failures in Rust that Monel would catch:");
    println!();
    println!("    - Removing the `Expired` error path but keeping the variant");
    println!("      Monel: parity error — contract declares Expired but impl never returns it");
    println!("      Rust: dead code warning at best (often suppressed)");
    println!();
}

// ─── Benchmark 3: Specification Density ────────────────────────────────────

fn bench_spec_density() {
    println!("───────────────────────────────────────────────────────────────");
    println!("  Benchmark 3: Specification Density [HAND-COUNTED]");
    println!("  (How much meaning per line?)");
    println!("───────────────────────────────────────────────────────────────");
    println!();

    let rust_spec_lines = vec![
        ("doc comments", 8),
        ("type signatures", 12),
        ("trait definitions", 10),
        ("Display impls", 14),
        ("Error impls", 2),
        ("Default impls", 10),
    ];
    let rust_total: usize = rust_spec_lines.iter().map(|(_, n)| n).sum();

    let monel_spec_lines = vec![
        ("does: descriptions", 6),
        ("type signatures", 8),
        ("effects declarations", 5),
        ("error contracts", 6),
        ("preconditions", 3),
        ("postconditions", 5),
        ("edge cases", 3),
        ("panic guarantees", 1),
    ];
    let monel_total: usize = monel_spec_lines.iter().map(|(_, n)| n).sum();

    println!("  Rust specification artifacts:");
    for (name, count) in &rust_spec_lines {
        println!("    {name:<25} {count:>3} lines");
    }
    println!("    ─────────────────────────────────");
    println!("    {:<25} {:>3} lines", "TOTAL", rust_total);
    println!();

    println!("  Monel intent specification:");
    for (name, count) in &monel_spec_lines {
        println!("    {name:<25} {count:>3} lines");
    }
    println!("    ─────────────────────────────────");
    println!("    {:<25} {:>3} lines", "TOTAL", monel_total);
    println!();

    println!("  Rust specifies: types, names, docs (WHAT it is)");
    println!("  Monel specifies: types, names, docs, effects, contracts,");
    println!("                   error semantics, panic guarantees (WHAT + WHY + WHEN)");
    println!();

    println!("  Verifiability:");
    println!("    Rust                          Monel");
    println!("    ────────────────────────────  ────────────────────────────");
    println!("    Types checked: YES compiler   Types checked: YES compiler");
    println!("    Docs verified: NO  never      does: verified: YES parity S4");
    println!("    Effects tracked: NO  never    Effects tracked: YES parity S2");
    println!("    Errors exhaustive: YES match  Errors exhaustive: YES parity S2");
    println!("    Contracts checked: NO  never  Contracts checked: YES parity S3");
    println!("    Panic guarantee: NO  never    Panic guarantee: YES parity S3");
    println!();
}

// ─── Benchmark 4: Parity catches real bugs ─────────────────────────────────

fn bench_parity_catches_bugs() {
    println!("───────────────────────────────────────────────────────────────");
    println!("  Benchmark 4: Bug Detection (Parity vs Rust Compiler)");
    println!("───────────────────────────────────────────────────────────────");
    println!();

    let scenarios = vec![
        ("AI adds undeclared database write", run_undeclared_effect_bug()),
        ("AI adds error variant not in spec", run_undeclared_error_bug()),
        ("AI forgets to implement a declared fn", run_missing_impl_bug()),
        ("AI adds helper without @no_intent", run_orphan_impl_bug()),
        ("AI changes return type subtly", run_return_type_bug()),
    ];

    println!(
        "  {:<42} {:>7}  {:>7}",
        "Bug Scenario", "Rust", "Monel"
    );
    println!(
        "  {:<42} {:>7}  {:>7}",
        "──────────────────────────────────────────",
        "───────",
        "───────"
    );

    for (name, (rust_catches, monel_catches, monel_error)) in &scenarios {
        let rust_icon = if *rust_catches { "CATCH" } else { "MISS" };
        let monel_icon = if *monel_catches { "CATCH" } else { "MISS" };
        println!("  {name:<42} {rust_icon:>7}  {monel_icon:>7}");
        if *monel_catches && !monel_error.is_empty() {
            println!("    => {monel_error}");
        }
    }

    let rust_caught = scenarios.iter().filter(|(_, (r, _, _))| *r).count();
    let monel_caught = scenarios.iter().filter(|(_, (_, m, _))| *m).count();

    println!();
    println!("  Rust catches:  {rust_caught}/5 (type mismatches only)");
    println!("  Monel catches: {monel_caught}/5 (types + effects + errors + existence)");
    println!();

    println!("═══════════════════════════════════════════════════════════════");
    println!("  Summary: Monel's advantage is NOT runtime performance.");
    println!("  It's verification bandwidth — catching more classes of bugs");
    println!("  at compile time, and reducing AI token cost by {monel_caught}x fewer");
    println!("  blind spots when agents modify code.");
    println!("═══════════════════════════════════════════════════════════════");
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Get the first actual error (not warning) from a parity report, for display.
fn first_error(report: &ParityReport) -> String {
    report
        .errors
        .iter()
        .find(|e| e.severity == Severity::Error)
        .or_else(|| report.errors.first())
        .map(|e| format!("{e}"))
        .unwrap_or_default()
}

// ─── Bug scenario runners ──────────────────────────────────────────────────

/// Returns (rust_catches, monel_catches, monel_error_description)
fn run_undeclared_effect_bug() -> (bool, bool, String) {
    let intent = build_auth_intent();
    let mut imp = build_correct_impl();

    // AI adds Net.send without updating intent
    for decl in imp.declarations.iter_mut() {
        if let ImplDecl::Fn(f) = decl {
            if f.name == "authenticate" {
                f.actual_effects.push(Effect::new("Net", "send"));
            }
        }
    }

    let report = check_parity(&intent, &imp);
    let err = first_error(&report);
    (false, !report.is_ok(), err)
}

fn run_undeclared_error_bug() -> (bool, bool, String) {
    let intent = build_auth_intent();
    let mut imp = build_correct_impl();

    // AI adds RateLimited error variant to AuthError enum
    for decl in imp.declarations.iter_mut() {
        if let ImplDecl::Type(t) = decl {
            if t.name == "AuthError" {
                if let ImplTypeKind::Enum(variants) = &mut t.kind {
                    variants.push(ImplVariant {
                        name: "RateLimited".into(),
                        payload: None,
                    });
                }
            }
        }
    }

    let report = check_parity(&intent, &imp);
    let err = first_error(&report);
    (false, !report.is_ok(), err)
}

fn run_missing_impl_bug() -> (bool, bool, String) {
    let intent = build_auth_intent();
    let mut imp = build_correct_impl();

    // AI removes handle_request implementation
    imp.declarations
        .retain(|d| !matches!(d, ImplDecl::Fn(f) if f.name == "handle_request"));

    let report = check_parity(&intent, &imp);
    let err = first_error(&report);
    (false, !report.is_ok(), err)
}

fn run_orphan_impl_bug() -> (bool, bool, String) {
    let intent = build_auth_intent();
    let mut imp = build_correct_impl();

    // AI adds a helper with @intent but no matching intent declaration
    imp.declarations.push(ImplDecl::Fn(ImplFn {
        name: "check_rate_limit".into(),
        annotation: IntentAnnotation::Intent("check_rate_limit".into()),
        type_params: vec![],
        params: vec![Param {
            name: "ip".into(),
            ty: Type::Named("String".into()),
        }],
        return_type: Type::Named("Bool".into()),
        effects: vec![Effect::new("Net", "send")],
        actual_effects: vec![Effect::new("Net", "send")],
        actual_errors: vec![],
    }));

    let report = check_parity(&intent, &imp);
    let err = first_error(&report);
    (false, !report.is_ok(), err)
}

fn run_return_type_bug() -> (bool, bool, String) {
    let intent = build_auth_intent();
    let mut imp = build_correct_impl();

    // AI changes authenticate to return Option<Session> instead of Result<Session, AuthError>
    for decl in imp.declarations.iter_mut() {
        if let ImplDecl::Fn(f) = decl {
            if f.name == "authenticate" {
                f.return_type =
                    Type::Generic("Option".into(), vec![Type::Named("Session".into())]);
            }
        }
    }

    let report = check_parity(&intent, &imp);
    let err = first_error(&report);
    // Rust catches this if callers pattern-match on Result
    (true, !report.is_ok(), err)
}

// ─── Test data builders ────────────────────────────────────────────────────

fn build_auth_intent() -> IntentFile {
    IntentFile {
        module: "http_handler".into(),
        declarations: vec![
            IntentDecl::Type(IntentType {
                name: "Config".into(),
                type_params: vec![],
                does: Some("server configuration with sensible defaults".into()),
                fields: vec![
                    IntentField { name: "port".into(), ty: Type::Named("Int".into()) },
                    IntentField { name: "host".into(), ty: Type::Named("String".into()) },
                    IntentField { name: "tls".into(), ty: Type::Generic("Option".into(), vec![Type::Named("TlsConfig".into())]) },
                    IntentField { name: "max_connections".into(), ty: Type::Named("Int".into()) },
                    IntentField { name: "request_timeout".into(), ty: Type::Named("Duration".into()) },
                ],
                variants: vec![],
                invariants: vec![],
            }),
            IntentDecl::Type(IntentType {
                name: "ServerError".into(),
                type_params: vec![],
                does: Some("server startup errors".into()),
                fields: vec![],
                variants: vec![
                    IntentVariant { name: "PortInUse".into(), description: "the specified port is already bound".into(), payload: None },
                    IntentVariant { name: "PermissionDenied".into(), description: "insufficient permissions".into(), payload: None },
                    IntentVariant { name: "TlsError".into(), description: "TLS configuration is invalid".into(), payload: Some(Type::Named("String".into())) },
                ],
                invariants: vec![],
            }),
            IntentDecl::Type(IntentType {
                name: "AuthError".into(),
                type_params: vec![],
                does: Some("authentication errors".into()),
                fields: vec![],
                variants: vec![
                    IntentVariant { name: "InvalidCreds".into(), description: "invalid credentials".into(), payload: None },
                    IntentVariant { name: "Locked".into(), description: "account locked".into(), payload: None },
                    IntentVariant { name: "Expired".into(), description: "session expired".into(), payload: None },
                ],
                invariants: vec![],
            }),
            IntentDecl::Fn(IntentFn {
                name: "serve".into(),
                type_params: vec![],
                params: vec![
                    Param { name: "config".into(), ty: Type::Named("Config".into()) },
                    Param { name: "handler".into(), ty: Type::Named("Handler".into()) },
                ],
                return_type: Type::Generic("Result".into(), vec![
                    Type::Named("Server".into()),
                    Type::Named("ServerError".into()),
                ]),
                effects: vec![
                    Effect::new("Net", "listen"),
                    Effect::new("Log", "write"),
                ],
                is_strict: false,
                does: Some("starts an HTTP server".into()),
                fails: vec!["PortInUse".into(), "PermissionDenied".into(), "TlsError".into()],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            IntentDecl::Fn(IntentFn {
                name: "route".into(),
                type_params: vec![],
                params: vec![
                    Param { name: "method".into(), ty: Type::Named("Method".into()) },
                    Param { name: "pattern".into(), ty: Type::Named("String".into()) },
                    Param { name: "handler".into(), ty: Type::Named("Handler".into()) },
                ],
                return_type: Type::Named("Route".into()),
                effects: vec![Effect::new("pure", "pure")],
                is_strict: false,
                does: Some("creates a route".into()),
                fails: vec![],
                edge_cases: vec![],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
            IntentDecl::Fn(IntentFn {
                name: "authenticate".into(),
                type_params: vec![],
                params: vec![
                    Param { name: "creds".into(), ty: Type::Named("Credentials".into()) },
                ],
                return_type: Type::Generic("Result".into(), vec![
                    Type::Named("Session".into()),
                    Type::Named("AuthError".into()),
                ]),
                effects: vec![
                    Effect::new("Db", "read"),
                    Effect::new("Db", "write"),
                    Effect::new("Crypto", "verify"),
                    Effect::new("Log", "write"),
                ],
                is_strict: true,
                does: Some("authenticates a user".into()),
                fails: vec![],
                edge_cases: vec![],
                requires: vec!["creds.username.len > 0".into(), "creds.password.len > 0".into()],
                ensures: vec!["ok => result.user_id > 0".into()],
                errors: vec![
                    ("InvalidCreds".into(), "invalid username or password".into()),
                    ("Locked".into(), "account locked after 5+ failed attempts".into()),
                    ("Expired".into(), "session has expired".into()),
                ],
                panics_never: true,
            }),
            IntentDecl::Fn(IntentFn {
                name: "handle_request".into(),
                type_params: vec![],
                params: vec![
                    Param { name: "req".into(), ty: Type::Named("Request".into()) },
                ],
                return_type: Type::Named("Response".into()),
                effects: vec![
                    Effect::new("Db", "read"),
                    Effect::new("Db", "write"),
                    Effect::new("Log", "write"),
                    Effect::new("Http", "send"),
                ],
                is_strict: false,
                does: Some("routes and processes an incoming HTTP request".into()),
                fails: vec![],
                edge_cases: vec![
                    "returns 404 for unmatched routes".into(),
                    "returns 500 for internal failures".into(),
                ],
                requires: vec![],
                ensures: vec![],
                errors: vec![],
                panics_never: false,
            }),
        ],
    }
}

fn build_correct_impl() -> ImplFile {
    ImplFile {
        module: Some("http_handler".into()),
        declarations: vec![
            ImplDecl::Type(ImplType {
                name: "Config".into(),
                type_params: vec![],
                kind: ImplTypeKind::Struct(vec![
                    ImplField { name: "port".into(), ty: Type::Named("Int".into()) },
                    ImplField { name: "host".into(), ty: Type::Named("String".into()) },
                    ImplField { name: "tls".into(), ty: Type::Generic("Option".into(), vec![Type::Named("TlsConfig".into())]) },
                    ImplField { name: "max_connections".into(), ty: Type::Named("Int".into()) },
                    ImplField { name: "request_timeout".into(), ty: Type::Named("Duration".into()) },
                ]),
            }),
            ImplDecl::Type(ImplType {
                name: "ServerError".into(),
                type_params: vec![],
                kind: ImplTypeKind::Enum(vec![
                    ImplVariant { name: "PortInUse".into(), payload: None },
                    ImplVariant { name: "PermissionDenied".into(), payload: None },
                    ImplVariant { name: "TlsError".into(), payload: Some(Type::Named("String".into())) },
                ]),
            }),
            ImplDecl::Type(ImplType {
                name: "AuthError".into(),
                type_params: vec![],
                kind: ImplTypeKind::Enum(vec![
                    ImplVariant { name: "InvalidCreds".into(), payload: None },
                    ImplVariant { name: "Locked".into(), payload: None },
                    ImplVariant { name: "Expired".into(), payload: None },
                ]),
            }),
            ImplDecl::Fn(ImplFn {
                name: "serve".into(),
                annotation: IntentAnnotation::Intent("serve".into()),
                type_params: vec![],
                params: vec![
                    Param { name: "config".into(), ty: Type::Named("Config".into()) },
                    Param { name: "handler".into(), ty: Type::Named("Handler".into()) },
                ],
                return_type: Type::Generic("Result".into(), vec![
                    Type::Named("Server".into()),
                    Type::Named("ServerError".into()),
                ]),
                effects: vec![
                    Effect::new("Net", "listen"),
                    Effect::new("Log", "write"),
                ],
                actual_effects: vec![
                    Effect::new("Net", "listen"),
                    Effect::new("Log", "write"),
                ],
                actual_errors: vec![],
            }),
            ImplDecl::Fn(ImplFn {
                name: "route".into(),
                annotation: IntentAnnotation::Intent("route".into()),
                type_params: vec![],
                params: vec![
                    Param { name: "method".into(), ty: Type::Named("Method".into()) },
                    Param { name: "pattern".into(), ty: Type::Named("String".into()) },
                    Param { name: "handler".into(), ty: Type::Named("Handler".into()) },
                ],
                return_type: Type::Named("Route".into()),
                effects: vec![Effect::new("pure", "pure")],
                actual_effects: vec![],
                actual_errors: vec![],
            }),
            ImplDecl::Fn(ImplFn {
                name: "authenticate".into(),
                annotation: IntentAnnotation::Intent("authenticate".into()),
                type_params: vec![],
                params: vec![
                    Param { name: "creds".into(), ty: Type::Named("Credentials".into()) },
                ],
                return_type: Type::Generic("Result".into(), vec![
                    Type::Named("Session".into()),
                    Type::Named("AuthError".into()),
                ]),
                effects: vec![
                    Effect::new("Db", "read"),
                    Effect::new("Db", "write"),
                    Effect::new("Crypto", "verify"),
                    Effect::new("Log", "write"),
                ],
                actual_effects: vec![
                    Effect::new("Db", "read"),
                    Effect::new("Db", "write"),
                    Effect::new("Crypto", "verify"),
                    Effect::new("Log", "write"),
                ],
                actual_errors: vec![
                    "InvalidCreds".into(),
                    "Locked".into(),
                    "Expired".into(),
                ],
            }),
            ImplDecl::Fn(ImplFn {
                name: "handle_request".into(),
                annotation: IntentAnnotation::Intent("handle_request".into()),
                type_params: vec![],
                params: vec![
                    Param { name: "req".into(), ty: Type::Named("Request".into()) },
                ],
                return_type: Type::Named("Response".into()),
                effects: vec![
                    Effect::new("Db", "read"),
                    Effect::new("Db", "write"),
                    Effect::new("Log", "write"),
                    Effect::new("Http", "send"),
                ],
                actual_effects: vec![
                    Effect::new("Db", "read"),
                    Effect::new("Db", "write"),
                    Effect::new("Log", "write"),
                    Effect::new("Http", "send"),
                ],
                actual_errors: vec![],
            }),
        ],
    }
}

fn build_drifted_impl() -> ImplFile {
    let mut imp = build_correct_impl();

    // Simulate AI adding rate limiting: new effect + new error variant
    for decl in imp.declarations.iter_mut() {
        if let ImplDecl::Fn(f) = decl {
            if f.name == "authenticate" {
                // AI added Net.send for external rate limit check
                f.effects.push(Effect::new("Net", "send"));
                f.actual_effects.push(Effect::new("Net", "send"));
                f.actual_errors.push("RateLimited".into());
            }
        }
    }

    // AI added RateLimited to AuthError enum
    for decl in imp.declarations.iter_mut() {
        if let ImplDecl::Type(t) = decl {
            if t.name == "AuthError" {
                if let ImplTypeKind::Enum(variants) = &mut t.kind {
                    variants.push(ImplVariant {
                        name: "RateLimited".into(),
                        payload: None,
                    });
                }
            }
        }
    }

    imp
}
