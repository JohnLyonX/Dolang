//! Dolang error types

use std::fmt;

/// Runtime error types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RuntimeError {
    TypeMismatch {
        name: String,
        expected: String,
        actual: String,
    },
    UndefinedVariable(String),
    UndefinedFunction(String),
    ConstReassign(String),
    FunctionNoReturnType { name: String },
    DivisionByZero,
    ModuloByZero,
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeMismatch { name, expected, actual } => {
                write!(f, "type mismatch: cannot assign {} to variable '{}' of type {}", actual, name, expected)
            }
            RuntimeError::UndefinedVariable(name) => {
                write!(f, "variable '{}' not found", name)
            }
            RuntimeError::UndefinedFunction(name) => {
                write!(f, "function '{}' is not defined", name)
            }
            RuntimeError::ConstReassign(name) => {
                write!(f, "cannot reassign constant '{}'", name)
            }
            RuntimeError::FunctionNoReturnType { name } => {
                write!(f, "function '{}' has no return type declared but returns a value", name)
            }
            RuntimeError::DivisionByZero => {
                write!(f, "division by zero")
            }
            RuntimeError::ModuloByZero => {
                write!(f, "modulo by zero")
            }
            RuntimeError::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Unified error type for Dolang
#[derive(Debug)]
pub enum Error {
    /// Parse error
    Parse(ParseError),
    /// Runtime error
    Runtime(RuntimeError),
    /// Invalid expression
    InvalidExpression,
    /// Invalid assignment
    InvalidAssignment,
    /// Invalid statement
    InvalidStatement,
    /// Lexer error
    Lexer(String),
    /// Interpreter error
    Interpreter(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{}", e),
            Error::Runtime(e) => write!(f, "runtime error: {}", e),
            Error::InvalidExpression => write!(f, "invalid expression"),
            Error::InvalidAssignment => write!(f, "invalid assignment"),
            Error::InvalidStatement => write!(f, "invalid statement"),
            Error::Lexer(s) => write!(f, "lexer error: {}", s),
            Error::Interpreter(s) => write!(f, "runtime error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl From<RuntimeError> for Error {
    fn from(err: RuntimeError) -> Self {
        Error::Runtime(err)
    }
}

/// Parse error with context information
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub found: Option<String>,
    pub expected: Option<String>,
}

impl ParseError {
    #[allow(dead_code)]
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            line: 1,
            column: 1,
            found: None,
            expected: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    #[allow(dead_code)]
    pub fn with_found(mut self, found: &str) -> Self {
        self.found = Some(found.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_expected(mut self, expected: &str) -> Self {
        self.expected = Some(expected.to_string());
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at line {}:{}", self.line, self.column)?;
        if let Some(found) = &self.found {
            write!(f, "\n  found: {}", found)?;
        }
        if let Some(expected) = &self.expected {
            write!(f, "\n  expected: {}", expected)?;
        }
        write!(f, "\n  {}", self.message)
    }
}

impl std::error::Error for ParseError {}
