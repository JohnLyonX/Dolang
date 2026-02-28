// Expression parser - handles all expression parsing.
use crate::ast::{
    BinaryExpr, BoolLiteral, CharLiteral, Expr, FnCallExpr, NumberLiteral,
    StringLiteral, UnaryExpr, VarLookup,
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
            self.pos += 1;
            let right = self.parse_and_expr()?;
            left = Box::new(Expr::Binary(BinaryExpr {
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
            self.pos += 1;
            let right = self.parse_equality_expr()?;
            left = Box::new(Expr::Binary(BinaryExpr {
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
                self.pos += 1;
                let right = self.parse_comparison_expr()?;
                left = Box::new(Expr::Binary(BinaryExpr {
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
                self.pos += 1;
                let right = self.parse_additive_expr()?;
                left = Box::new(Expr::Binary(BinaryExpr {
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
                self.pos += 1;
                let right = self.parse_multiplicative_expr()?;
                left = Box::new(Expr::Binary(BinaryExpr {
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
                self.pos += 1;
                let right = self.parse_unary_expr()?;
                left = Box::new(Expr::Binary(BinaryExpr {
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
                self.pos += 1;
                let right = self.parse_unary_expr()?;
                return Ok(Box::new(Expr::Unary(UnaryExpr {
                    op: typ.clone(),
                    right,
                })));
            }
        }
        self.parse_primary()
    }

    pub fn parse_primary(&mut self) -> Result<Box<Expr>, Error> {
        if self.pos >= self.tokens.len() {
            return Err(self.error("unexpected end of input"));
        }
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;

        match tok.typ {
            Type::Number => Ok(Box::new(Expr::Number(NumberLiteral { value: tok.literal }))),
            Type::Char => Ok(Box::new(Expr::Char(CharLiteral { value: tok.literal }))),
            Type::Bool => {
                let value = tok.literal == "true";
                Ok(Box::new(Expr::Bool(BoolLiteral { value })))
            }
            Type::String => Ok(Box::new(Expr::StringLiteral(StringLiteral { value: tok.literal }))),
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
                        name: tok.literal,
                        args,
                    })))
                } else {
                    Ok(Box::new(Expr::VarLookup(VarLookup { name: tok.literal })))
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
            _ => Err(self.error(&format!("unexpected token: {:?} {}", tok.typ, tok.literal))),
        }
    }

    /// Parse anonymous function literal: $fn(x, y) -> Int { body }
    pub fn parse_fn_literal(&mut self) -> Result<Box<Expr>, Error> {
        crate::parser::literal::parse_fn_literal(self)
    }
}
