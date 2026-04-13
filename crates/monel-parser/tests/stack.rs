use monel_parser::parse;
use monel_parser::ast::*;

const STACK_MN: &str = include_str!("../../../spec/examples/stack.mn");

#[test]
fn parse_stack_mn() {
    let result = parse(STACK_MN);
    match &result {
        Ok(file) => {
            println!("Parsed successfully!");
            println!("  Imports: {}", file.imports.len());
            println!("  Declarations: {}", file.declarations.len());
            for decl in &file.declarations {
                match decl {
                    Decl::Fn(f) => println!("  fn {}", f.name),
                    Decl::Type(t) => println!("  type {}", t.name),
                    Decl::Const(c) => println!("  const {}", c.name),
                    Decl::Trait(t) => println!("  trait {} ({} methods)", t.name, t.methods.len()),
                    Decl::Impl(i) => println!("  impl {} ({} methods)",
                        i.trait_name.as_deref().unwrap_or("(inherent)"),
                        i.methods.len()),
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
fn stack_has_correct_structure() {
    let file = parse(STACK_MN).expect("parse failed");

    // 2 imports
    assert_eq!(file.imports.len(), 2);

    // Count declaration types
    let consts: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        Decl::Const(c) => Some(c), _ => None,
    }).collect();
    let types: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        Decl::Type(t) => Some(t), _ => None,
    }).collect();
    let traits: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        Decl::Trait(t) => Some(t), _ => None,
    }).collect();
    let impls: Vec<_> = file.declarations.iter().filter_map(|d| match d {
        Decl::Impl(i) => Some(i), _ => None,
    }).collect();

    assert_eq!(consts.len(), 2, "expected 2 consts");
    assert_eq!(consts[0].name, "DEFAULT_CAPACITY");
    assert_eq!(consts[1].name, "MAX_CAPACITY");
    assert!(matches!(consts[0].visibility, Visibility::Public));
    assert!(matches!(consts[1].visibility, Visibility::Private));

    assert_eq!(types.len(), 2, "expected 2 types");
    assert_eq!(types[0].name, "Stack");
    assert_eq!(types[1].name, "StackError");

    assert_eq!(traits.len(), 1, "expected 1 trait");
    assert_eq!(traits[0].name, "Iterable");
    assert_eq!(traits[0].associated_types.len(), 1);
    assert_eq!(traits[0].methods.len(), 2, "Iterable should have iter + count");

    assert_eq!(impls.len(), 3, "expected 3 impl blocks");
}

#[test]
fn stack_inherent_impl_has_methods() {
    let file = parse(STACK_MN).expect("parse failed");
    let inherent = file.declarations.iter().find_map(|d| match d {
        Decl::Impl(i) if i.trait_name.is_none() => Some(i),
        _ => None,
    }).expect("no inherent impl found");

    let method_names: Vec<&str> = inherent.methods.iter().map(|m| m.name.as_str()).collect();
    assert!(method_names.contains(&"new"), "missing new");
    assert!(method_names.contains(&"push"), "missing push");
    assert!(method_names.contains(&"pop"), "missing pop");
    assert!(method_names.contains(&"peek"), "missing peek");
    assert!(method_names.contains(&"len"), "missing len");
    assert!(method_names.contains(&"is_empty"), "missing is_empty");
    assert!(method_names.contains(&"clear"), "missing clear");
    assert!(method_names.contains(&"into_vec"), "missing into_vec");

    // into_vec takes owned self
    let into_vec = inherent.methods.iter().find(|m| m.name == "into_vec").unwrap();
    assert!(matches!(&into_vec.params[0].ty, TypeExpr::Owned(_)));
}

#[test]
fn stack_display_impl_has_where_clause() {
    let file = parse(STACK_MN).expect("parse failed");
    let display_impl = file.declarations.iter().find_map(|d| match d {
        Decl::Impl(i) if i.trait_name.as_deref() == Some("Display") => Some(i),
        _ => None,
    }).expect("no Display impl found");

    assert_eq!(display_impl.where_clauses.len(), 1);
    assert_eq!(display_impl.where_clauses[0].param, "T");
    assert!(display_impl.where_clauses[0].bounds.contains(&"Display".to_string()));
}

#[test]
fn stack_push_has_contracts() {
    let file = parse(STACK_MN).expect("parse failed");
    let inherent = file.declarations.iter().find_map(|d| match d {
        Decl::Impl(i) if i.trait_name.is_none() => Some(i),
        _ => None,
    }).expect("no inherent impl found");

    let push = inherent.methods.iter().find(|m| m.name == "push").unwrap();
    assert_eq!(push.ensures.len(), 3, "push should have 3 ensures");
    assert!(matches!(push.panics, PanicsDecl::Never));
    assert!(matches!(push.visibility, Visibility::Public));
}
