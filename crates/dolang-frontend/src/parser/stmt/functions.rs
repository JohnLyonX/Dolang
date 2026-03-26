use crate::ast::{FnDeclStmt, Span, Stmt};
use crate::error::Error;
use crate::token::Type;

use super::StmtParser;

impl<'a> StmtParser<'a> {
    pub fn parse_fn_decl(&mut self) -> Result<Stmt, Error> {
        let start = self.peek().pos;
        let is_public = self.peek().typ == Type::Fn;
        self.advance();

        if self.at_end() || self.peek().typ != Type::Ident {
            return Err(Error::InvalidStatement(None));
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

        Ok(Stmt::FnDecl(FnDeclStmt {
            span: Span::from_token(start),
            name,
            is_public,
            params,
            variadic_param,
            return_type,
            body,
        }))
    }
}
