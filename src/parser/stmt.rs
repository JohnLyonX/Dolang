// Statement parser - handles all statement parsing.
use crate::ast::{
    AssignStmt, BinaryExpr, BreakStmt, ConstDeclStmt, ContinueStmt, ExitStmt, Expr, FnDeclStmt, ForInStmt, ForStmt,
    IfBranch, IfStmt, LoopStmt, ReturnStmt, Span, Stmt, VarDeclStmt, WhileStmt,
};
use crate::error::Error;
use crate::parser::parse_expr_tokens;
use crate::token::{Token, Type};

/// Top-level statement parser — token-stream based, supports blocks.
pub struct StmtParser<'a> {
    pub tokens: &'a [Token],
    pub pos: usize,
    #[allow(dead_code)]
    pub src: &'a str,
}

impl<'a> StmtParser<'a> {
    pub fn new(tokens: &'a [Token], src: &'a str) -> Self {
        Self { tokens, pos: 0, src }
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub fn peek_next(&self) -> Option<&Token> {
        if self.pos + 1 < self.tokens.len() {
            Some(&self.tokens[self.pos + 1])
        } else {
            None
        }
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].typ == Type::Eof
    }

    pub fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    pub fn skip_semis(&mut self) {
        while !self.at_end() && self.peek().typ == Type::Semicolon {
            self.pos += 1;
        }
    }

    /// Expect a token of the given type, consume it, or return an error.
    pub fn expect(&mut self, typ: Type) -> Result<&Token, Error> {
        if self.at_end() || self.peek().typ != typ {
            return Err(Error::InvalidStatement(None));
        }
        Ok(self.advance())
    }

    /// Parse a sequence of statements until EOF (or a closing `}`).
    pub fn parse_stmts(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut stmts = Vec::new();
        loop {
            self.skip_semis();
            if self.at_end() || self.peek().typ == Type::RBrace {
                break;
            }
            let stmt = self.parse_one_stmt()?;
            if let Some(s) = stmt {
                stmts.push(s);
            }
        }
        Ok(stmts)
    }

    /// Parse a `{ ... }` block and return the contained statements.
    pub fn parse_block(&mut self) -> Result<Vec<Stmt>, Error> {
        // Consume `{`
        self.expect(Type::LBrace)?;
        let stmts = self.parse_stmts()?;
        // Consume `}`
        self.expect(Type::RBrace)?;
        Ok(stmts)
    }

    /// Collect tokens up to the next `;` or `}` (not inside braces).
    pub fn collect_until_semi(&mut self) -> Vec<Token> {
        let mut toks = Vec::new();
        let mut brace_depth = 0;
        while !self.at_end() {
            let typ = self.peek().typ.clone();
            if brace_depth == 0 && (typ == Type::Semicolon || typ == Type::RBrace || typ == Type::Eof) {
                break;
            }
            if typ == Type::LBrace {
                brace_depth += 1;
            } else if typ == Type::RBrace {
                brace_depth -= 1;
            }
            toks.push(self.advance().clone());
        }
        toks
    }

    /// Parse a single top-level statement.
    pub fn parse_one_stmt(&mut self) -> Result<Option<Stmt>, Error> {
        let typ = self.peek().typ.clone();

        // --- $if statement ---
        if typ == Type::If {
            return self.parse_if_stmt().map(Some);
        }

        // --- $while statement ---
        if typ == Type::While {
            return self.parse_while_stmt().map(Some);
        }

        // --- $loop statement ---
        if typ == Type::Loop {
            return self.parse_loop_stmt().map(Some);
        }

        // --- $for statement ---
        if typ == Type::For {
            return self.parse_for_stmt().map(Some);
        }

        // --- $fn function declaration ---
        // Only parse as named function if followed by an identifier
        if typ == Type::Fn {
            if let Some(next_pos) = self.peek_next() {
                if next_pos.typ == Type::Ident {
                    return self.parse_fn_decl().map(Some);
                }
                // Otherwise, let it be parsed as an expression (anonymous function)
            }
        }

        // --- $# return ---
        if typ == Type::Return {
            let start = self.peek().pos;
            self.advance(); // consume $#
            if !self.at_end() && self.peek().typ == Type::Semicolon {
                self.skip_semis();
                return Ok(Some(Stmt::Return(ReturnStmt { span: Span::from_token(start), value: None })));
            }
            let expr_toks = self.collect_until_semi();
            if expr_toks.is_empty() {
                self.skip_semis();
                return Ok(Some(Stmt::Return(ReturnStmt { span: Span::from_token(start), value: None })));
            }
            let expr = parse_expr_tokens(&expr_toks)?;
            self.skip_semis();
            return Ok(Some(Stmt::Return(ReturnStmt { span: Span::from_token(start), value: Some(expr) })));
        }

        // --- $break ---
        if typ == Type::Break {
            let start = self.peek().pos;
            self.advance();
            self.skip_semis();
            return Ok(Some(Stmt::Break(BreakStmt { span: Span::from_token(start) })));
        }

        // --- $continue ---
        if typ == Type::Continue {
            let start = self.peek().pos;
            self.advance();
            self.skip_semis();
            return Ok(Some(Stmt::Continue(ContinueStmt { span: Span::from_token(start) })));
        }

        // --- Exit ---
        if typ == Type::Ident
            && matches!(
                self.peek().literal.as_str(),
                "exit" | "exit()" | "quit"
            )
        {
            let start = self.peek().pos;
            self.advance();
            self.skip_semis();
            return Ok(Some(Stmt::Exit(ExitStmt { span: Span::from_token(start) })));
        }

        // --- $>> print ---
        if typ == Type::Print {
            let start = self.peek().pos;
            self.advance(); // consume $>>
            let expr_toks = self.collect_until_semi();
            if expr_toks.is_empty() {
                return Err(Error::InvalidExpression(None));
            }
            let expr = parse_expr_tokens(&expr_toks)?;
            self.skip_semis();
            return Ok(Some(Stmt::Print(crate::ast::PrintStmt { span: Span::from_token(start), value: expr })));
        }

        // --- $ varDecl ---
        if typ == Type::VarDecl {
            let start = self.peek().pos;
            self.advance(); // consume $
            if self.at_end() || self.peek().typ != Type::Ident {
                return Err(Error::InvalidStatement(None));
            }
            let name = self.advance().literal.clone();
            // expect <&
            if self.at_end() || self.peek().typ != Type::Assign {
                return Err(Error::InvalidStatement(None));
            }
            self.advance(); // consume <&
            let expr_toks = self.collect_until_semi();
            let expr = parse_expr_tokens(&expr_toks)?;
            self.skip_semis();
            return Ok(Some(Stmt::VarDecl(VarDeclStmt { span: Span::from_token(start), name, value: expr })));
        }

        // --- $@ constDecl ---
        if typ == Type::ConstDecl {
            let start = self.peek().pos;
            self.advance(); // consume $@
            if self.at_end() || self.peek().typ != Type::Ident {
                return Err(Error::InvalidStatement(None));
            }
            let name = self.advance().literal.clone();
            if self.at_end() || self.peek().typ != Type::Assign {
                return Err(Error::InvalidStatement(None));
            }
            self.advance(); // consume <&
            let expr_toks = self.collect_until_semi();
            let expr = parse_expr_tokens(&expr_toks)?;
            self.skip_semis();
            return Ok(Some(Stmt::ConstDecl(ConstDeclStmt { span: Span::from_token(start), name, value: expr })));
        }

        // --- Function call as statement: name(args); ---
        if typ == Type::Ident {
            // Look ahead: if next token is '(' then it's a function call
            if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1].typ == Type::LParen {
                let saved_pos = self.pos;
                let stmt_toks = self.collect_until_semi();
                self.skip_semis();
                // Try parsing as function call expression
                match parse_expr_tokens(&stmt_toks) {
                    Ok(expr) => return Ok(Some(Stmt::ExprStmt(expr))),
                    Err(_) => {
                        // Restore and fall through
                        self.pos = saved_pos;
                    }
                }
            }
        }

        // --- Assign: ident <& expr  OR  var-read <& expr ---
        // --- Compound assign: ident += expr, -= expr, *= expr, /= expr, %= expr ---
        let saved_pos = self.pos;
        let stmt_toks = self.collect_until_semi();
        self.skip_semis();

        // First check for compound assignment (+=, -=, *=, /=, %=)
        if let Some((assign_idx, op)) = find_compound_assign(&stmt_toks) {
            let start = stmt_toks.first().map(|t| t.pos).unwrap_or(0);
            let name_expr = parse_expr_tokens(&stmt_toks[..assign_idx])?;
            let value_expr = parse_expr_tokens(&stmt_toks[assign_idx + 1..])?;
            // Transform: i += j  =>  Assign(i, BinaryExpr(i, Plus, j))
            let binary_expr = BinaryExpr {
                span: Span::from_token(start),
                left: name_expr.clone(),
                right: value_expr,
                op,
            };
            return Ok(Some(Stmt::Assign(AssignStmt {
                span: Span::from_token(start),
                name: name_expr,
                value: Box::new(Expr::Binary(binary_expr)),
            })));
        }

        // Then check for regular assignment (= or <&)
        if let Some(assign_idx) = find_token(&stmt_toks, Type::Assign) {
            let start = stmt_toks.first().map(|t| t.pos).unwrap_or(0);
            let name_expr = parse_expr_tokens(&stmt_toks[..assign_idx])?;
            let value_expr = parse_expr_tokens(&stmt_toks[assign_idx + 1..])?;
            return Ok(Some(Stmt::Assign(AssignStmt {
                span: Span::from_token(start),
                name: name_expr,
                value: value_expr,
            })));
        }

        // Try parsing as expression statement (e.g. bare function call)
        if !stmt_toks.is_empty() {
            if let Ok(expr) = parse_expr_tokens(&stmt_toks) {
                return Ok(Some(Stmt::ExprStmt(expr)));
            }
        }

        let _ = saved_pos;
        Err(Error::InvalidStatement(None))
    }

    /// Parse a full $if ... { } [$elif ... { }]* [$else { }] statement.
    pub fn parse_if_stmt(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        let mut branches: Vec<IfBranch> = Vec::new();
        let mut else_body: Vec<Stmt> = Vec::new();

        // $if branch
        self.advance(); // consume `$if`
        let cond_toks = self.collect_until_lbrace();
        if cond_toks.is_empty() {
            return Err(Error::InvalidExpression(None));
        }
        let cond = parse_expr_tokens(&cond_toks)?;
        let body = self.parse_block()?;
        branches.push(IfBranch { span: Span::from_token(0), condition: cond, body });

        // Zero or more $elif branches
        loop {
            self.skip_semis();
            if !self.at_end() && self.peek().typ == Type::Elif {
                self.advance(); // consume `$elif`
                let cond_toks = self.collect_until_lbrace();
                if cond_toks.is_empty() {
                    return Err(Error::InvalidExpression(None));
                }
                let cond = parse_expr_tokens(&cond_toks)?;
                let body = self.parse_block()?;
                branches.push(IfBranch { span: Span::from_token(0), condition: cond, body });
            } else {
                break;
            }
        }

        // Optional $else
        self.skip_semis();
        if !self.at_end() && self.peek().typ == Type::Else {
            self.advance(); // consume `$else`
            else_body = self.parse_block()?;
        }

        Ok(Stmt::If(IfStmt { span: Span::from_token(start), branches, else_body }))
    }

    /// Collect tokens up to the next `{` (not consuming it).
    pub fn collect_until_lbrace(&mut self) -> Vec<Token> {
        let mut toks = Vec::new();
        while !self.at_end()
            && self.peek().typ != Type::LBrace
            && self.peek().typ != Type::Eof
        {
            toks.push(self.advance().clone());
        }
        toks
    }

    /// Parse: $while condition { body }
    pub fn parse_while_stmt(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        self.advance(); // consume `$while`
        let cond_toks = self.collect_until_lbrace();
        if cond_toks.is_empty() {
            return Err(Error::InvalidExpression(None));
        }
        let condition = parse_expr_tokens(&cond_toks)?;
        let body = self.parse_block()?;
        Ok(Stmt::While(WhileStmt { span: Span::from_token(start), condition, body }))
    }

    /// Parse: $loop { body }
    pub fn parse_loop_stmt(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        self.advance(); // consume `$loop`
        let body = self.parse_block()?;
        Ok(Stmt::Loop(LoopStmt { span: Span::from_token(start), body }))
    }

    /// Parse: $for [init;] [condition;] [update] { body }
    /// Or: $for item in iterable { body }
    pub fn parse_for_stmt(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        self.advance(); // consume `$for`

        // Collect everything between `$for` and `{`
        let header_toks = self.collect_until_lbrace();

        // Check if this is a for-in loop (contains $in keyword)
        let has_in = header_toks.iter().any(|t| t.typ == Type::In);

        if has_in {
            // Parse: $for item in iterable { body }
            // Split by $in
            let parts: Vec<&[Token]> = header_toks
                .split(|t| t.typ == Type::In)
                .collect();

            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "invalid for-in syntax, expected: $for item in iterable { ... }".to_string(),
                    line: 1,
                    column: 1,
                    found: None,
                    expected: Some("variable and iterable".to_string()),
                }));
            }

            // First part should be the variable name
            let var_toks = parts[0];
            if var_toks.len() != 1 || var_toks[0].typ != Type::Ident {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "expected loop variable name".to_string(),
                    line: 1,
                    column: 1,
                    found: Some(format!("{:?}", var_toks.first().map(|t| &t.typ))),
                    expected: Some("identifier".to_string()),
                }));
            }
            let var = var_toks[0].literal.clone();

            // Second part should be the iterable expression
            let iterable = parse_expr_tokens(parts[1])?;

            let body = self.parse_block()?;

            return Ok(Stmt::ForIn(ForInStmt {
                span: Span::from_token(start),
                var,
                iterable,
                body,
            }));
        }

        // Traditional for loop: $for init; condition; update { body }
        // Split header by `;` into (up to) 3 parts
        let parts = split_by_semi(&header_toks);

        let (init, condition, update) = match parts.len() {
            0 => (None, None, None),
            1 => {
                let cond = if parts[0].is_empty() {
                    None
                } else {
                    Some(parse_expr_tokens(&parts[0])?)
                };
                (None, cond, None)
            }
            2 => {
                let init = parse_init_or_update(&parts[0])?;
                let cond = if parts[1].is_empty() {
                    None
                } else {
                    Some(parse_expr_tokens(&parts[1])?)
                };
                (init, cond, None)
            }
            _ => {
                let init = parse_init_or_update(&parts[0])?;
                let cond = if parts[1].is_empty() {
                    None
                } else {
                    Some(parse_expr_tokens(&parts[1])?)
                };
                let update = parse_init_or_update(&parts[2])?;
                (init, cond, update)
            }
        };

        let body = self.parse_block()?;
        Ok(Stmt::For(ForStmt {
            span: Span::from_token(start),
            init,
            condition,
            update,
            body,
        }))
    }

    /// Parse: $fn name(param1, param2) -> return_type { body }
    pub fn parse_fn_decl(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        self.advance(); // consume `$fn`

        // Expect function name
        if self.at_end() || self.peek().typ != Type::Ident {
            return Err(Error::InvalidStatement(None));
        }
        let name = self.advance().literal.clone();

        // Expect '('
        self.expect(Type::LParen)?;

        // Parse parameter names
        let mut params: Vec<String> = Vec::new();
        if !self.at_end() && self.peek().typ != Type::RParen {
            loop {
                if self.at_end() || self.peek().typ != Type::Ident {
                    return Err(Error::InvalidStatement(None));
                }
                params.push(self.advance().literal.clone());
                if self.at_end() || self.peek().typ != Type::Comma {
                    break;
                }
                self.advance(); // consume ','
            }
        }

        // Expect ')'
        self.expect(Type::RParen)?;

        // Optional: -> return_type
        let return_type = if !self.at_end() && self.peek().typ == Type::Arrow {
            self.advance(); // consume '->'
            if self.at_end() || self.peek().typ != Type::Ident {
                return Err(Error::InvalidStatement(None));
            }
            Some(self.advance().literal.clone())
        } else {
            None
        };

        // Parse body block
        let body = self.parse_block()?;

        Ok(Stmt::FnDecl(FnDeclStmt {
            span: Span::from_token(start),
            name,
            params,
            return_type,
            body,
        }))
    }
}

/// Find a token of a specific type in a slice.
fn find_token(toks: &[Token], t: Type) -> Option<usize> {
    toks.iter().position(|tok| tok.typ == t)
}

/// Find a compound assignment token and return its index and the corresponding binary operator.
fn find_compound_assign(toks: &[Token]) -> Option<(usize, Type)> {
    for (i, tok) in toks.iter().enumerate() {
        let binary_op = match tok.typ {
            Type::PlusAssign => Some(Type::Plus),
            Type::MinusAssign => Some(Type::Minus),
            Type::MulAssign => Some(Type::Mul),
            Type::DivAssign => Some(Type::Div),
            Type::ModAssign => Some(Type::Mod),
            _ => None,
        };
        if let Some(op) = binary_op {
            return Some((i, op));
        }
    }
    None
}

/// Split a token slice by Semicolon into parts.
fn split_by_semi(toks: &[Token]) -> Vec<Vec<Token>> {
    let mut parts: Vec<Vec<Token>> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    for tok in toks {
        if tok.typ == Type::Semicolon {
            parts.push(current.clone());
            current.clear();
        } else {
            current.push(tok.clone());
        }
    }
    parts.push(current);
    parts
}

/// Parse a token slice as a single init/update statement.
fn parse_init_or_update(toks: &[Token]) -> Result<Option<Box<Stmt>>, Error> {
    if toks.is_empty() {
        return Ok(None);
    }

    // VarDecl: starts with $
    if toks[0].typ == Type::VarDecl {
        if toks.len() < 3 || toks[1].typ != Type::Ident || toks[2].typ != Type::Assign {
            return Err(Error::InvalidStatement(None));
        }
        let start = toks[0].pos;
        let name = toks[1].literal.clone();
        let expr = parse_expr_tokens(&toks[3..])?;
        return Ok(Some(Box::new(Stmt::VarDecl(VarDeclStmt { span: Span::from_token(start), name, value: expr }))));
    }

    // Assign: look for <&
    if let Some(idx) = find_token(toks, Type::Assign) {
        let start = toks.first().map(|t| t.pos).unwrap_or(0);
        let name_expr = parse_expr_tokens(&toks[..idx])?;
        let value_expr = parse_expr_tokens(&toks[idx + 1..])?;
        return Ok(Some(Box::new(Stmt::Assign(AssignStmt {
            span: Span::from_token(start),
            name: name_expr,
            value: value_expr,
        }))));
    }

    Err(Error::InvalidStatement(None))
}
