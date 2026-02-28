// Expression parser - handles all expression parsing.
use std::borrow::Cow;

use crate::ast::{
    BinaryExpr, BoolLiteral, CharLiteral, Expr, FnCallExpr, IndexAccess, ListLiteral, MapLiteral, MethodCall, NumberLiteral,
    Span, Spanned, StringLiteral, UnaryExpr, VarLookup,
};
use crate::error::{Error, ParseError};
use crate::parser::calc_line_col;
use crate::token::{Token, Type};

/// Parse a token slice into an expression.
pub fn parse_expr_tokens(toks: &[Token]) -> Result<Box<Expr>, Error> {
    if toks.is_empty() {
        return Err(Error::Parse(ParseError {
            message: "empty expression".to_string(),
            line: 1,
            column: 1,
            found: None,
            expected: Some("expression".to_string()),
        }));
    }

    let mut parser = ExprParser {
        tokens: toks,
        pos: 0,
        src: "",
    };
    let expr = parser.parse_expr()?;
    if parser.pos != toks.len() {
        let tok = parser.tokens.get(parser.pos);
        return Err(Error::Parse(ParseError {
            message: "unexpected token after expression".to_string(),
            line: 1,
            column: 1,
            found: tok.map(|t| format!("{:?}", t.typ)),
            expected: None,
        }));
    }
    Ok(expr)
}

pub struct ExprParser<'a> {
    pub tokens: &'a [Token],
    pub pos: usize,
    pub src: &'a str,
}

impl<'a> ExprParser<'a> {
    /// Create a parse error with context
    pub fn error(&self, message: &str) -> Error {
        let tok = self.tokens.get(self.pos).or_else(|| self.tokens.last());
        let (line, col) = if let Some(t) = tok {
            calc_line_col(self.src, t.pos)
        } else {
            (1, 1)
        };

        let found = tok.map(|t| format!("{:?} {}", t.typ, t.literal));

        Error::Parse(ParseError {
            message: message.to_string(),
            line,
            column: col,
            found,
            expected: None,
        })
    }

    /// Create a parse error with expected token info
    pub fn error_expected(&self, message: &str, expected: &str) -> Error {
        let tok = self.tokens.get(self.pos).or_else(|| self.tokens.last());
        let (line, col) = if let Some(t) = tok {
            calc_line_col(self.src, t.pos)
        } else {
            (1, 1)
        };

        let found = tok.map(|t| format!("{:?} {}", t.typ, t.literal));

        Error::Parse(ParseError {
            message: message.to_string(),
            line,
            column: col,
            found,
            expected: Some(expected.to_string()),
        })
    }

    pub fn parse_expr(&mut self) -> Result<Box<Expr>, Error> {
        self.parse_or_expr()
    }

    pub fn parse_or_expr(&mut self) -> Result<Box<Expr>, Error> {
        let mut left = self.parse_and_expr()?;

        while self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::Or {
            let _op_pos = self.tokens[self.pos].pos;
            self.pos += 1;
            let right = self.parse_and_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Box::new(Expr::Binary(BinaryExpr {
                span,
                left,
                right,
                op: Type::Or,
            }));
        }
        Ok(left)
    }

    pub fn parse_and_expr(&mut self) -> Result<Box<Expr>, Error> {
        let mut left = self.parse_equality_expr()?;

        while self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::And {
            let _op_pos = self.tokens[self.pos].pos;
            self.pos += 1;
            let right = self.parse_equality_expr()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Box::new(Expr::Binary(BinaryExpr {
                span,
                left,
                right,
                op: Type::And,
            }));
        }
        Ok(left)
    }

    pub fn parse_equality_expr(&mut self) -> Result<Box<Expr>, Error> {
        let mut left = self.parse_comparison_expr()?;

        while self.pos < self.tokens.len() {
            let typ = &self.tokens[self.pos].typ;
            if *typ == Type::Eq || *typ == Type::Ne {
                let _op_pos = self.tokens[self.pos].pos;
                self.pos += 1;
                let right = self.parse_comparison_expr()?;
                let span = Span::new(left.span().start, right.span().end);
                left = Box::new(Expr::Binary(BinaryExpr {
                    span,
                    left,
                    right,
                    op: typ.clone(),
                }));
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_comparison_expr(&mut self) -> Result<Box<Expr>, Error> {
        let mut left = self.parse_additive_expr()?;

        while self.pos < self.tokens.len() {
            let typ = &self.tokens[self.pos].typ;
            if *typ == Type::Gt || *typ == Type::Lt || *typ == Type::Gte || *typ == Type::Lte {
                let _op_pos = self.tokens[self.pos].pos;
                self.pos += 1;
                let right = self.parse_additive_expr()?;
                let span = Span::new(left.span().start, right.span().end);
                left = Box::new(Expr::Binary(BinaryExpr {
                    span,
                    left,
                    right,
                    op: typ.clone(),
                }));
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_additive_expr(&mut self) -> Result<Box<Expr>, Error> {
        let mut left = self.parse_multiplicative_expr()?;

        while self.pos < self.tokens.len() {
            let typ = &self.tokens[self.pos].typ;
            if *typ == Type::Plus || *typ == Type::Minus {
                let _op_pos = self.tokens[self.pos].pos;
                self.pos += 1;
                let right = self.parse_multiplicative_expr()?;
                let span = Span::new(left.span().start, right.span().end);
                left = Box::new(Expr::Binary(BinaryExpr {
                    span,
                    left,
                    right,
                    op: typ.clone(),
                }));
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_multiplicative_expr(&mut self) -> Result<Box<Expr>, Error> {
        let mut left = self.parse_unary_expr()?;

        while self.pos < self.tokens.len() {
            let typ = &self.tokens[self.pos].typ;
            if *typ == Type::Mul || *typ == Type::Div || *typ == Type::Mod {
                let _op_pos = self.tokens[self.pos].pos;
                self.pos += 1;
                let right = self.parse_unary_expr()?;
                let span = Span::new(left.span().start, right.span().end);
                left = Box::new(Expr::Binary(BinaryExpr {
                    span,
                    left,
                    right,
                    op: typ.clone(),
                }));
            } else {
                break;
            }
        }
        Ok(left)
    }

    pub fn parse_unary_expr(&mut self) -> Result<Box<Expr>, Error> {
        if self.pos < self.tokens.len() {
            let typ = &self.tokens[self.pos].typ;
            if *typ == Type::Not || *typ == Type::Minus {
                let _op_pos = self.tokens[self.pos].pos;
                self.pos += 1;
                let right = self.parse_unary_expr()?;
                let span = Span::new(_op_pos, right.span().end);
                return Ok(Box::new(Expr::Unary(UnaryExpr {
                    span,
                    op: typ.clone(),
                    right,
                })));
            }
        }
        // Parse primary expression and then handle postfix index access
        let mut expr = self.parse_primary()?;

        // Handle chained index access: expr[expr][expr]...
        while self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::LBracket {
            let start = self.tokens[self.pos].pos;
            self.pos += 1; // consume '['

            let index = self.parse_expr()?;

            if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::RBracket {
                return Err(self.error_expected("expected closing bracket in index access", "]"));
            }
            self.pos += 1; // consume ']'

            expr = Box::new(Expr::IndexAccess(IndexAccess {
                span: Span::new(start, self.tokens[self.pos - 1].pos + 1),
                object: expr,
                index,
            }));
        }

        // Handle chained method calls: expr.method(args).method(args)...
        while self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::Dot {
            let start = self.tokens[self.pos].pos;
            self.pos += 1; // consume '.'

            // Expect method name
            if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::Ident {
                return Err(self.error_expected("expected method name after '.'", "identifier"));
            }
            let method = self.tokens[self.pos].literal.clone();
            self.pos += 1; // consume method name

            // Check for parentheses with arguments
            let mut args: Vec<Expr> = Vec::new();
            if self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::LParen {
                self.pos += 1; // consume '('

                // Parse arguments
                if self.pos < self.tokens.len() && self.tokens[self.pos].typ != Type::RParen {
                    loop {
                        let arg = self.parse_expr()?;
                        args.push(*arg);
                        if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::Comma {
                            break;
                        }
                        self.pos += 1; // consume ','
                    }
                }

                if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::RParen {
                    return Err(self.error_expected("expected closing parenthesis in method call", ")"));
                }
                self.pos += 1; // consume ')'
            }

            expr = Box::new(Expr::MethodCall(MethodCall {
                span: Span::new(start, self.tokens[self.pos - 1].pos + 1),
                object: expr,
                method,
                args,
            }));
        }

        Ok(expr)
    }

    pub fn parse_primary(&mut self) -> Result<Box<Expr>, Error> {
        if self.pos >= self.tokens.len() {
            return Err(self.error("unexpected end of input"));
        }
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;

        match tok.typ {
            Type::Number => Ok(Box::new(Expr::Number(NumberLiteral { span: Span::from_token(tok.pos), value: Cow::Owned(tok.literal) }))),
            Type::Char => Ok(Box::new(Expr::Char(CharLiteral { span: Span::from_token(tok.pos), value: Cow::Owned(tok.literal) }))),
            Type::Bool => {
                let value = tok.literal == "true";
                Ok(Box::new(Expr::Bool(BoolLiteral { span: Span::from_token(tok.pos), value })))
            }
            Type::String => Ok(Box::new(Expr::StringLiteral(StringLiteral { span: Span::from_token(tok.pos), value: Cow::Owned(tok.literal) }))),
            // $fn: anonymous function literal
            Type::Fn => self.parse_fn_literal(),
            // Ident: check if followed by '(' for function call
            Type::Ident => {
                if self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::LParen {
                    // Function call: name(arg1, arg2, ...)
                    self.pos += 1; // consume '('
                    let mut args: Vec<Expr> = Vec::new();
                    if self.pos < self.tokens.len() && self.tokens[self.pos].typ != Type::RParen {
                        loop {
                            let arg = self.parse_expr()?;
                            args.push(*arg);
                            if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::Comma {
                                break;
                            }
                            self.pos += 1; // consume ','
                        }
                    }
                    if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::RParen {
                        return Err(self.error_expected("expected closing parenthesis in function call", ")"));
                    }
                    self.pos += 1; // consume ')'
                    Ok(Box::new(Expr::FnCall(FnCallExpr {
                        span: Span::from_token(tok.pos),
                        name: tok.literal,
                        args,
                    })))
                } else {
                    Ok(Box::new(Expr::VarLookup(VarLookup { span: Span::from_token(tok.pos), name: Cow::Owned(tok.literal) })))
                }
            }
            // Parenthesized expression
            Type::LParen => {
                let expr = self.parse_expr()?;
                if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::RParen {
                    return Err(self.error_expected("expected closing parenthesis", ")"));
                }
                self.pos += 1; // consume ')'
                Ok(expr)
            }
            // List literal: [expr, expr, ...]
            Type::LBracket => {
                let start = tok.pos;
                let mut elements: Vec<Expr> = Vec::new();

                // Check for empty list: []
                if self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::RBracket {
                    self.pos += 1; // consume ']'
                    return Ok(Box::new(Expr::ListLiteral(ListLiteral {
                        span: Span::new(start, self.tokens[self.pos - 1].pos + 1),
                        elements,
                    })));
                }

                // Parse elements
                loop {
                    let elem = self.parse_expr()?;
                    elements.push(*elem);
                    if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::Comma {
                        break;
                    }
                    self.pos += 1; // consume ','
                }

                if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::RBracket {
                    return Err(self.error_expected("expected closing bracket in list literal", "]"));
                }
                self.pos += 1; // consume ']'

                Ok(Box::new(Expr::ListLiteral(ListLiteral {
                    span: Span::new(start, self.tokens[self.pos - 1].pos + 1),
                    elements,
                })))
            }
            // Map literal: { "key": expr, "key": expr, ... }
            Type::LBrace => {
                let start = tok.pos;
                let mut entries: Vec<(String, Expr)> = Vec::new();

                // Check for empty map: {}
                if self.pos < self.tokens.len() && self.tokens[self.pos].typ == Type::RBrace {
                    self.pos += 1; // consume '}'
                    return Ok(Box::new(Expr::MapLiteral(MapLiteral {
                        span: Span::new(start, self.tokens[self.pos - 1].pos + 1),
                        entries,
                    })));
                }

                // Parse key-value pairs
                loop {
                    // Expect string key
                    if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::String {
                        return Err(self.error_expected("expected string key in map literal", "string"));
                    }
                    let key = self.tokens[self.pos].literal.clone();
                    self.pos += 1; // consume string

                    // Expect colon
                    if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::Colon {
                        return Err(self.error_expected("expected ':' in map literal", ":"));
                    }
                    self.pos += 1; // consume ':'

                    // Parse value expression
                    let value = self.parse_expr()?;
                    entries.push((key, *value));

                    // Check for comma or closing brace
                    if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::Comma {
                        break;
                    }
                    self.pos += 1; // consume ','
                }

                if self.pos >= self.tokens.len() || self.tokens[self.pos].typ != Type::RBrace {
                    return Err(self.error_expected("expected closing brace in map literal", "}"));
                }
                self.pos += 1; // consume '}'

                Ok(Box::new(Expr::MapLiteral(MapLiteral {
                    span: Span::new(start, self.tokens[self.pos - 1].pos + 1),
                    entries,
                })))
            }
            _ => Err(self.error(&format!("unexpected token: {:?} {}", tok.typ, tok.literal))),
        }
    }

    /// Parse anonymous function literal: $fn(x, y) -> Int { body }
    pub fn parse_fn_literal(&mut self) -> Result<Box<Expr>, Error> {
        crate::parser::literal::parse_fn_literal(self)
    }
}
