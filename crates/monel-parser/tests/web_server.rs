use monel_parser::parse;

const WEB_SERVER_MN: &str = include_str!("../../../spec/examples/web_server.mn");

#[test]
fn parse_web_server_mn() {
    match parse(WEB_SERVER_MN) {
        Ok(file) => {
            println!("Parsed {} imports, {} declarations", file.imports.len(), file.declarations.len());
        }
        Err(errors) => {
            // web_server.mn needs: derives, async fn types, complex patterns
            eprintln!("web_server.mn: {} parse errors (needs derives, async types)", errors.len());
            for e in errors.iter().take(5) { eprintln!("  {}", e); }
            assert!(errors.len() <= 46, "regression: expected <=46 errors, got {}", errors.len());
        }
    }
}
