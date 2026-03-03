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

    /// Phase 2 过渡期：从旧版字符串存储格式解析为 DolangValue
    /// Phase 2 稳定后删除此方法
    pub fn parse_legacy(s: &str) -> Option<Self> {
        if s == "true" {
            return Some(DolangValue::Bool(true));
        }
        if s == "false" {
            return Some(DolangValue::Bool(false));
        }
        if s == "null" {
            return Some(DolangValue::Null);
        }
        if s.starts_with("__STR__:") {
            return Some(DolangValue::Str(s[8..].to_string()));
        }
        // 尝试解析为 i64
        if let Ok(n) = s.parse::<i64>() {
            return Some(DolangValue::Int(n));
        }
        // 尝试解析为 f64
        if let Ok(f) = s.parse::<f64>() {
            return Some(DolangValue::Float(f));
        }
        // List 解析
        if s.starts_with("__LST__:") {
            return parse_legacy_list(s);
        }
        // Map 解析
        if s.starts_with("__MAP__:") {
            return parse_legacy_map(s);
        }
        // 兜底：作为原始字符串
        Some(DolangValue::Str(s.to_string()))
    }

    /// Phase 2 过渡期：序列化回旧版字符串存储格式
    /// Phase 2 稳定后删除此方法
    pub fn to_legacy(&self) -> String {
        match self {
            DolangValue::Int(n) => n.to_string(),
            DolangValue::Float(n) => {
                if n.fract().abs() < 1e-10 {
                    format!("{:.1}", n)
                } else {
                    n.to_string()
                }
            }
            DolangValue::Str(s) => format!("__STR__:{}", s),
            DolangValue::Bool(b) => b.to_string(),
            DolangValue::Null => "null".to_string(),
            DolangValue::List(items) => {
                let items_str: Vec<String> = items.iter().map(|i| i.to_legacy()).collect();
                format!("__LST__:{}", items_str.join(":"))
            }
            DolangValue::Map(entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v.to_legacy()))
                    .collect();
                format!("__MAP__:{}", entries_str.join(":"))
            }
            DolangValue::Function { .. } => {
                // 函数转为字符串时返回函数名标识
                "<function>".to_string()
            }
        }
    }
}

/// 解析旧版列表字符串
/// 格式: __LST__:len:element:len:element:...
fn parse_legacy_list(s: &str) -> Option<DolangValue> {
    if !s.starts_with("__LST__:") {
        return None;
    }
    let content = &s[8..];
    if content.is_empty() {
        return Some(DolangValue::List(Vec::new()));
    }

    // 使用长度前缀格式解析
    let mut results = Vec::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        // Find the colon after the length
        if let Some(colon_pos) = remaining.find(':') {
            let len_str = &remaining[..colon_pos];
            let len = len_str.parse::<usize>().ok()?;
            let after_len = &remaining[colon_pos + 1..];

            // Check if we have enough content
            if after_len.len() < len {
                return None;
            }

            let element = &after_len[..len];
            // 递归解析元素
            if let Some(val) = DolangValue::parse_legacy(element) {
                results.push(val);
            }

            // Move past this element (length + colon + element)
            remaining = &after_len[len..];
            if !remaining.is_empty() && remaining.starts_with(':') {
                remaining = &remaining[1..];
            }
        } else {
            break;
        }
    }

    Some(DolangValue::List(results))
}

/// 解析旧版 Map 字符串
/// 格式: __MAP__:key_len:key:value_len:value:...
fn parse_legacy_map(s: &str) -> Option<DolangValue> {
    if !s.starts_with("__MAP__:") {
        return None;
    }
    let content = &s[8..];
    if content.is_empty() {
        return Some(DolangValue::Map(IndexMap::new()));
    }

    // 使用长度前缀格式解析
    let mut map = IndexMap::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        // Parse key
        if let Some(colon_pos) = remaining.find(':') {
            let key_len = remaining[..colon_pos].parse::<usize>().ok()?;
            let after_key_len = &remaining[colon_pos + 1..];

            if after_key_len.len() < key_len {
                return None;
            }
            let key = &after_key_len[..key_len];

            // Move to value
            let after_key = &after_key_len[key_len..];
            if after_key.is_empty() || !after_key.starts_with(':') {
                return None;
            }
            let val_part = &after_key[1..];

            // Parse value
            if let Some(val_colon_pos) = val_part.find(':') {
                let val_len = val_part[..val_colon_pos].parse::<usize>().ok()?;
                let after_val_len = &val_part[val_colon_pos + 1..];

                if after_val_len.len() < val_len {
                    return None;
                }
                let val_str = &after_val_len[..val_len];
                if let Some(val) = DolangValue::parse_legacy(val_str) {
                    map.insert(key.to_string(), val);
                }

                // Move past this entry
                remaining = &after_val_len[val_len..];
                if !remaining.is_empty() && remaining.starts_with(':') {
                    remaining = &remaining[1..];
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Some(DolangValue::Map(map))
}
