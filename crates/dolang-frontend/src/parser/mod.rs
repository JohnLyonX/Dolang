// Parser module - converts tokens into AST statements.
pub mod expr;
pub mod literal;
pub mod stmt;

use self::stmt::StmtParser;
use crate::ast::Stmt;
use crate::error::Error;
use crate::lexer::Lexer;

pub use expr::{parse_expr_tokens, parse_expr_tokens_with_src};

/// Parse turns source into AST statements.
pub fn parse(src: &str) -> Result<Vec<Stmt>, Error> {
    let mut lexer = Lexer::new(src);
    let toks = lexer.lex_all().map_err(Error::from)?;

    let mut parser = StmtParser::new(&toks, src);
    parser.parse_stmts()
}

/// Calculate line and column from byte position
pub fn calc_line_col(src: &str, pos: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, c) in src.char_indices() {
        if i >= pos {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::ast::Stmt;

    #[test]
    fn parses_basic_function_and_assignment() {
        let source = r#"
$fn add(a, b) -> Int {
    $# a + b;
}

$ value = add(1, 2);
"#;

        let statements = parse(source).expect("source should parse");
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn parses_private_function_and_wildcard_module_import() {
        let source = r#"
$mod math.*;

_$fn hidden() -> Int {
    $# 1;
}
"#;

        let statements = parse(source).expect("source should parse");
        match &statements[0] {
            Stmt::ModDecl(stmt) => {
                assert_eq!(stmt.path, "math");
                assert!(stmt.wildcard);
            }
            other => panic!("expected module declaration, got {other:?}"),
        }

        match &statements[1] {
            Stmt::FnDecl(stmt) => assert!(!stmt.is_public),
            other => panic!("expected function declaration, got {other:?}"),
        }
    }
}
