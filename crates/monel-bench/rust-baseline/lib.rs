// A typical Rust HTTP auth service — no intent, no effect tracking, no parity checking.
// This is what an AI agent encounters when asked to modify "authenticate".

pub mod types;
pub mod errors;
pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod config;
