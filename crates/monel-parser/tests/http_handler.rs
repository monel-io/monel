use monel_parser::parse;
use monel_parser::ast::*;

const HTTP_MN: &str = include_str!("../../../spec/examples/http_handler.mn");

#[test]
fn parse_http_handler_mn() {
    let result = parse(HTTP_MN);
    match &result {
        Ok(file) => {
            println!("Parsed successfully!");
            println!("  Imports: {}", file.imports.len());
            for decl in &file.declarations {
                match decl {
                    Decl::Fn(f) => println!("  fn {} ({} params, ensures: {})",
                        f.name, f.params.len(), f.ensures.len()),
                    Decl::Type(t) => println!("  type {}", t.name),
                    _ => {}
                }
            }
        }
        Err(errors) => {
            for e in errors {
                eprintln!("  ERROR: {}", e);
            }
            panic!("Parse failed with {} errors", errors.len());
        }
    }
}

#[test]
fn http_handler_structure() {
    let file = parse(HTTP_MN).expect("parse failed");
    assert_eq!(file.imports.len(), 4);

    let types: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        Decl::Type(t) => Some(t), _ => None,
    }).collect();
    assert_eq!(types.len(), 5);

    let fns: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        Decl::Fn(f) => Some(f), _ => None,
    }).collect();
    let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"serve"));
    assert!(names.contains(&"authenticate"));
    assert!(names.contains(&"handle_request"));
}
