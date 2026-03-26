use crate::ast::{HttpBlockStmt, HttpFnStmt, Span, Stmt};
use crate::error::Error;
use crate::token::Type;

use super::StmtParser;

impl<'a> StmtParser<'a> {
    pub fn parse_http_fn(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;

        let method = match self.peek().typ {
            Type::HttpGet => "GET",
            Type::HttpPost => "POST",
            Type::HttpPut => "PUT",
            Type::HttpDel => "DELETE",
            Type::HttpPatch => "PATCH",
            _ => return Err(Error::InvalidStatement(None)),
        };
        self.advance();

        self.expect(Type::LParen)?;
        if self.at_end() || self.peek().typ != Type::String {
            return Err(Error::Parse(crate::error::ParseError {
                message: format!("expected path string after ${}", method.to_lowercase()),
                line: 1,
                column: 1,
                found: None,
                expected: Some("path string".to_string()),
            }));
        }
        let path = self.advance().literal.clone();
        self.expect(Type::RParen)?;

        if self.at_end() || self.peek().typ != Type::Ident {
            return Err(Error::Parse(crate::error::ParseError {
                message: "expected function name after path".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: Some("function name".to_string()),
            }));
        }
        let name = self.advance().literal.clone();

        self.expect(Type::LParen)?;

        let mut params: Vec<String> = Vec::new();
        let mut variadic_param: Option<String> = None;
        if !self.at_end() && self.peek().typ != Type::RParen {
            loop {
                if self.peek().typ == Type::Spread {
                    self.advance();
                    if self.at_end() || self.peek().typ != Type::Ident {
                        return Err(Error::InvalidStatement(None));
                    }
                    variadic_param = Some(self.advance().literal.clone());
                    break;
                }
                if self.at_end() || self.peek().typ != Type::Ident {
                    return Err(Error::InvalidStatement(None));
                }
                params.push(self.advance().literal.clone());
                if self.at_end() || self.peek().typ != Type::Comma {
                    break;
                }
                self.advance();
            }
        }

        self.expect(Type::RParen)?;

        let return_type = if !self.at_end() && self.peek().typ == Type::Arrow {
            self.advance();
            if self.at_end() || self.peek().typ != Type::Ident {
                return Err(Error::InvalidStatement(None));
            }
            let base = self.advance().literal.clone();
            // Support JSON<User> compound return type annotation
            if !self.at_end() && self.peek().typ == Type::Lt {
                self.advance(); // consume <
                if self.at_end() || self.peek().typ != Type::Ident {
                    return Err(Error::InvalidStatement(None));
                }
                let param = self.advance().literal.clone();
                self.expect(Type::Gt)?; // consume >
                Some(format!("{}<{}>", base, param))
            } else {
                Some(base)
            }
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Stmt::HttpFn(HttpFnStmt {
            span: Span::from_token(start),
            method: method.to_string(),
            path,
            name,
            params,
            variadic_param,
            return_type,
            body,
        }))
    }

    pub fn parse_http_block(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        self.advance();

        let mut prefix: Option<String> = None;
        if !self.at_end() && self.peek().typ == Type::LParen {
            self.advance();
            let mut path = String::new();
            while !self.at_end() && self.peek().typ != Type::RParen {
                path.push_str(&self.peek().literal);
                self.advance();
            }
            if self.at_end() || self.peek().typ != Type::RParen {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "expected ')' after $HTTP(path".to_string(),
                    line: 1,
                    column: 1,
                    expected: Some("')'".to_string()),
                    found: None,
                }));
            }
            self.advance();
            prefix = Some(path);
        }

        if !self.at_end() && self.peek().typ == Type::Dot {
            let mut links: Vec<String> = Vec::new();
            while !self.at_end() && self.peek().typ == Type::Dot {
                self.advance();
                if self.at_end() || self.peek().literal != "link" {
                    return Err(Error::Parse(crate::error::ParseError {
                        message: "expected 'link' after '.'".to_string(),
                        line: 1,
                        column: 1,
                        expected: Some("'link'".to_string()),
                        found: None,
                    }));
                }
                self.advance();
                if self.at_end() || self.peek().typ != Type::LParen {
                    return Err(Error::Parse(crate::error::ParseError {
                        message: "expected '(' after .link".to_string(),
                        line: 1,
                        column: 1,
                        expected: Some("'('".to_string()),
                        found: None,
                    }));
                }
                self.advance();

                let mut module_path = String::new();
                while !self.at_end() && self.peek().typ != Type::RParen {
                    module_path.push_str(&self.peek().literal);
                    self.advance();
                }
                if self.at_end() || self.peek().typ != Type::RParen {
                    return Err(Error::Parse(crate::error::ParseError {
                        message: "expected ')' after .link(module".to_string(),
                        line: 1,
                        column: 1,
                        expected: Some("')'".to_string()),
                        found: None,
                    }));
                }
                self.advance();
                links.push(module_path);
            }

            if !links.is_empty() {
                if !self.at_end() && self.peek().typ == Type::Semicolon {
                    self.advance();
                }
                return Ok(Stmt::HttpBlock(HttpBlockStmt {
                    span: Span::new(start, start + 1),
                    prefix,
                    link: links.into_iter().next(),
                    routes: Vec::new(),
                }));
            }
        }

        if prefix.is_some() && (self.at_end() || self.peek().typ != Type::LBrace) {
            return Err(Error::Parse(crate::error::ParseError {
                message: "expected '{' or '.link' after $HTTP(path)".to_string(),
                line: 1,
                column: 1,
                expected: Some("'{' or '.link'".to_string()),
                found: None,
            }));
        }

        if self.at_end() || self.peek().typ != Type::LBrace {
            return Err(Error::Parse(crate::error::ParseError {
                message: "expected '{' after $HTTP".to_string(),
                line: 1,
                column: 1,
                expected: Some("'{'".to_string()),
                found: None,
            }));
        }
        self.advance();

        let mut tokens = Vec::new();
        let mut brace_depth = 1;
        while !self.at_end() {
            let tok = self.peek().clone();
            if tok.typ == Type::LBrace {
                brace_depth += 1;
            } else if tok.typ == Type::RBrace {
                brace_depth -= 1;
                if brace_depth == 0 {
                    self.advance();
                    break;
                }
            }
            tokens.push(tok);
            self.advance();
        }

        let mut routes = Vec::new();
        let mut pos = 0;
        while pos < tokens.len() {
            let tok = &tokens[pos];
            if matches!(
                tok.typ,
                Type::HttpGet | Type::HttpPost | Type::HttpPut | Type::HttpDel | Type::HttpPatch
            ) {
                let mut sub_parser = StmtParser::new(&tokens[pos..], "");
                match sub_parser.parse_http_fn() {
                    Ok(Stmt::HttpFn(route)) => routes.push(route),
                    Ok(_) => {
                        pos += 1;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
                pos += sub_parser.pos;
            } else {
                pos += 1;
            }
        }

        Ok(Stmt::HttpBlock(HttpBlockStmt {
            span: Span::new(start, start + 1),
            prefix,
            link: None,
            routes,
        }))
    }
}
