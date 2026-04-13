use crate::ast::*;
use crate::lexer::{tokenize, RawToken, Spanned, Token};

/// Parse error with location.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.message)
    }
}

/// Parse a `.mn` source file into a SourceFile AST.
pub fn parse(input: &str) -> Result<SourceFile, Vec<ParseError>> {
    let tokens = tokenize(input);
    let mut p = Parser::new(tokens);
    p.parse_file()
}

struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
    errors: Vec<ParseError>,
    indent_depth: usize,
}

impl Parser {
    fn new(tokens: Vec<Spanned>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: vec![],
            indent_depth: 0,
        }
    }

    // ─── Token navigation ───

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .map(|s| &s.token)
            .unwrap_or(&Token::Eof)
    }

    fn peek_raw(&self) -> Option<&RawToken> {
        match self.peek() {
            Token::Raw(r) => Some(r),
            _ => None,
        }
    }

    fn advance(&mut self) -> &Token {
        let tok = self.tokens.get(self.pos).map(|s| &s.token).unwrap_or(&Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn current_loc(&self) -> (usize, usize) {
        self.tokens
            .get(self.pos)
            .map(|s| (s.line, s.col))
            .unwrap_or((0, 0))
    }

    fn expect_raw(&mut self, expected: &RawToken) -> Result<(), ParseError> {
        if self.peek_raw() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            let (line, col) = self.current_loc();
            Err(ParseError {
                message: format!("expected {:?}, got {:?}", expected, self.peek()),
                line,
                col,
            })
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    fn skip_indented_block(&mut self) {
        let mut depth = 1i32;
        while depth > 0 && !self.at_eof() {
            match self.peek() {
                Token::Indent => { depth += 1; self.advance(); self.indent_depth += 1; }
                Token::Dedent => { depth -= 1; self.advance(); self.indent_depth = self.indent_depth.saturating_sub(1); }
                _ => { self.advance(); }
            }
        }
    }

    fn consume_indent(&mut self) -> bool {
        if self.peek() == &Token::Indent {
            self.advance();
            self.indent_depth += 1;
            true
        } else {
            false
        }
    }

    fn consume_dedent(&mut self) -> bool {
        if self.peek() == &Token::Dedent {
            self.advance();
            self.indent_depth = self.indent_depth.saturating_sub(1);
            true
        } else {
            false
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn eat_ident(&mut self) -> Result<String, ParseError> {
        match self.peek_raw().cloned() {
            Some(RawToken::Ident(name)) => {
                self.advance();
                Ok(name)
            }
            _ => {
                let (line, col) = self.current_loc();
                Err(ParseError {
                    message: format!("expected identifier, got {:?}", self.peek()),
                    line,
                    col,
                })
            }
        }
    }

    fn eat_ident_or_keyword(&mut self) -> Result<String, ParseError> {
        // Some keywords can appear as identifiers in certain contexts (e.g., type names)
        match self.peek_raw().cloned() {
            Some(RawToken::Ident(name)) => { self.advance(); Ok(name) }
            Some(RawToken::Result_) => { self.advance(); Ok("Result".to_string()) }
            Some(RawToken::Some_) => { self.advance(); Ok("Some".to_string()) }
            Some(RawToken::None_) => { self.advance(); Ok("None".to_string()) }
            Some(RawToken::Ok_) => { self.advance(); Ok("Ok".to_string()) }
            Some(RawToken::Err_) => { self.advance(); Ok("Err".to_string()) }
            Some(RawToken::SelfUpper) => { self.advance(); Ok("Self".to_string()) }
            _ => self.eat_ident(),
        }
    }

    fn eat_field_name(&mut self) -> Result<String, ParseError> {
        match self.peek_raw().cloned() {
            Some(RawToken::Ident(name)) => { self.advance(); Ok(name) }
            Some(RawToken::Match) => { self.advance(); Ok("match".into()) }
            Some(RawToken::Type) => { self.advance(); Ok("type".into()) }
            Some(RawToken::Result_) => { self.advance(); Ok("Result".into()) }
            Some(RawToken::Some_) => { self.advance(); Ok("Some".into()) }
            Some(RawToken::None_) => { self.advance(); Ok("None".into()) }
            Some(RawToken::Ok_) => { self.advance(); Ok("Ok".into()) }
            Some(RawToken::Err_) => { self.advance(); Ok("Err".into()) }
            Some(RawToken::SelfUpper) => { self.advance(); Ok("Self".into()) }
            Some(RawToken::Old) => { self.advance(); Ok("old".into()) }
            Some(RawToken::Never) => { self.advance(); Ok("never".into()) }
            _ => self.eat_ident(),
        }
    }

    fn err(&self, msg: &str) -> ParseError {
        let (line, col) = self.current_loc();
        ParseError {
            message: msg.to_string(),
            line,
            col,
        }
    }

    // ─── File parsing ───

    fn parse_file(&mut self) -> Result<SourceFile, Vec<ParseError>> {
        let mut imports = vec![];
        let mut declarations = vec![];

        self.skip_newlines();

        while !self.at_eof() {
            self.skip_newlines();
            if self.at_eof() {
                break;
            }

            // Skip stray dedents at top level (can happen with nested indentation)
            while matches!(self.peek(), Token::Dedent) {
                self.advance();
            }
            if self.at_eof() {
                break;
            }

            match self.peek_raw() {
                Some(RawToken::Use) => match self.parse_import() {
                    Ok(imp) => imports.push(imp),
                    Err(e) => {
                        self.errors.push(e);
                        self.recover_to_top_level();
                    }
                },
                Some(RawToken::Fn | RawToken::Pub | RawToken::Type | RawToken::Const
                     | RawToken::Trait | RawToken::Impl | RawToken::Async
                     | RawToken::StateMachine | RawToken::Layout | RawToken::Interaction) => {
                    match self.parse_decl() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => {
                            self.errors.push(e);
                            self.recover_to_top_level();
                        }
                    }
                }
                _ => {
                    self.errors.push(self.err(&format!(
                        "unexpected token at top level: {:?}",
                        self.peek()
                    )));
                    self.recover_to_top_level();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(SourceFile {
                imports,
                declarations,
            })
        } else {
            Err(self.errors.clone())
        }
    }

    fn recover_to_top_level(&mut self) {
        loop {
            match self.peek() {
                Token::Eof => break,
                Token::Raw(RawToken::Fn | RawToken::Type | RawToken::Use | RawToken::Pub
                           | RawToken::Const | RawToken::Trait | RawToken::Impl
                           | RawToken::Async | RawToken::StateMachine) => break,
                Token::Newline => {
                    self.advance();
                    // Check if next is a top-level keyword
                    match self.peek_raw() {
                        Some(RawToken::Fn | RawToken::Type | RawToken::Use | RawToken::Pub
                             | RawToken::Const | RawToken::Trait | RawToken::Impl
                             | RawToken::Async | RawToken::StateMachine) => break,
                        _ => {}
                    }
                }
                _ => { self.advance(); }
            }
        }
    }

    // ─── Imports ───

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.expect_raw(&RawToken::Use)?;
        // Path: `std/io` parsed as Ident("std") Slash Ident("io")
        let mut path = self.eat_ident()?;
        while self.peek_raw() == Some(&RawToken::Slash) {
            self.advance();
            let segment = self.eat_ident_or_keyword()?;
            path.push('/');
            path.push_str(&segment);
        }
        // Names: `{println, readln}`
        let mut names = vec![];
        if self.peek_raw() == Some(&RawToken::LBrace) {
            self.advance();
            loop {
                let name = self.eat_ident_or_keyword()?;
                // Handle `Name as Alias`
                if let Some(RawToken::Ident(kw)) = self.peek_raw() {
                    if kw == "as" {
                        self.advance();
                        let _alias = self.eat_ident_or_keyword()?;
                        // Store aliased name (for now just use original)
                    }
                }
                names.push(name);
                if self.peek_raw() == Some(&RawToken::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_raw(&RawToken::RBrace)?;
        }
        Ok(Import { path, names })
    }

    // ─── Declarations ───

    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        let vis = self.parse_visibility();
        let is_async = if self.peek_raw() == Some(&RawToken::Async) {
            self.advance();
            true
        } else {
            false
        };

        match self.peek_raw() {
            Some(RawToken::Fn) => Ok(Decl::Fn(self.parse_fn_decl(vis, is_async)?)),
            Some(RawToken::Type) => Ok(Decl::Type(self.parse_type_decl(vis)?)),
            Some(RawToken::Const) => Ok(Decl::Const(self.parse_const_decl(vis)?)),
            Some(RawToken::Trait) => Ok(Decl::Trait(self.parse_trait_decl(vis)?)),
            Some(RawToken::Impl) => Ok(Decl::Impl(self.parse_impl_block()?)),
            Some(RawToken::StateMachine) => Ok(Decl::StateMachine(self.parse_state_machine()?)),
            Some(RawToken::Layout | RawToken::Interaction) => {
                self.advance();
                let _name = self.eat_ident()?;
                if self.consume_indent() {
                    self.skip_indented_block();
                }
                Ok(Decl::StateMachine(StateMachineDecl {
                    name: _name,
                    doc: None,
                    states: vec![],
                    transitions: vec![],
                    initial: String::new(),
                    terminal: vec![],
                }))
            }
            _ => Err(self.err(&format!("expected declaration, got {:?}", self.peek()))),
        }
    }

    fn parse_visibility(&mut self) -> Visibility {
        if self.peek_raw() == Some(&RawToken::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    // ─── Function declarations ───

    fn parse_fn_decl(&mut self, vis: Visibility, is_async: bool) -> Result<FnDecl, ParseError> {
        self.expect_raw(&RawToken::Fn)?;
        let name = self.eat_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect_raw(&RawToken::LParen)?;
        let params = self.parse_params()?;
        self.expect_raw(&RawToken::RParen)?;

        let return_type = if self.peek_raw() == Some(&RawToken::Arrow) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        // Check for compact form: `= expr`
        if self.peek_raw() == Some(&RawToken::Eq) {
            self.advance();
            let expr = self.parse_expr()?;
            return Ok(FnDecl {
                visibility: vis,
                is_async,
                name,
                type_params,
                params,
                return_type,
                doc: None,
                requires: vec![],
                ensures: vec![],
                effects: vec![],
                panics: PanicsDecl::Unspecified,
                complexity: None,
                fails: vec![],
                body: Some(FnBody::Compact(expr)),
            });
        }

        // Parse contract clauses and body from indented block
        let mut doc = None;
        let mut requires = vec![];
        let mut ensures = vec![];
        let mut effects = vec![];
        let mut panics = PanicsDecl::Unspecified;
        let mut complexity = None;
        let mut fails = vec![];
        let mut body_stmts = vec![];

        if self.peek() == &Token::Indent {
            let entry_depth = self.indent_depth;
            self.consume_indent();

            // Parse contract clauses until we hit a non-contract line
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                match self.peek_raw() {
                    Some(RawToken::Doc) => {
                        self.advance();
                        if let Some(RawToken::String_(s)) = self.peek_raw().cloned() {
                            self.advance();
                            doc = Some(s);
                        }
                    }
                    Some(RawToken::Requires) => {
                        self.advance();
                        requires.extend(self.parse_indented_exprs()?);
                    }
                    Some(RawToken::Ensures) => {
                        self.advance();
                        ensures.extend(self.parse_ensures_clauses()?);
                    }
                    Some(RawToken::Effects) => {
                        self.advance();
                        effects = self.parse_effect_list()?;
                    }
                    Some(RawToken::Panics) => {
                        self.advance();
                        panics = self.parse_panics()?;
                    }
                    Some(RawToken::Complexity) => {
                        self.advance();
                        complexity = Some(self.parse_complexity()?);
                    }
                    Some(RawToken::Fails) => {
                        self.advance();
                        fails = self.parse_fails()?;
                    }
                    _ => {
                        // Not a contract keyword; this is the body
                        body_stmts = self.parse_body_stmts()?;
                        break;
                    }
                }
            }

            // Consume dedents until we're back to the depth we started at
            while self.indent_depth > entry_depth {
                if !self.consume_dedent() {
                    break;
                }
            }
        }

        let body = if body_stmts.is_empty() {
            None
        } else {
            Some(FnBody::Block(body_stmts))
        };

        Ok(FnDecl {
            visibility: vis,
            is_async,
            name,
            type_params,
            params,
            return_type,
            doc,
            requires,
            ensures,
            effects,
            panics,
            complexity,
            fails,
            body,
        })
    }

    // ─── Contract clause parsing ───

    fn parse_indented_exprs(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = vec![];
        if self.peek() == &Token::Indent {
            self.advance();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                exprs.push(self.parse_expr()?);
            }
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        } else {
            // Single-line: `requires: x > 0` (no indent)
            exprs.push(self.parse_expr()?);
        }
        Ok(exprs)
    }

    fn parse_ensures_clauses(&mut self) -> Result<Vec<EnsuresClause>, ParseError> {
        let mut clauses = vec![];
        if self.peek() == &Token::Indent {
            self.advance();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                clauses.push(self.parse_one_ensures()?);
            }
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        } else {
            clauses.push(self.parse_one_ensures()?);
        }
        Ok(clauses)
    }

    fn parse_one_ensures(&mut self) -> Result<EnsuresClause, ParseError> {
        // Check for `ok =>` (lowercase ok)
        if let Some(RawToken::Ident(name)) = self.peek_raw().cloned() {
            if name == "ok" {
                self.advance();
                self.expect_raw(&RawToken::FatArrow)?;
                let expr = self.parse_expr()?;
                return Ok(EnsuresClause::Ok(expr));
            }
        }

        match self.peek_raw() {
            Some(RawToken::Ok_) => {
                self.advance();
                self.expect_raw(&RawToken::FatArrow)?;
                let expr = self.parse_expr()?;
                Ok(EnsuresClause::Ok(expr))
            }
            Some(RawToken::Err_) => {
                self.advance();
                self.expect_raw(&RawToken::LParen)?;
                let variant = self.eat_ident_or_keyword()?;
                // May have a wildcard pattern like `TooLong(_)`
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    // skip inner pattern for now
                    while self.peek_raw() != Some(&RawToken::RParen) && !self.at_eof() {
                        self.advance();
                    }
                    if self.peek_raw() == Some(&RawToken::RParen) {
                        self.advance();
                    }
                }
                self.expect_raw(&RawToken::RParen)?;
                self.expect_raw(&RawToken::FatArrow)?;
                let expr = self.parse_expr()?;
                Ok(EnsuresClause::Err(variant, expr))
            }
            Some(RawToken::Ident(name)) if name == "err" => {
                self.advance();
                // Bare `err =>` without variant
                if self.peek_raw() == Some(&RawToken::FatArrow) {
                    self.advance();
                    let expr = self.parse_expr()?;
                    return Ok(EnsuresClause::Err("_".to_string(), expr));
                }
                self.expect_raw(&RawToken::LParen)?;
                let variant = self.eat_ident_or_keyword()?;
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    while self.peek_raw() != Some(&RawToken::RParen) && !self.at_eof() {
                        self.advance();
                    }
                    if self.peek_raw() == Some(&RawToken::RParen) {
                        self.advance();
                    }
                }
                self.expect_raw(&RawToken::RParen)?;
                self.expect_raw(&RawToken::FatArrow)?;
                let expr = self.parse_expr()?;
                Ok(EnsuresClause::Err(variant, expr))
            }
            Some(RawToken::Result_) => {
                self.advance();
                if let Some(RawToken::Ident(name)) = self.peek_raw().cloned() {
                    if name == "is" {
                        self.advance();
                        let pattern = self.parse_pattern()?;
                        self.expect_raw(&RawToken::FatArrow)?;
                        let expr = self.parse_expr()?;
                        return Ok(EnsuresClause::ResultIs(pattern, expr));
                    }
                }
                let left = Expr::Ident("result".to_string());
                let full_expr = self.parse_expr_continuation(left)?;
                if self.peek_raw() == Some(&RawToken::FatArrow) {
                    self.advance();
                    let postcondition = self.parse_expr()?;
                    Ok(EnsuresClause::Always(Expr::BinOp(
                        Box::new(full_expr),
                        BinOp::And,
                        Box::new(postcondition),
                    )))
                } else {
                    Ok(EnsuresClause::Always(full_expr))
                }
            }
            _ => {
                let expr = self.parse_expr()?;
                // Check for `condition => postcondition` (conditional ensures without ok/err)
                if self.peek_raw() == Some(&RawToken::FatArrow) {
                    self.advance();
                    let postcondition = self.parse_expr()?;
                    // Wrap as: Always(condition => postcondition) using BinOp or a dedicated node
                    // For now, represent as Always with an implication (condition and postcondition)
                    Ok(EnsuresClause::Always(Expr::BinOp(
                        Box::new(expr),
                        BinOp::And, // TODO: should be implication, using And as placeholder
                        Box::new(postcondition),
                    )))
                } else {
                    Ok(EnsuresClause::Always(expr))
                }
            }
        }
    }

    fn parse_effect_list(&mut self) -> Result<Vec<EffectRef>, ParseError> {
        self.expect_raw(&RawToken::LBracket)?;
        let mut effects = vec![];
        loop {
            if self.peek_raw() == Some(&RawToken::RBracket) {
                break;
            }
            let category = match self.peek_raw() {
                Some(RawToken::Unsafe) => { self.advance(); "unsafe".to_string() }
                Some(RawToken::Async) => { self.advance(); "async".to_string() }
                _ => self.eat_ident_or_keyword()?,
            };
            if self.peek_raw() == Some(&RawToken::Dot) {
                self.advance();
                let operation = self.eat_ident()?;
                effects.push(EffectRef {
                    category,
                    operation,
                });
            } else {
                // Single-word effect like `pure`, `async`, `unsafe`
                effects.push(EffectRef {
                    category: category.clone(),
                    operation: category,
                });
            }
            if self.peek_raw() == Some(&RawToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_raw(&RawToken::RBracket)?;
        Ok(effects)
    }

    fn parse_panics(&mut self) -> Result<PanicsDecl, ParseError> {
        if self.peek_raw() == Some(&RawToken::Never) {
            self.advance();
            Ok(PanicsDecl::Never)
        } else {
            let expr = self.parse_expr()?;
            Ok(PanicsDecl::Condition(expr))
        }
    }

    fn parse_complexity(&mut self) -> Result<ComplexityDecl, ParseError> {
        // Parse complexity bound like `O(n)`, `O(n log n)`, `O(1)`
        // For now, collect tokens until newline/indent/dedent as a string
        let mut bound = String::new();
        let mut where_var = None;
        let mut where_expr = None;

        // Consume O(...)
        loop {
            match self.peek() {
                Token::Newline | Token::Indent | Token::Dedent | Token::Eof => break,
                Token::Raw(RawToken::Where) => {
                    self.advance();
                    where_var = Some(self.eat_ident()?);
                    self.expect_raw(&RawToken::Eq)?;
                    where_expr = Some(self.parse_expr()?);
                    break;
                }
                Token::Raw(tok) => {
                    let text = match tok {
                        RawToken::Ident(s) => s.clone(),
                        RawToken::Int(n) => n.to_string(),
                        RawToken::LParen => "(".to_string(),
                        RawToken::RParen => ")".to_string(),
                        _ => format!("{:?}", tok),
                    };
                    if bound.is_empty() {
                        bound = text;
                    } else {
                        bound = format!("{} {}", bound, text);
                    }
                    self.advance();
                }
            }
        }

        Ok(ComplexityDecl {
            bound: bound.trim().to_string(),
            where_var,
            where_expr,
        })
    }

    fn parse_fails(&mut self) -> Result<Vec<FailsEntry>, ParseError> {
        let mut entries = vec![];
        if self.peek() == &Token::Indent {
            self.advance();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                let variant = self.eat_ident_or_keyword()?;
                self.expect_raw(&RawToken::Colon)?;
                let desc = match self.peek_raw().cloned() {
                    Some(RawToken::String_(s)) => { self.advance(); s }
                    _ => return Err(self.err("expected string after fails variant")),
                };
                entries.push(FailsEntry {
                    variant,
                    description: desc,
                });
            }
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        }
        Ok(entries)
    }

    // ─── Body parsing ───

    fn parse_body_stmts(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = vec![];
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::Dedent | Token::Eof) {
                break;
            }
            // Stop at ) which closes a parent context (e.g., closure inside fn call)
            if matches!(self.peek_raw(), Some(RawToken::RParen)) {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_raw() {
            Some(RawToken::Let) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::Mut) {
                    self.advance();
                }
                let name = self.eat_field_name()?;
                let ty = if self.peek_raw() == Some(&RawToken::Colon) {
                    self.advance();
                    Some(self.parse_type_expr()?)
                } else {
                    None
                };
                self.expect_raw(&RawToken::Eq)?;
                let expr = self.parse_expr()?;
                Ok(Stmt::Let(name, ty, expr))
            }
            _ => {
                let expr = self.parse_expr()?;
                // Check for assignment: `self.x = expr` or `self.x += expr`
                match self.peek_raw() {
                    Some(RawToken::Eq) => {
                        self.advance();
                        let rhs = self.parse_expr()?;
                        Ok(Stmt::Expr(Expr::Assign(Box::new(expr), Box::new(rhs))))
                    }
                    Some(RawToken::PlusEq) => {
                        self.advance();
                        let rhs = self.parse_expr()?;
                        Ok(Stmt::Expr(Expr::CompoundAssign(
                            Box::new(expr),
                            BinOp::Add,
                            Box::new(rhs),
                        )))
                    }
                    Some(RawToken::MinusEq) => {
                        self.advance();
                        let rhs = self.parse_expr()?;
                        Ok(Stmt::Expr(Expr::CompoundAssign(
                            Box::new(expr),
                            BinOp::Sub,
                            Box::new(rhs),
                        )))
                    }
                    _ => Ok(Stmt::Expr(expr)),
                }
            }
        }
    }

    // ─── Expression parsing (precedence climbing) ───

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_expr_continuation(&mut self, left: Expr) -> Result<Expr, ParseError> {
        // Continue parsing binary ops from an already-parsed left side
        let left = self.parse_postfix(left)?;
        // Now handle binary operators (comparison, arithmetic, logical)
        let op = match self.peek_raw() {
            Some(RawToken::EqEq) => Some(BinOp::Eq),
            Some(RawToken::NotEq) => Some(BinOp::NotEq),
            Some(RawToken::Lt) => Some(BinOp::Lt),
            Some(RawToken::LtEq) => Some(BinOp::LtEq),
            Some(RawToken::Gt) => Some(BinOp::Gt),
            Some(RawToken::GtEq) => Some(BinOp::GtEq),
            Some(RawToken::Plus) => Some(BinOp::Add),
            Some(RawToken::Minus) => Some(BinOp::Sub),
            Some(RawToken::Star) => Some(BinOp::Mul),
            Some(RawToken::And) => Some(BinOp::And),
            Some(RawToken::Or) => Some(BinOp::Or),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let right = self.parse_expr()?;
            Ok(Expr::BinOp(Box::new(left), op, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_and_expr()?;
        if self.peek_raw() == Some(&RawToken::Or) {
            self.advance();
            let right = self.parse_or_expr()?;
            Ok(Expr::BinOp(Box::new(left), BinOp::Or, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_comparison()?;
        if self.peek_raw() == Some(&RawToken::And) {
            self.advance();
            let right = self.parse_and_expr()?;
            Ok(Expr::BinOp(Box::new(left), BinOp::And, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_addition()?;
        let op = match self.peek_raw() {
            Some(RawToken::EqEq) => Some(BinOp::Eq),
            Some(RawToken::NotEq) => Some(BinOp::NotEq),
            Some(RawToken::Lt) => Some(BinOp::Lt),
            Some(RawToken::LtEq) => Some(BinOp::LtEq),
            Some(RawToken::Gt) => Some(BinOp::Gt),
            Some(RawToken::GtEq) => Some(BinOp::GtEq),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let right = self.parse_addition()?;
            Ok(Expr::BinOp(Box::new(left), op, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek_raw() {
                Some(RawToken::Plus) => Some(BinOp::Add),
                Some(RawToken::Minus) => Some(BinOp::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplication()?;
                left = Expr::BinOp(Box::new(left), op, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek_raw() {
                Some(RawToken::Star) => Some(BinOp::Mul),
                Some(RawToken::Slash) => Some(BinOp::Div),
                Some(RawToken::Percent) => Some(BinOp::Mod),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::BinOp(Box::new(left), op, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek_raw() {
            Some(RawToken::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)))
            }
            Some(RawToken::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)))
            }
            Some(RawToken::Try) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Try(Box::new(expr)))
            }
            Some(RawToken::Return) => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(Expr::Return(Box::new(expr)))
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_primary()?;
        self.parse_postfix(expr)
    }

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        loop {
            match self.peek_raw() {
                Some(RawToken::Dot) => {
                    self.advance();
                    let field = self.eat_field_name()?;
                    if self.peek_raw() == Some(&RawToken::LParen) {
                        self.advance();
                        let args = self.parse_arg_list()?;
                        self.expect_raw(&RawToken::RParen)?;
                        expr = Expr::Call(
                            Box::new(Expr::Field(Box::new(expr), field)),
                            args,
                        );
                    } else {
                        expr = Expr::Field(Box::new(expr), field);
                    }
                }
                Some(RawToken::LParen) => {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect_raw(&RawToken::RParen)?;
                    expr = Expr::Call(Box::new(expr), args);
                }
                Some(RawToken::LBracket) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    if self.peek_raw() == Some(&RawToken::DotDot) {
                        self.advance();
                        let end = self.parse_expr()?;
                        self.expect_raw(&RawToken::RBracket)?;
                        expr = Expr::Slice(Box::new(expr), Box::new(index), Box::new(end));
                    } else {
                        self.expect_raw(&RawToken::RBracket)?;
                        expr = Expr::Index(Box::new(expr), Box::new(index));
                    }
                }
                Some(RawToken::DotDot) => {
                    self.advance();
                    let end = self.parse_addition()?;
                    expr = Expr::Range(Box::new(expr), Box::new(end));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek_raw().cloned() {
            Some(RawToken::Int(n)) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Some(RawToken::Float(f)) => {
                self.advance();
                Ok(Expr::Float(f))
            }
            Some(RawToken::Char(c)) => {
                self.advance();
                Ok(Expr::String(vec![StringPart::Literal(c.to_string())]))
            }
            Some(RawToken::String_(s)) => {
                self.advance();
                // Parse interpolation segments
                let parts = parse_string_interpolation(&s);
                Ok(Expr::String(parts))
            }
            Some(RawToken::True) => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Some(RawToken::False) => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Some(RawToken::None_) => {
                self.advance();
                Ok(Expr::None_)
            }
            Some(RawToken::SelfLower) => {
                self.advance();
                Ok(Expr::Ident("self".to_string()))
            }
            Some(RawToken::Result_) => {
                self.advance();
                Ok(Expr::Ident("result".to_string()))
            }
            Some(RawToken::Old) => {
                self.advance();
                self.expect_raw(&RawToken::LParen)?;
                let inner = self.parse_expr()?;
                self.expect_raw(&RawToken::RParen)?;
                Ok(Expr::Old(Box::new(inner)))
            }
            Some(RawToken::Some_) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let inner = self.parse_expr()?;
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Expr::EnumVariant(String::new(), "Some".to_string(), vec![inner]))
                } else {
                    Ok(Expr::Ident("Some".to_string()))
                }
            }
            Some(RawToken::Ok_) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let inner = self.parse_expr()?;
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Expr::EnumVariant(String::new(), "Ok".to_string(), vec![inner]))
                } else {
                    Ok(Expr::Ident("Ok".to_string()))
                }
            }
            Some(RawToken::Err_) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let inner = self.parse_expr()?;
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Expr::EnumVariant(String::new(), "Err".to_string(), vec![inner]))
                } else {
                    Ok(Expr::Ident("Err".to_string()))
                }
            }
            Some(RawToken::Ident(name)) => {
                self.advance();
                // Check for indented struct construction: `TypeName\n  field: value`
                if self.peek() == &Token::Indent {
                    // Speculatively check if this is struct construction
                    // (first line after indent should be `ident: expr`)
                    if self.looks_like_struct_body() {
                        return self.parse_struct_construction(name);
                    }
                }
                Ok(Expr::Ident(name))
            }
            Some(RawToken::LParen) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::RParen) {
                    self.advance();
                    return Ok(Expr::Ident("()".to_string())); // unit
                }
                let expr = self.parse_expr()?;
                self.expect_raw(&RawToken::RParen)?;
                Ok(expr)
            }
            Some(RawToken::If) => self.parse_if_expr(),
            Some(RawToken::Match) => self.parse_match_expr(),
            Some(RawToken::For) => self.parse_for_expr(),
            Some(RawToken::While) => self.parse_while_expr(),
            Some(RawToken::Loop) => self.parse_loop_expr(),
            Some(RawToken::Fn) => self.parse_closure_expr(),
            Some(RawToken::Break) => { self.advance(); Ok(Expr::Ident("break".into())) }
            Some(RawToken::Continue) => { self.advance(); Ok(Expr::Ident("continue".into())) }
            Some(RawToken::Unsafe) => {
                self.advance();
                if self.consume_indent() {
                    let stmts = self.parse_body_stmts()?;
                    self.consume_dedent();
                    Ok(Expr::Block(stmts))
                } else {
                    let expr = self.parse_expr()?;
                    Ok(expr)
                }
            }
            _ => Err(self.err(&format!("expected expression, got {:?}", self.peek()))),
        }
    }

    fn looks_like_struct_body(&self) -> bool {
        // Look ahead: Indent, then Ident, then Colon = struct construction
        let mut i = self.pos;
        if i < self.tokens.len() && matches!(self.tokens[i].token, Token::Indent) {
            i += 1;
        }
        // Skip newlines
        while i < self.tokens.len() && matches!(self.tokens[i].token, Token::Newline) {
            i += 1;
        }
        if i + 1 < self.tokens.len() {
            matches!(
                (&self.tokens[i].token, &self.tokens[i + 1].token),
                (Token::Raw(RawToken::Ident(_)), Token::Raw(RawToken::Colon))
            )
        } else {
            false
        }
    }

    fn parse_struct_construction(&mut self, name: String) -> Result<Expr, ParseError> {
        self.consume_indent();
        let mut fields = vec![];
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::Dedent | Token::Eof) {
                break;
            }
            if matches!(self.peek_raw(), Some(RawToken::RParen)) {
                break;
            }
            let fname = self.eat_ident()?;
            self.expect_raw(&RawToken::Colon)?;
            let value = self.parse_expr()?;
            fields.push((fname, value));
        }
        self.consume_dedent();
        Ok(Expr::Construct(name, fields))
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = vec![];
        if self.peek_raw() == Some(&RawToken::RParen) {
            return Ok(args);
        }
        loop {
            if let Some(RawToken::Ident(name)) = self.peek_raw().cloned() {
                let saved = self.pos;
                self.advance();
                if self.peek_raw() == Some(&RawToken::FatArrow) {
                    // Closure: `ident => expr`
                    self.advance();
                    let body = self.parse_expr()?;
                    let param = Param {
                        name,
                        ty: TypeExpr::Named("_".to_string()),
                    };
                    args.push(Expr::Closure(vec![param], Box::new(body)));
                } else if self.peek_raw() == Some(&RawToken::Colon) {
                    // Named argument: `key: value`
                    self.advance();
                    let value = self.parse_expr()?;
                    // Store as a construct field for now (name: value)
                    args.push(Expr::Construct(name, vec![("value".into(), value)]));
                } else {
                    self.pos = saved;
                    args.push(self.parse_expr()?);
                }
            } else {
                args.push(self.parse_expr()?);
            }
            if self.peek_raw() == Some(&RawToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect_raw(&RawToken::If)?;
        let cond = self.parse_expr()?;

        let mut then_body = vec![];
        let mut else_body = None;

        if self.peek() == &Token::Indent {
            self.advance();
            then_body = self.parse_body_stmts()?;
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        } else {
            // Single-line if
            then_body.push(self.parse_stmt()?);
        }

        self.skip_newlines();

        if self.peek_raw() == Some(&RawToken::Else) {
            self.advance();
            if self.peek() == &Token::Indent {
                self.advance();
                let stmts = self.parse_body_stmts()?;
                if self.peek() == &Token::Dedent {
                    self.advance();
                }
                else_body = Some(stmts);
            } else if self.peek_raw() == Some(&RawToken::If) {
                // else if
                let elif = self.parse_if_expr()?;
                else_body = Some(vec![Stmt::Expr(elif)]);
            } else {
                else_body = Some(vec![self.parse_stmt()?]);
            }
        }

        Ok(Expr::If(Box::new(cond), then_body, else_body))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect_raw(&RawToken::Match)?;
        let scrutinee = self.parse_expr()?;
        let mut arms = vec![];

        if self.peek() == &Token::Indent {
            self.advance();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                if self.peek_raw() == Some(&RawToken::Pipe) {
                    self.advance();
                    let pattern = self.parse_pattern()?;
                    // Pattern guard: `| pattern if condition =>`
                    if self.peek_raw() == Some(&RawToken::If) {
                        self.advance();
                        let _guard = self.parse_expr()?;
                        // TODO: store guard in MatchArm
                    }
                    self.expect_raw(&RawToken::FatArrow)?;
                    let mut body = vec![];
                    if self.peek() == &Token::Indent {
                        self.advance();
                        body = self.parse_body_stmts()?;
                        if self.peek() == &Token::Dedent {
                            self.advance();
                        }
                    } else {
                        body.push(self.parse_stmt()?);
                    }
                    arms.push(MatchArm { pattern, body });
                } else {
                    break;
                }
            }
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        }

        Ok(Expr::Match(Box::new(scrutinee), arms))
    }

    fn parse_for_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect_raw(&RawToken::For)?;
        let var = self.eat_ident()?;
        self.expect_raw(&RawToken::In)?;
        let iter = self.parse_expr()?;

        let mut body = vec![];
        let mut invariants = vec![];

        if self.peek() == &Token::Indent {
            self.advance();
            // Check for loop invariant
            self.skip_newlines();
            if self.peek_raw() == Some(&RawToken::Invariant) {
                self.advance();
                invariants = self.parse_indented_exprs()?;
            }
            body = self.parse_body_stmts()?;
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        }

        Ok(Expr::For(var, Box::new(iter), body, invariants))
    }

    fn parse_while_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect_raw(&RawToken::While)?;
        let cond = self.parse_expr()?;
        let mut body = vec![];
        if self.consume_indent() {
            body = self.parse_body_stmts()?;
            self.consume_dedent();
        }
        Ok(Expr::Block(vec![Stmt::Expr(Expr::For(
            "_while".into(),
            Box::new(cond),
            body,
            vec![],
        ))]))
    }

    fn parse_loop_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect_raw(&RawToken::Loop)?;
        let mut body = vec![];
        if self.consume_indent() {
            body = self.parse_body_stmts()?;
            self.consume_dedent();
        }
        Ok(Expr::Block(body))
    }

    fn parse_closure_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect_raw(&RawToken::Fn)?;
        self.expect_raw(&RawToken::LParen)?;
        let params = self.parse_params()?;
        self.expect_raw(&RawToken::RParen)?;

        let _return_type = if self.peek_raw() == Some(&RawToken::Arrow) {
            self.advance();
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        // Body: indented block
        let mut body_stmts = vec![];
        if self.peek() == &Token::Indent {
            self.advance();
            body_stmts = self.parse_body_stmts()?;
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        } else {
            // Single expression on same line
            body_stmts.push(self.parse_stmt()?);
        }

        let body = if body_stmts.len() == 1 {
            match body_stmts.into_iter().next().unwrap() {
                Stmt::Expr(e) => e,
                other => Expr::Block(vec![other]),
            }
        } else {
            Expr::Block(body_stmts)
        };

        Ok(Expr::Closure(params, Box::new(body)))
    }

    // ─── Pattern parsing ───

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_raw().cloned() {
            Some(RawToken::Ident(name)) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Some(RawToken::None_) => {
                self.advance();
                Ok(Pattern::Ident("None".to_string()))
            }
            Some(RawToken::Some_) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Pattern::Constructor("Some".to_string(), vec![inner]))
                } else {
                    Ok(Pattern::Ident("Some".to_string()))
                }
            }
            Some(RawToken::Ok_) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Pattern::Constructor("Ok".to_string(), vec![inner]))
                } else {
                    Ok(Pattern::Ident("Ok".to_string()))
                }
            }
            Some(RawToken::Err_) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let inner = self.parse_pattern()?;
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Pattern::Constructor("Err".to_string(), vec![inner]))
                } else {
                    Ok(Pattern::Ident("Err".to_string()))
                }
            }
            Some(RawToken::Ident(name)) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::LParen) {
                    self.advance();
                    let mut pats = vec![];
                    loop {
                        if self.peek_raw() == Some(&RawToken::RParen) {
                            break;
                        }
                        pats.push(self.parse_pattern()?);
                        if self.peek_raw() == Some(&RawToken::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(Pattern::Constructor(name, pats))
                } else if self.peek_raw() == Some(&RawToken::Dot) {
                    // `GreetError.EmptyMessage`
                    self.advance();
                    let variant = self.eat_ident_or_keyword()?;
                    if self.peek_raw() == Some(&RawToken::LParen) {
                        self.advance();
                        let mut pats = vec![];
                        loop {
                            if self.peek_raw() == Some(&RawToken::RParen) {
                                break;
                            }
                            pats.push(self.parse_pattern()?);
                            if self.peek_raw() == Some(&RawToken::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.expect_raw(&RawToken::RParen)?;
                        Ok(Pattern::Constructor(
                            format!("{}.{}", name, variant),
                            pats,
                        ))
                    } else {
                        Ok(Pattern::Ident(format!("{}.{}", name, variant)))
                    }
                } else {
                    Ok(Pattern::Binding(name))
                }
            }
            Some(RawToken::Int(n)) => {
                self.advance();
                Ok(Pattern::Literal(Expr::Int(n)))
            }
            _ => Err(self.err(&format!("expected pattern, got {:?}", self.peek()))),
        }
    }

    // ─── Type expressions ───

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        match self.peek_raw().cloned() {
            Some(RawToken::LParen) => {
                self.advance();
                if self.peek_raw() == Some(&RawToken::RParen) {
                    self.advance();
                    return Ok(TypeExpr::Unit);
                }
                // Tuple or grouped type
                let first = self.parse_type_expr()?;
                if self.peek_raw() == Some(&RawToken::Comma) {
                    let mut types = vec![first];
                    while self.peek_raw() == Some(&RawToken::Comma) {
                        self.advance();
                        types.push(self.parse_type_expr()?);
                    }
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(TypeExpr::Tuple(types))
                } else {
                    self.expect_raw(&RawToken::RParen)?;
                    Ok(first)
                }
            }
            Some(RawToken::Mut) => {
                self.advance();
                let inner = self.parse_type_expr()?;
                Ok(TypeExpr::Mut(Box::new(inner)))
            }
            Some(RawToken::Owned) => {
                self.advance();
                let inner = self.parse_type_expr()?;
                Ok(TypeExpr::Owned(Box::new(inner)))
            }
            Some(RawToken::Ampersand) => {
                self.advance();
                let inner = self.parse_type_expr()?;
                Ok(TypeExpr::Ref(Box::new(inner)))
            }
            Some(RawToken::Fn) => {
                self.advance();
                self.expect_raw(&RawToken::LParen)?;
                let mut param_types = vec![];
                while self.peek_raw() != Some(&RawToken::RParen) && !self.at_eof() {
                    param_types.push(self.parse_type_expr()?);
                    if self.peek_raw() == Some(&RawToken::Comma) {
                        self.advance();
                    }
                }
                self.expect_raw(&RawToken::RParen)?;
                self.expect_raw(&RawToken::Arrow)?;
                let ret = self.parse_type_expr()?;
                Ok(TypeExpr::Func(param_types, Box::new(ret)))
            }
            _ => {
                let mut name = self.eat_ident_or_keyword()?;
                // Handle associated type paths: `Self.Item`
                while self.peek_raw() == Some(&RawToken::Dot) {
                    self.advance();
                    let segment = self.eat_ident_or_keyword()?;
                    name = format!("{}.{}", name, segment);
                }
                if self.peek_raw() == Some(&RawToken::Lt) {
                    self.advance();
                    let mut args = vec![];
                    loop {
                        args.push(self.parse_type_expr()?);
                        if self.peek_raw() == Some(&RawToken::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.expect_raw(&RawToken::Gt)?;
                    Ok(TypeExpr::Generic(name, args))
                } else {
                    Ok(TypeExpr::Named(name))
                }
            }
        }
    }

    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if self.peek_raw() != Some(&RawToken::Lt) {
            return Ok(vec![]);
        }
        self.advance();
        let mut params = vec![];
        loop {
            let name = self.eat_ident()?;
            let mut bounds = vec![];
            if self.peek_raw() == Some(&RawToken::Colon) {
                self.advance();
                loop {
                    bounds.push(self.eat_ident_or_keyword()?);
                    if self.peek_raw() == Some(&RawToken::Plus) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            params.push(TypeParam { name, bounds });
            if self.peek_raw() == Some(&RawToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_raw(&RawToken::Gt)?;
        Ok(params)
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = vec![];
        if self.peek_raw() == Some(&RawToken::RParen) {
            return Ok(params);
        }
        loop {
            let name = match self.peek_raw() {
                Some(RawToken::SelfLower) => { self.advance(); "self".to_string() }
                _ => self.eat_ident()?,
            };
            self.expect_raw(&RawToken::Colon)?;
            let ty = self.parse_type_expr()?;
            params.push(Param { name, ty });
            if self.peek_raw() == Some(&RawToken::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    // ─── Type declarations ───

    fn parse_type_decl(&mut self, vis: Visibility) -> Result<TypeDecl, ParseError> {
        self.expect_raw(&RawToken::Type)?;
        let name = self.eat_ident_or_keyword()?;
        let type_params = self.parse_type_params()?;

        // Check for alias: `type Name = ...`
        if self.peek_raw() == Some(&RawToken::Eq) {
            self.advance();
            // Check for distinct
            if self.peek_raw() == Some(&RawToken::Distinct) {
                self.advance();
                let ty = self.parse_type_expr()?;
                return Ok(TypeDecl {
                    visibility: vis,
                    name,
                    type_params,
                    kind: TypeDeclKind::Distinct(ty),
                    invariants: vec![],
                });
            }
            let ty = self.parse_type_expr()?;
            return Ok(TypeDecl {
                visibility: vis,
                name,
                type_params,
                kind: TypeDeclKind::Alias(ty),
                invariants: vec![],
            });
        }

        // Struct or enum with indented body
        let mut fields = vec![];
        let mut variants = vec![];
        let mut invariants = vec![];

        if self.peek() == &Token::Indent {
            self.advance();

            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }

                match self.peek_raw() {
                    Some(RawToken::Invariant) => {
                        self.advance();
                        invariants = self.parse_indented_exprs()?;
                    }
                    Some(RawToken::Pipe) => {
                        self.advance();
                        let vname = self.eat_ident_or_keyword()?;
                        let mut vfields = vec![];
                        if self.peek_raw() == Some(&RawToken::LParen) {
                            self.advance();
                            loop {
                                if self.peek_raw() == Some(&RawToken::RParen) {
                                    break;
                                }
                                // Check for named fields: `name: Type` vs just `Type`
                                let saved_pos = self.pos;
                                let maybe_name = self.eat_ident();
                                if maybe_name.is_ok()
                                    && self.peek_raw() == Some(&RawToken::Colon)
                                {
                                    self.advance();
                                    let ty = self.parse_type_expr()?;
                                    vfields.push(Field {
                                        name: maybe_name.unwrap(),
                                        ty,
                                    });
                                } else {
                                    // Reset and parse as unnamed
                                    self.pos = saved_pos;
                                    let ty = self.parse_type_expr()?;
                                    vfields.push(Field {
                                        name: String::new(),
                                        ty,
                                    });
                                }
                                if self.peek_raw() == Some(&RawToken::Comma) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            self.expect_raw(&RawToken::RParen)?;
                        }
                        variants.push(Variant {
                            name: vname,
                            fields: vfields,
                            invariants: vec![],
                        });
                    }
                    _ => {
                        // Struct field: `name: Type`
                        let fname = self.eat_ident()?;
                        self.expect_raw(&RawToken::Colon)?;
                        let ty = self.parse_type_expr()?;
                        fields.push(Field { name: fname, ty });
                    }
                }
            }

            if self.peek() == &Token::Dedent {
                self.advance();
            }
        }

        let kind = if !variants.is_empty() {
            TypeDeclKind::Enum(variants)
        } else {
            TypeDeclKind::Struct(fields)
        };

        Ok(TypeDecl {
            visibility: vis,
            name,
            type_params,
            kind,
            invariants,
        })
    }

    // ─── Const declarations ───

    fn parse_const_decl(&mut self, vis: Visibility) -> Result<ConstDecl, ParseError> {
        self.expect_raw(&RawToken::Const)?;
        let name = self.eat_ident()?;
        self.expect_raw(&RawToken::Colon)?;
        let ty = self.parse_type_expr()?;
        self.expect_raw(&RawToken::Eq)?;
        let value = self.parse_expr()?;
        Ok(ConstDecl {
            visibility: vis,
            name,
            ty,
            value,
        })
    }

    // ─── Trait declarations ───

    fn parse_trait_decl(&mut self, vis: Visibility) -> Result<TraitDecl, ParseError> {
        self.expect_raw(&RawToken::Trait)?;
        let name = self.eat_ident_or_keyword()?;
        let type_params = self.parse_type_params()?;

        let mut associated_types = vec![];
        let mut methods = vec![];

        if self.peek() == &Token::Indent {
            self.advance();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                let item_vis = self.parse_visibility();
                match self.peek_raw() {
                    Some(RawToken::Type) => {
                        self.advance();
                        let tname = self.eat_ident_or_keyword()?;
                        associated_types.push(tname);
                    }
                    Some(RawToken::Fn) => {
                        methods.push(self.parse_fn_decl(item_vis, false)?);
                    }
                    _ => {
                        return Err(self.err("expected fn or type in trait body"));
                    }
                }
            }
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        }

        Ok(TraitDecl {
            visibility: vis,
            name,
            type_params,
            associated_types,
            methods,
        })
    }

    // ─── Impl blocks ───

    fn parse_impl_block(&mut self) -> Result<ImplBlock, ParseError> {
        self.expect_raw(&RawToken::Impl)?;
        let type_params = self.parse_type_params()?;

        // Parse first type name
        let first_name = self.eat_ident_or_keyword()?;
        let first_type = if self.peek_raw() == Some(&RawToken::Lt) {
            self.advance();
            let mut args = vec![];
            loop {
                args.push(self.parse_type_expr()?);
                if self.peek_raw() == Some(&RawToken::Comma) {
                    self.advance();
                } else { break; }
            }
            self.expect_raw(&RawToken::Gt)?;
            TypeExpr::Generic(first_name.clone(), args)
        } else {
            TypeExpr::Named(first_name.clone())
        };

        // Check for `for Type` (trait impl)
        let (trait_name, target_type) = if self.peek_raw() == Some(&RawToken::For) {
            self.advance();
            let target = self.parse_type_expr()?;
            (Some(first_name), target)
        } else {
            (None, first_type)
        };

        // where clauses
        let mut where_clauses = vec![];
        if self.peek_raw() == Some(&RawToken::Where) {
            self.advance();
            loop {
                let param = self.eat_ident()?;
                self.expect_raw(&RawToken::Colon)?;
                let mut bounds = vec![];
                loop {
                    bounds.push(self.eat_ident_or_keyword()?);
                    if self.peek_raw() == Some(&RawToken::Plus) {
                        self.advance();
                    } else { break; }
                }
                where_clauses.push(WhereClause { param, bounds });
                if self.peek_raw() == Some(&RawToken::Comma) {
                    self.advance();
                } else { break; }
            }
        }

        let mut methods = vec![];
        let mut associated_types = vec![];

        if self.peek() == &Token::Indent {
            self.advance();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                let item_vis = self.parse_visibility();
                match self.peek_raw() {
                    Some(RawToken::Fn) => {
                        methods.push(self.parse_fn_decl(item_vis, false)?);
                    }
                    Some(RawToken::Type) => {
                        self.advance();
                        let tname = self.eat_ident_or_keyword()?;
                        self.expect_raw(&RawToken::Eq)?;
                        let ty = self.parse_type_expr()?;
                        associated_types.push((tname, ty));
                    }
                    _ => break,
                }
            }
            if self.peek() == &Token::Dedent {
                self.advance();
            }
        }

        Ok(ImplBlock {
            type_params,
            trait_name,
            target_type,
            where_clauses,
            methods,
            associated_types,
        })
    }

    // ─── State machine (stub) ───

    fn parse_state_machine(&mut self) -> Result<StateMachineDecl, ParseError> {
        self.expect_raw(&RawToken::StateMachine)?;
        let name = self.eat_ident()?;

        // Optional `for Type.field`
        if self.peek_raw() == Some(&RawToken::For) {
            self.advance();
            let _target = self.parse_type_expr()?;
        }

        let mut doc = None;
        let mut states = vec![];
        let mut transitions = vec![];
        let mut initial = String::new();
        let mut terminal = vec![];

        if self.consume_indent() {
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Dedent | Token::Eof) {
                    break;
                }
                match self.peek_raw() {
                    Some(RawToken::Doc) => {
                        self.advance();
                        if let Some(RawToken::String_(s)) = self.peek_raw().cloned() {
                            self.advance();
                            doc = Some(s);
                        }
                    }
                    Some(RawToken::States) => {
                        self.advance();
                        if self.peek_raw() == Some(&RawToken::LBracket) {
                            self.advance();
                            loop {
                                if self.peek_raw() == Some(&RawToken::RBracket) { break; }
                                states.push(self.eat_ident_or_keyword()?);
                                if self.peek_raw() == Some(&RawToken::Comma) {
                                    self.advance();
                                } else { break; }
                            }
                            self.expect_raw(&RawToken::RBracket)?;
                        } else if self.consume_indent() {
                            loop {
                                self.skip_newlines();
                                if matches!(self.peek(), Token::Dedent | Token::Eof) { break; }
                                states.push(self.eat_ident_or_keyword()?);
                            }
                            self.consume_dedent();
                        }
                    }
                    Some(RawToken::Transitions) => {
                        self.advance();
                        if self.consume_indent() {
                            loop {
                                self.skip_newlines();
                                if matches!(self.peek(), Token::Dedent | Token::Eof) { break; }
                                let from = self.eat_ident_or_keyword()?;
                                // `--event-->` tokenized as Minus Minus Ident Minus Arrow
                                self.expect_raw(&RawToken::Minus)?;
                                self.expect_raw(&RawToken::Minus)?;
                                let event = self.eat_ident()?;
                                self.expect_raw(&RawToken::Minus)?;
                                self.expect_raw(&RawToken::Arrow)?;
                                let to = self.eat_ident_or_keyword()?;
                                transitions.push(TransitionDecl { from, event, to });
                            }
                            self.consume_dedent();
                        }
                    }
                    Some(RawToken::Initial) => {
                        self.advance();
                        initial = self.eat_ident_or_keyword()?;
                    }
                    Some(RawToken::Terminal) => {
                        self.advance();
                        if self.peek_raw() == Some(&RawToken::LBracket) {
                            self.advance();
                            loop {
                                if self.peek_raw() == Some(&RawToken::RBracket) { break; }
                                terminal.push(self.eat_ident_or_keyword()?);
                                if self.peek_raw() == Some(&RawToken::Comma) {
                                    self.advance();
                                } else { break; }
                            }
                            self.expect_raw(&RawToken::RBracket)?;
                        } else {
                            terminal.push(self.eat_ident_or_keyword()?);
                        }
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
            self.consume_dedent();
        }

        Ok(StateMachineDecl {
            name,
            doc,
            states,
            transitions,
            initial,
            terminal,
        })
    }
}

/// Parse string interpolation: `"{self.name}: {message}"` into parts.
fn parse_string_interpolation(s: &str) -> Vec<StringPart> {
    let mut parts = vec![];
    let mut current = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            if !current.is_empty() {
                parts.push(StringPart::Literal(std::mem::take(&mut current)));
            }
            let mut expr_str = String::new();
            let mut depth = 1;
            for c in chars.by_ref() {
                if c == '{' { depth += 1; }
                if c == '}' { depth -= 1; if depth == 0 { break; } }
                expr_str.push(c);
            }
            // For simplicity, store interpolations as field-access chains
            parts.push(StringPart::Interpolation(Expr::Ident(expr_str)));
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        parts.push(StringPart::Literal(current));
    }

    if parts.is_empty() {
        parts.push(StringPart::Literal(String::new()));
    }

    parts
}
