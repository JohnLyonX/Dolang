// Parser module - converts tokens into AST statements.
pub mod expr;
pub mod literal;
pub mod stmt;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::ast::Stmt;
use self::stmt::StmtParser;

pub use expr::parse_expr_tokens;

/// Parse turns source into AST statements.
pub fn parse(src: &str) -> Result<Vec<Stmt>, Error> {
    let mut lexer = Lexer::new(src);
    let toks = lexer.lex_all().map_err(Error::Lexer)?;

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
