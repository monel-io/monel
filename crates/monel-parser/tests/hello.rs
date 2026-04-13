use monel_parser::parse;

const HELLO_MN: &str = include_str!("../../../spec/examples/hello.mn");

#[test]
fn parse_hello_mn() {
    let result = parse(HELLO_MN);
    match &result {
        Ok(file) => {
            println!("Parsed successfully!");
            println!("  Imports: {}", file.imports.len());
            println!("  Declarations: {}", file.declarations.len());
            for decl in &file.declarations {
                match decl {
                    monel_parser::ast::Decl::Fn(f) => {
                        println!("  fn {} ({} params, requires: {}, ensures: {}, effects: {})",
                            f.name, f.params.len(), f.requires.len(),
                            f.ensures.len(), f.effects.len());
                    }
                    monel_parser::ast::Decl::Type(t) => {
                        println!("  type {} ({:?})", t.name, match &t.kind {
                            monel_parser::ast::TypeDeclKind::Struct(f) => format!("struct, {} fields", f.len()),
                            monel_parser::ast::TypeDeclKind::Enum(v) => format!("enum, {} variants", v.len()),
                            monel_parser::ast::TypeDeclKind::Alias(_) => "alias".to_string(),
                            monel_parser::ast::TypeDeclKind::Distinct(_) => "distinct".to_string(),
                        });
                    }
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
fn hello_has_correct_imports() {
    let file = parse(HELLO_MN).expect("parse failed");
    assert_eq!(file.imports.len(), 2);
    assert_eq!(file.imports[0].path, "std/io");
    assert_eq!(file.imports[0].names, vec!["println"]);
    assert_eq!(file.imports[1].path, "std/time");
    assert_eq!(file.imports[1].names, vec!["Clock", "Instant"]);
}

#[test]
fn hello_has_correct_types() {
    let file = parse(HELLO_MN).expect("parse failed");
    let types: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        monel_parser::ast::Decl::Type(t) => Some(t),
        _ => None,
    }).collect();

    assert_eq!(types.len(), 3, "expected 3 type declarations");
    assert_eq!(types[0].name, "Name");
    assert_eq!(types[1].name, "Greeter");
    assert_eq!(types[2].name, "GreetError");

    // Name is distinct String
    assert!(matches!(&types[0].kind, monel_parser::ast::TypeDeclKind::Distinct(_)));

    // Greeter is a struct with 4 fields and invariants
    if let monel_parser::ast::TypeDeclKind::Struct(fields) = &types[1].kind {
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].name, "name");
        assert_eq!(fields[1].name, "greet_count");
        assert_eq!(fields[2].name, "last_greeted");
        assert_eq!(fields[3].name, "history");
    } else {
        panic!("expected Greeter to be a struct");
    }
    assert_eq!(types[1].invariants.len(), 3, "Greeter should have 3 invariants");

    // GreetError is an enum with 2 variants
    if let monel_parser::ast::TypeDeclKind::Enum(variants) = &types[2].kind {
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].name, "EmptyMessage");
        assert_eq!(variants[1].name, "TooLong");
    } else {
        panic!("expected GreetError to be an enum");
    }
}

#[test]
fn hello_has_correct_functions() {
    let file = parse(HELLO_MN).expect("parse failed");
    let fns: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        monel_parser::ast::Decl::Fn(f) => Some(f),
        _ => None,
    }).collect();

    // Expected: new, greet, greet_count, name, has_greeted, last_greeted, nth, find_greeting, main
    let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"new"), "missing fn new");
    assert!(names.contains(&"greet"), "missing fn greet");
    assert!(names.contains(&"greet_count"), "missing fn greet_count");
    assert!(names.contains(&"name"), "missing fn name");
    assert!(names.contains(&"has_greeted"), "missing fn has_greeted");
    assert!(names.contains(&"last_greeted"), "missing fn last_greeted");
    assert!(names.contains(&"nth"), "missing fn nth");
    assert!(names.contains(&"find_greeting"), "missing fn find_greeting");
    assert!(names.contains(&"main"), "missing fn main");
}

#[test]
fn hello_greet_has_contracts() {
    let file = parse(HELLO_MN).expect("parse failed");
    let greet = file.declarations.iter().find_map(|d| match d {
        monel_parser::ast::Decl::Fn(f) if f.name == "greet" => Some(f),
        _ => None,
    }).expect("fn greet not found");

    assert_eq!(greet.requires.len(), 1, "greet should have 1 requires");
    assert_eq!(greet.ensures.len(), 5, "greet should have 5 ensures");
    assert_eq!(greet.effects.len(), 2, "greet should have 2 effects");
    assert!(matches!(greet.panics, monel_parser::ast::PanicsDecl::Never));
    assert!(greet.body.is_some(), "greet should have a body");
}

#[test]
fn hello_compact_fns_have_bodies() {
    let file = parse(HELLO_MN).expect("parse failed");
    let greet_count = file.declarations.iter().find_map(|d| match d {
        monel_parser::ast::Decl::Fn(f) if f.name == "greet_count" => Some(f),
        _ => None,
    }).expect("fn greet_count not found");

    assert!(matches!(greet_count.body, Some(monel_parser::ast::FnBody::Compact(_))));
}
