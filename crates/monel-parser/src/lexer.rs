use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum RawToken {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("type")]
    Type,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("const")]
    Const,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("owned")]
    Owned,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("while")]
    While,
    #[token("loop")]
    Loop,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("try")]
    Try,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("unsafe")]
    Unsafe,
    #[token("where")]
    Where,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("None")]
    None_,
    #[token("Some")]
    Some_,
    #[token("Ok")]
    Ok_,
    #[token("Err")]
    Err_,
    #[token("self")]
    SelfLower,
    #[token("Self")]
    SelfUpper,
    #[token("distinct")]
    Distinct,
    #[token("state_machine")]
    StateMachine,
    #[token("layout")]
    Layout,
    #[token("interaction")]
    Interaction,

    // Contract keywords
    #[token("requires:")]
    Requires,
    #[token("ensures:")]
    Ensures,
    #[token("effects:")]
    Effects,
    #[token("panics:")]
    Panics,
    #[token("complexity:")]
    Complexity,
    #[token("invariant:")]
    Invariant,
    #[token("fails:")]
    Fails,
    #[token("doc:")]
    Doc,
    #[token("never")]
    Never,
    #[token("old")]
    Old,
    #[token("result")]
    Result_,
    #[token("states:")]
    States,
    #[token("transitions:")]
    Transitions,
    #[token("initial:")]
    Initial,
    #[token("terminal:")]
    Terminal,
    #[token("regions:")]
    Regions,
    #[token("constraint:")]
    Constraint,
    #[token("derives:")]
    Derives,

    // Operators
    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token(">=")]
    GtEq,
    #[token("<=")]
    LtEq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("&")]
    Ampersand,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("|")]
    Pipe,
    #[token("?")]
    Question,

    // Literals
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    Float(f64),

    // String literal (simple, no interpolation at lex level)
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String_(String),

    // Character literal: 'a', ' '
    #[regex(r"'[^']'", |lex| {
        let s = lex.slice();
        Some(s.chars().nth(1).unwrap())
    })]
    Char(char),

    // Identifiers (must come after keywords so keywords take priority)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Comments
    #[regex(r"#[^\n]*", allow_greedy = true)]
    Comment,

    // Newlines (significant for indentation)
    #[regex(r"\n")]
    Newline,
}

/// Token with span information and indentation-derived tokens.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Raw(RawToken),
    Indent,
    Dedent,
    Newline,
    Eof,
}

/// A token with its source location.
#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

/// Tokenizes source code and produces a stream with Indent/Dedent/Newline tokens.
pub fn tokenize(input: &str) -> Vec<Spanned> {
    let mut result = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];
    let lines: Vec<&str> = input.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;

        // Skip completely empty lines and comment-only lines
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            // Still emit the comment tokens for comment-only lines
            if trimmed.starts_with('#') {
                // We skip comments in the token stream
            }
            continue;
        }

        // Calculate indentation (number of leading spaces)
        let indent = line.len() - line.trim_start().len();
        let current_indent = *indent_stack.last().unwrap();

        if indent > current_indent {
            indent_stack.push(indent);
            result.push(Spanned {
                token: Token::Indent,
                line: line_num,
                col: 1,
            });
        } else if indent < current_indent {
            while *indent_stack.last().unwrap() > indent {
                indent_stack.pop();
                result.push(Spanned {
                    token: Token::Dedent,
                    line: line_num,
                    col: 1,
                });
            }
        }

        // Emit Newline between logical lines (not before the first)
        if !result.is_empty()
            && !matches!(
                result.last().map(|s| &s.token),
                Some(Token::Indent) | Some(Token::Dedent)
            )
        {
            result.push(Spanned {
                token: Token::Newline,
                line: line_num,
                col: 1,
            });
        }

        // Tokenize this line
        let line_content = line.trim_start();
        let col_offset = indent + 1;
        let mut lexer = RawToken::lexer(line_content);

        while let Some(tok_result) = lexer.next() {
            match tok_result {
                Ok(RawToken::Comment) => break, // skip rest of line
                Ok(RawToken::Newline) => {} // handled by line iteration
                Ok(tok) => {
                    let span = lexer.span();
                    result.push(Spanned {
                        token: Token::Raw(tok),
                        line: line_num,
                        col: col_offset + span.start,
                    });
                }
                Err(()) => {
                    let span = lexer.span();
                    // Skip unknown characters for now
                    eprintln!(
                        "Lexer error at {}:{}: unexpected '{}'",
                        line_num,
                        col_offset + span.start,
                        &line_content[span.clone()]
                    );
                }
            }
        }
    }

    // Close remaining indentation levels
    while indent_stack.len() > 1 {
        indent_stack.pop();
        let line_num = lines.len();
        result.push(Spanned {
            token: Token::Dedent,
            line: line_num,
            col: 1,
        });
    }

    result.push(Spanned {
        token: Token::Eof,
        line: lines.len(),
        col: 1,
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_fn() {
        let input = "fn main() -> Int\n  return 42\n";
        let tokens = tokenize(input);
        let kinds: Vec<&Token> = tokens.iter().map(|s| &s.token).collect();
        assert_eq!(kinds[0], &Token::Raw(RawToken::Fn));
        assert_eq!(kinds[1], &Token::Raw(RawToken::Ident("main".into())));
        assert_eq!(kinds[2], &Token::Raw(RawToken::LParen));
        assert_eq!(kinds[3], &Token::Raw(RawToken::RParen));
        assert_eq!(kinds[4], &Token::Raw(RawToken::Arrow));
        assert_eq!(kinds[5], &Token::Raw(RawToken::Ident("Int".into())));
        assert_eq!(kinds[6], &Token::Indent);
        assert_eq!(kinds[7], &Token::Raw(RawToken::Return));
        assert_eq!(kinds[8], &Token::Raw(RawToken::Int(42)));
        assert_eq!(kinds[9], &Token::Dedent);
        assert_eq!(kinds[10], &Token::Eof);
    }

    #[test]
    fn tokenize_indentation() {
        let input = "type Foo\n  x: Int\n  y: String\nfn bar()\n  return 1\n";
        let tokens = tokenize(input);
        let kinds: Vec<&Token> = tokens.iter().map(|s| &s.token).collect();
        // type Foo INDENT x : Int NEWLINE y : String DEDENT fn bar ( ) INDENT return 1 DEDENT EOF
        assert!(kinds.contains(&&Token::Indent));
        assert!(kinds.contains(&&Token::Dedent));
    }

    #[test]
    fn tokenize_contract_keywords() {
        let input = "fn f(x: Int)\n  requires:\n    x > 0\n  return x\n";
        let tokens = tokenize(input);
        let has_requires = tokens
            .iter()
            .any(|s| matches!(&s.token, Token::Raw(RawToken::Requires)));
        assert!(has_requires);
    }

    #[test]
    fn tokenize_integer_with_underscores() {
        let input = "let x = 1_000_000\n";
        let tokens = tokenize(input);
        let has_int = tokens
            .iter()
            .any(|s| matches!(&s.token, Token::Raw(RawToken::Int(1_000_000))));
        assert!(has_int);
    }

    #[test]
    fn tokenize_string_literal() {
        let input = "let s = \"hello world\"\n";
        let tokens = tokenize(input);
        let has_str = tokens.iter().any(|s| {
            matches!(&s.token, Token::Raw(RawToken::String_(v)) if v == "hello world")
        });
        assert!(has_str);
    }

    #[test]
    fn tokenize_skips_comment_lines() {
        let input = "# this is a comment\nfn main()\n  return 0\n";
        let tokens = tokenize(input);
        // First real token should be fn
        assert_eq!(tokens[0].token, Token::Raw(RawToken::Fn));
    }

    #[test]
    fn tokenize_hello_mn_does_not_panic() {
        let source = include_str!("../../../spec/examples/hello.mn");
        let tokens = tokenize(source);
        // Should produce tokens without panicking
        assert!(!tokens.is_empty());
        // Should end with EOF
        assert_eq!(tokens.last().unwrap().token, Token::Eof);
        // Should have at least some fn tokens
        let fn_count = tokens
            .iter()
            .filter(|s| matches!(&s.token, Token::Raw(RawToken::Fn)))
            .count();
        assert!(fn_count >= 5, "Expected at least 5 fn tokens, got {fn_count}");
    }
}
