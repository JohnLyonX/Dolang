// Abstract Syntax Tree node definitions for Dolang.
use std::borrow::Cow;

use crate::token::Type;

/// Source code span - start and end positions (character indices)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn from_token(pos: usize) -> Self {
        Self { start: pos, end: pos }
    }
}

/// Helper trait to get span from AST nodes
pub trait Spanned {
    fn span(&self) -> Span;
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(NumberLiteral),
    Char(CharLiteral),
    Bool(BoolLiteral),
    StringLiteral(StringLiteral),
    ListLiteral(ListLiteral),
    MapLiteral(MapLiteral),
    VarLookup(VarLookup),
    IndexAccess(IndexAccess),
    MethodCall(MethodCall),
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
    ForIn(ForInStmt),       // $for item in iterable { ... }
    Break(BreakStmt),       // $break;
    Continue(ContinueStmt), // $continue;
    Exit(ExitStmt),
    FnDecl(FnDeclStmt),     // $fn name(params) -> type { body }
    Return(ReturnStmt),     // $# expression;
    ExprStmt(Box<Expr>),    // expression statement (for function calls as statements)
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub span: Span,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub op: Type,
}

#[derive(Debug, Clone)]
pub struct NumberLiteral {
    pub span: Span,
    pub value: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct CharLiteral {
    pub span: Span,
    pub value: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct BoolLiteral {
    pub span: Span,
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub span: Span,
    pub value: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct ListLiteral {
    pub span: Span,
    pub elements: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct MapLiteral {
    pub span: Span,
    pub entries: Vec<(String, Expr)>,  // (key, value expression)
}

#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub span: Span,
    pub object: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub span: Span,
    pub object: Box<Expr>,
    pub method: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub span: Span,
    pub op: Type,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarLookup {
    pub span: Span,
    pub name: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct PrintStmt {
    pub span: Span,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub span: Span,
    pub name: Box<Expr>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub span: Span,
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct ConstDeclStmt {
    pub span: Span,
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfBranch {
    pub span: Span,
    pub condition: Box<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub span: Span,
    pub branches: Vec<IfBranch>,
    pub else_body: Vec<Stmt>,  // $else branch
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub span: Span,
    pub condition: Box<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub span: Span,
    pub body: Vec<Stmt>,
}

/// $for init; condition; update { body }
/// init and update are optional single statements (VarDecl or Assign).
#[derive(Debug, Clone)]
pub struct ForStmt {
    pub span: Span,
    pub init: Option<Box<Stmt>>,
    pub condition: Option<Box<Expr>>,
    pub update: Option<Box<Stmt>>,
    pub body: Vec<Stmt>,
}

/// $for item in iterable { body }
/// item is the loop variable, iterable is List, Map, or String
#[derive(Debug, Clone)]
pub struct ForInStmt {
    pub span: Span,
    pub var: String,           // loop variable name
    pub iterable: Box<Expr>,   // expression to iterate over
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExitStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnDeclStmt {
    pub span: Span,
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct FnCallExpr {
    pub span: Span,
    pub name: String,
    pub args: Vec<Expr>,
}

/// Anonymous function literal: $fn(x, y) -> Int { ... }
#[derive(Debug, Clone)]
pub struct FnLiteral {
    pub span: Span,
    pub params: Vec<String>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub span: Span,
    pub value: Option<Box<Expr>>,
}

// Implement Spanned trait for all AST nodes
impl Spanned for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl Spanned for NumberLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for CharLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for BoolLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for StringLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ListLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for MapLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for IndexAccess {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for MethodCall {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for VarLookup {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for BinaryExpr {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for UnaryExpr {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for FnCallExpr {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for FnLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for PrintStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for AssignStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for VarDeclStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ConstDeclStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for IfBranch {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for IfStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for WhileStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for LoopStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ForStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ForInStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for BreakStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ContinueStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ExitStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for FnDeclStmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ReturnStmt {
    fn span(&self) -> Span {
        self.span
    }
}

// Spanned for Box<Expr>
impl Spanned for Box<Expr> {
    fn span(&self) -> Span {
        (**self).span()
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(n) => n.span(),
            Expr::Char(c) => c.span(),
            Expr::Bool(b) => b.span(),
            Expr::StringLiteral(s) => s.span(),
            Expr::ListLiteral(l) => l.span(),
            Expr::MapLiteral(m) => m.span(),
            Expr::IndexAccess(i) => i.span(),
            Expr::MethodCall(m) => m.span(),
            Expr::VarLookup(v) => v.span(),
            Expr::Binary(b) => b.span(),
            Expr::Unary(u) => u.span(),
            Expr::FnCall(f) => f.span(),
            Expr::FnLiteral(f) => f.span(),
        }
    }
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Print(s) => s.span(),
            Stmt::Assign(s) => s.span(),
            Stmt::VarDecl(s) => s.span(),
            Stmt::ConstDecl(s) => s.span(),
            Stmt::If(s) => s.span(),
            Stmt::While(s) => s.span(),
            Stmt::Loop(s) => s.span(),
            Stmt::For(s) => s.span(),
            Stmt::ForIn(s) => s.span(),
            Stmt::Break(s) => s.span(),
            Stmt::Continue(s) => s.span(),
            Stmt::Exit(s) => s.span(),
            Stmt::FnDecl(s) => s.span(),
            Stmt::Return(s) => s.span(),
            Stmt::ExprStmt(e) => e.span(),
        }
    }
}
