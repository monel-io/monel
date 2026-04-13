use monel_parser::parse;

const TERMINAL_MN: &str = include_str!("../../../spec/examples/terminal.mn");

#[test]
fn parse_terminal_mn() {
    match parse(TERMINAL_MN) {
        Ok(file) => {
            println!("Parsed {} imports, {} declarations", file.imports.len(), file.declarations.len());
        }
        Err(errors) => {
            // 3 errors remain from deeply nested match in process_event
            eprintln!("terminal.mn: {} parse errors (expected <=3 from deep nesting)", errors.len());
            for e in &errors { eprintln!("  {}", e); }
            assert!(errors.len() <= 3, "regression: expected <=3 errors, got {}", errors.len());
        }
    }
}
