// Dolang 运行时值的类型安全表示
//
// 设计决策：
// - Char 内部归入 Str（与 Dolang 文档一致："字符类型在内部也以字符串形式存储"）
// - Null 表示无返回值函数的结果（$fn greet() { ... } 无 $# 语句时）
// - Function 表示匿名函数作为一等公民存储在变量中的情况

use crate::ast::Stmt;
use indexmap::IndexMap;
use std::fmt;

/// Dolang 运行时值的类型安全表示
#[derive(Debug, Clone)]
pub enum DolangValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Vec<DolangValue>),
    Map(IndexMap<String, DolangValue>),
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
        return_type: Option<String>,
    },
    Null,
}

impl PartialEq for DolangValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => {
                // 处理 NaN：Dolang 语义中 NaN == NaN → false
                a == b
            }
            (Self::Str(a), Self::Str(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Null, Self::Null) => true,
            // List / Map 递归比较
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            // Function 不可比较
            (Self::Function { .. }, _) => false,
            _ => false,
        }
    }
}

impl fmt::Display for DolangValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{}", n),
            Self::Float(n) => {
                // Format float but always keep decimal point
                if n.fract().abs() < 1e-10 {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Self::Str(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Null => write!(f, "null"),
            Self::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Self::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Self::Function { params, .. } => {
                write!(f, "<fn({})>", params.join(", "))
            }
        }
    }
}

impl DolangValue {
    /// 运行时类型名，用于 .type() 方法返回值
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::Str(_) => "String",
            Self::Bool(_) => "Bool",
            Self::List(_) => "List",
            Self::Map(_) => "Map",
            Self::Function { .. } => "Function",
            Self::Null => "Null",
        }
    }

    /// 判断值是否为真（用于布尔上下文）
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Int(n) => *n != 0,
            Self::Float(f) => *f != 0.0 && !f.is_nan(),
            Self::Str(s) => !s.is_empty(),
            Self::List(l) => !l.is_empty(),
            Self::Map(m) => !m.is_empty(),
            Self::Function { .. } => true,
            Self::Null => false,
        }
    }
}
