use std::borrow::Cow;

use crate::ast::{
    FStringLiteral, FileReadStmt, FileWriteStmt, PrintStmt, PrintTarget, ReadMode, ReadStmt, Span,
    Stmt, StringLiteral,
};
use crate::error::Error;
use crate::token::{Token, Type};

use super::StmtParser;

impl<'a> StmtParser<'a> {
    pub fn parse_print_stmt(&mut self) -> Result<Option<Stmt>, Error> {
        let start = self.peek().pos;
        let print_literal = self.peek().literal.clone();
        self.advance();

        if print_literal == "$>>FILE" {
            return self.parse_file_write(start);
        }

        let expr_toks = self.collect_until_semi();
        if expr_toks.is_empty() {
            return Err(Error::InvalidExpression(None));
        }

        if expr_toks.len() >= 2 && expr_toks[0].typ == Type::Ident && expr_toks[0].literal == "ERR"
        {
            let value = self.parse_stderr_target(&expr_toks)?;
            self.skip_semis();
            return Ok(Some(Stmt::Print(PrintStmt {
                span: Span::from_token(start),
                value,
                target: PrintTarget::Stderr,
            })));
        }

        let expr = self.parse_expr_tokens(&expr_toks)?;
        self.skip_semis();
        Ok(Some(Stmt::Print(PrintStmt {
            span: Span::from_token(start),
            value: expr,
            target: PrintTarget::Stdout,
        })))
    }

    pub fn parse_read_stmt(&mut self) -> Result<Option<Stmt>, Error> {
        let start = self.peek().pos;
        let read_literal = self.peek().literal.clone();
        self.advance();

        if read_literal == "$<<FILE" {
            return self.parse_file_read(start);
        }

        let expr_toks = self.collect_until_semi();
        if expr_toks.is_empty() {
            return Err(Error::Parse(crate::error::ParseError {
                message: "invalid read statement, use $<<ENV(\"KEY\") or $<<LINE(\"prompt\")"
                    .to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            }));
        }
        if expr_toks[0].typ != Type::Ident {
            return Err(Error::Parse(crate::error::ParseError {
                message: "expected ENV, LINE, or FILE after $<<".to_string(),
                line: 1,
                column: 1,
                found: Some(format!("{:?}", expr_toks[0].typ)),
                expected: Some("ENV, LINE, or FILE".to_string()),
            }));
        }

        let mode_name = expr_toks[0].literal.clone();
        if mode_name == "FILE" {
            return self.parse_file_read(start);
        }
        if mode_name != "ENV" && mode_name != "LINE" {
            return Err(Error::Parse(crate::error::ParseError {
                message: format!(
                    "unknown read mode '{}', expected ENV, LINE, or FILE",
                    mode_name
                ),
                line: 1,
                column: 1,
                found: Some(mode_name),
                expected: Some("ENV, LINE, or FILE".to_string()),
            }));
        }

        if expr_toks.len() < 2 || expr_toks[1].typ != Type::LParen {
            return Err(Error::Parse(crate::error::ParseError {
                message: format!(
                    "{} requires parentheses: $<<{}(\"KEY\")",
                    mode_name, mode_name
                ),
                line: 1,
                column: 1,
                found: None,
                expected: Some("(".to_string()),
            }));
        }

        let args = collect_top_level_string_args(&expr_toks, 2)?;

        let mode = if mode_name == "ENV" {
            if args.len() != 1 {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "ENV requires a key: $<<ENV(\"KEY\")".to_string(),
                    line: 1,
                    column: 1,
                    found: if args.is_empty() {
                        None
                    } else {
                        Some("multiple arguments".to_string())
                    },
                    expected: Some("one string argument".to_string()),
                }));
            }
            if args[0].typ != Type::String {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "ENV key must be a String".to_string(),
                    line: 1,
                    column: 1,
                    found: Some(format!("{:?}", args[0].typ)),
                    expected: Some("String".to_string()),
                }));
            }
            ReadMode::Env
        } else {
            if args.len() > 1 {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "LINE accepts at most one argument".to_string(),
                    line: 1,
                    column: 1,
                    found: Some(format!("{} arguments", args.len())),
                    expected: Some("0 or 1 argument".to_string()),
                }));
            }
            if args.len() == 1 && args[0].typ != Type::String {
                return Err(Error::Parse(crate::error::ParseError {
                    message: "LINE prompt must be a String".to_string(),
                    line: 1,
                    column: 1,
                    found: Some(format!("{:?}", args[0].typ)),
                    expected: Some("String".to_string()),
                }));
            }
            ReadMode::Line
        };

        let prompt = args.first().map(parse_string_like_expr).transpose()?;

        self.skip_semis();
        Ok(Some(Stmt::Read(ReadStmt {
            span: Span::from_token(start),
            mode,
            prompt,
        })))
    }

    pub fn parse_file_write(&mut self, start: usize) -> Result<Option<Stmt>, Error> {
        let expr_toks = self.collect_until_semi();
        if expr_toks.is_empty() {
            return Err(Error::Parse(crate::error::ParseError {
                message: "invalid file write statement, use $>>FILE(path, content) or $>>FILE(path, content, mode)".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            }));
        }
        if expr_toks[0].typ != Type::LParen {
            return Err(Error::Parse(crate::error::ParseError {
                message: "expected '(' after $>>FILE".to_string(),
                line: 1,
                column: 1,
                found: Some(format!("{:?}", expr_toks[0].typ)),
                expected: Some("(".to_string()),
            }));
        }

        let args = collect_top_level_file_args(&expr_toks);
        if args.len() < 2 {
            return Err(Error::Parse(crate::error::ParseError {
                message: "FILE write requires at least 2 arguments: path, content".to_string(),
                line: 1,
                column: 1,
                found: Some(format!("{} arguments", args.len())),
                expected: Some("path, content".to_string()),
            }));
        }

        let path_expr = self.parse_expr_tokens(&[args[0].clone()]).map_err(|_| {
            Error::Parse(crate::error::ParseError {
                message: "invalid path expression".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            })
        })?;
        let content_expr = self.parse_expr_tokens(&[args[1].clone()]).map_err(|_| {
            Error::Parse(crate::error::ParseError {
                message: "invalid content expression".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            })
        })?;
        let mode = if args.len() >= 3 {
            Some(self.parse_expr_tokens(&[args[2].clone()]).map_err(|_| {
                Error::Parse(crate::error::ParseError {
                    message: "invalid mode expression".to_string(),
                    line: 1,
                    column: 1,
                    found: None,
                    expected: None,
                })
            })?)
        } else {
            None
        };

        Ok(Some(Stmt::FileWrite(FileWriteStmt {
            span: Span::from_token(start),
            path: path_expr,
            content: Some(content_expr),
            mode,
        })))
    }

    pub fn parse_file_read(&mut self, start: usize) -> Result<Option<Stmt>, Error> {
        let expr_toks = self.collect_until_semi();
        if expr_toks.is_empty() {
            return Err(Error::Parse(crate::error::ParseError {
                message: "invalid file read statement, use $<<FILE(path)".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            }));
        }
        if expr_toks[0].typ != Type::LParen {
            return Err(Error::Parse(crate::error::ParseError {
                message: "expected '(' after $<<FILE".to_string(),
                line: 1,
                column: 1,
                found: Some(format!("{:?}", expr_toks[0].typ)),
                expected: Some("(".to_string()),
            }));
        }

        let args = collect_top_level_file_args(&expr_toks);
        if args.is_empty() {
            return Err(Error::Parse(crate::error::ParseError {
                message: "FILE read requires at least 1 argument: path".to_string(),
                line: 1,
                column: 1,
                found: Some("0 arguments".to_string()),
                expected: Some("path".to_string()),
            }));
        }

        let path_expr = self.parse_expr_tokens(&[args[0].clone()]).map_err(|_| {
            Error::Parse(crate::error::ParseError {
                message: "invalid path expression".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            })
        })?;
        let mode = if args.len() >= 2 {
            Some(self.parse_expr_tokens(&[args[1].clone()]).map_err(|_| {
                Error::Parse(crate::error::ParseError {
                    message: "invalid mode expression".to_string(),
                    line: 1,
                    column: 1,
                    found: None,
                    expected: None,
                })
            })?)
        } else {
            None
        };

        Ok(Some(Stmt::FileRead(FileReadStmt {
            span: Span::from_token(start),
            path: path_expr,
            mode,
        })))
    }

    fn parse_stderr_target(&self, expr_toks: &[Token]) -> Result<Box<crate::ast::Expr>, Error> {
        if expr_toks[1].typ != Type::LParen {
            return Err(Error::Parse(crate::error::ParseError {
                message: "ERR requires parentheses: $>>ERR(\"msg\")".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            }));
        }

        let mut found_arg: Option<(Token, bool)> = None;
        let mut paren_depth = 0;
        for tok in expr_toks.iter().skip(2) {
            match tok.typ {
                Type::LParen => paren_depth += 1,
                Type::RParen => {
                    if paren_depth == 0 {
                        break;
                    }
                    paren_depth -= 1;
                }
                Type::String if paren_depth == 0 && found_arg.is_none() => {
                    found_arg = Some((tok.clone(), false));
                }
                Type::FString if paren_depth == 0 && found_arg.is_none() => {
                    found_arg = Some((tok.clone(), true));
                }
                _ => {}
            }
        }

        let (arg_tok, is_fstring) = found_arg.ok_or_else(|| {
            Error::Parse(crate::error::ParseError {
                message: "ERR requires a string argument: $>>ERR(\"msg\")".to_string(),
                line: 1,
                column: 1,
                found: None,
                expected: None,
            })
        })?;

        parse_string_like_expr_with_flag(&arg_tok, is_fstring)
    }
}

fn collect_top_level_string_args(
    expr_toks: &[Token],
    start_index: usize,
) -> Result<Vec<Token>, Error> {
    let mut args = Vec::new();
    let mut paren_depth = 1;
    let mut i = start_index;
    while i < expr_toks.len() {
        let tok = &expr_toks[i];
        match tok.typ {
            Type::LParen => paren_depth += 1,
            Type::RParen => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        if paren_depth == 1 && (tok.typ == Type::String || tok.typ == Type::FString) {
            args.push(tok.clone());
        }
        i += 1;
    }

    if i >= expr_toks.len() || expr_toks[i].typ != Type::RParen {
        return Err(Error::Parse(crate::error::ParseError {
            message: "expected closing parenthesis".to_string(),
            line: 1,
            column: 1,
            found: None,
            expected: Some(")".to_string()),
        }));
    }

    Ok(args)
}

fn collect_top_level_file_args(expr_toks: &[Token]) -> Vec<Token> {
    let mut args: Vec<Token> = Vec::new();
    let mut paren_depth = 1;
    let mut i = 1;
    while i < expr_toks.len() {
        let tok = &expr_toks[i];
        match tok.typ {
            Type::LParen => paren_depth += 1,
            Type::RParen => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        if paren_depth == 1
            && (tok.typ == Type::String
                || tok.typ == Type::FString
                || tok.typ == Type::Ident
                || tok.typ == Type::Number)
        {
            args.push(tok.clone());
        }
        i += 1;
    }
    args
}

fn parse_string_like_expr(tok: &Token) -> Result<Box<crate::ast::Expr>, Error> {
    parse_string_like_expr_with_flag(tok, tok.typ == Type::FString)
}

fn parse_string_like_expr_with_flag(
    tok: &Token,
    is_fstring: bool,
) -> Result<Box<crate::ast::Expr>, Error> {
    if is_fstring {
        let segments =
            crate::parser::expr::parse_fstring_segments(&tok.literal, tok.pos).map_err(|e| {
                Error::Parse(crate::error::ParseError {
                    message: e.to_string(),
                    line: 1,
                    column: 1,
                    found: None,
                    expected: None,
                })
            })?;
        Ok(Box::new(crate::ast::Expr::FString(FStringLiteral {
            span: Span::from_token(tok.pos),
            segments,
        })))
    } else {
        Ok(Box::new(crate::ast::Expr::StringLiteral(StringLiteral {
            span: Span::from_token(tok.pos),
            value: Cow::Owned(tok.literal.clone()),
        })))
    }
}
