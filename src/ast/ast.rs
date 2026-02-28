// Abstract Syntax Tree node definitions for Dolang.
use crate::token::Type;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(NumberLiteral),
    Char(CharLiteral),
    Bool(BoolLiteral),
    StringLiteral(StringLiteral),
    VarLookup(VarLookup),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    FnCall(FnCallExpr),
    FnLiteral(FnLiteral),  // Anonymous function: $fn(x, y) -> Int { ... }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(PrintStmt),
    Assign(AssignStmt),
    VarDecl(VarDeclStmt),   // $ a <& 1;
    ConstDecl(ConstDeclStmt), // $@ a <& 1;
    If(IfStmt),             // $if condition { ... } $elif ... $else ...
    While(WhileStmt),       // $while condition { ... }
    Loop(LoopStmt),         // $loop { ... }
    For(ForStmt),           // $for init; condition; update { ... }
    Break(BreakStmt),       // $break;
    Continue(ContinueStmt), // $continue;
    Exit(ExitStmt),
    FnDecl(FnDeclStmt),     // $fn name(params) -> type { body }
    Return(ReturnStmt),     // $# expression;
    ExprStmt(Box<Expr>),    // expression statement (for function calls as statements)
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub op: Type,
}

#[derive(Debug, Clone)]
pub struct NumberLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct CharLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BoolLiteral {
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: Type,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarLookup {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PrintStmt {
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub name: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstDeclStmt {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub condition: Box<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub branches: Vec<IfBranch>,
    pub else_body: Vec<Stmt>,  // $else branch
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Box<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub body: Vec<Stmt>,
}

/// $for init; condition; update { body }
/// init and update are optional single statements (VarDecl or Assign).
#[derive(Debug, Clone)]
pub struct ForStmt {
    pub init: Option<Box<Stmt>>,
    pub condition: Option<Box<Expr>>,
    pub update: Option<Box<Stmt>>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {}

#[derive(Debug, Clone)]
pub struct ContinueStmt {}

#[derive(Debug, Clone)]
pub struct ExitStmt {}

#[derive(Debug, Clone)]
pub struct FnDeclStmt {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct FnCallExpr {
    pub name: String,
    pub args: Vec<Expr>,
}

/// Anonymous function literal: $fn(x, y) -> Int { ... }
#[derive(Debug, Clone)]
pub struct FnLiteral {
    pub params: Vec<String>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Box<Expr>>,
}
