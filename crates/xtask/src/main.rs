use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(|s| s.as_str()) {
        Some("book-build") => run("mdbook", &["build"]),
        Some("book-serve") => run("mdbook", &["serve", "--open"]),
        Some("book-install") => run("cargo", &["install", "mdbook@0.4.44", "mdbook-mermaid@0.14.0"]),
        _ => {
            eprintln!("Usage: cargo xtask <COMMAND>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  book-build     Build the spec book with mdbook");
            eprintln!("  book-serve     Serve the spec book with live reload");
            eprintln!("  book-install   Install mdbook and plugins");
            exit(if args.is_empty() { 0 } else { 1 });
        }
    }
}

fn run(program: &str, args: &[&str]) {
    let status = Command::new(program)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run `{program}`: {e}");
            exit(1);
        });

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}
