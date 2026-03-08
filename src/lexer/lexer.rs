// Lexer for Dolang - converts source text into tokens.
use crate::token::{Token, Type};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn lex_all(&mut self) -> Result<Vec<Token>, String> {
        let mut toks = Vec::new();

        loop {
            let tok = self.next_token()?;
            toks.push(tok.clone());
            if tok.typ == Type::Eof {
                break;
            }
        }

        Ok(toks)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Ok(Token::eof(self.pos));
        }

        let ch = self.peek();
        let start = self.pos;

        match ch {
            ';' => {
                self.advance();
                Ok(Token::new(Type::Semicolon, ";", start))
            }
            '+' => {
                self.advance();
                // Check for += (compound assignment)
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::PlusAssign, "+=", start));
                }
                Ok(Token::new(Type::Plus, "+", start))
            }
            '-' => {
                self.advance();
                // Check for -= (compound assignment)
                if self.pos < self.input.len() && self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::MinusAssign, "-=", start));
                }
                // Check for -> (arrow)
                if self.pos < self.input.len() && self.peek() == '>' {
                    self.advance();
                    return Ok(Token::new(Type::Arrow, "->", start));
                }
                Ok(Token::new(Type::Minus, "-", start))
            }
            '*' => {
                self.advance();
                // Check for *= (compound assignment)
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::MulAssign, "*=", start));
                }
                Ok(Token::new(Type::Mul, "*", start))
            }
            '/' => {
                self.advance();
                // Check for single-line comment //
                if self.peek() == '/' {
                    self.advance();
                    self.skip_single_line_comment();
                    return self.next_token();
                }
                // Check for multi-line comment /*
                if self.peek() == '*' {
                    self.advance();
                    self.skip_multi_line_comment()?;
                    return self.next_token();
                }
                // Check for /= (compound assignment)
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::DivAssign, "/=", start));
                }
                Ok(Token::new(Type::Div, "/", start))
            }
            '%' => {
                self.advance();
                // Check for %= (compound assignment)
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::ModAssign, "%=", start));
                }
                Ok(Token::new(Type::Mod, "%", start))
            }
            '!' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::Ne, "!=", start));
                }
                Ok(Token::new(Type::Not, "!", start))
            }
            '<' => {
                // Check for <=
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::Lte, "<=", start));
                }
                Ok(Token::new(Type::Lt, "<", start))
            }
            '>' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::Gte, ">=", start));
                }
                Ok(Token::new(Type::Gt, ">", start))
            }
            '=' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    return Ok(Token::new(Type::Eq, "==", start));
                }
                // Single = is assignment
                Ok(Token::new(Type::Assign, "=", start))
            }
            '&' => {
                self.advance();
                if self.peek() == '&' {
                    self.advance();
                    return Ok(Token::new(Type::And, "&&", start));
                }
                Err("unexpected '&'".to_string())
            }
            '|' => {
                self.advance();
                if self.peek() == '|' {
                    self.advance();
                    return Ok(Token::new(Type::Or, "||", start));
                }
                Err("unexpected '|'".to_string())
            }
            '{' => {
                self.advance();
                Ok(Token::new(Type::LBrace, "{", start))
            }
            '}' => {
                self.advance();
                Ok(Token::new(Type::RBrace, "}", start))
            }
            '(' => {
                self.advance();
                Ok(Token::new(Type::LParen, "(", start))
            }
            ')' => {
                self.advance();
                Ok(Token::new(Type::RParen, ")", start))
            }
            '[' => {
                self.advance();
                Ok(Token::new(Type::LBracket, "[", start))
            }
            ']' => {
                self.advance();
                Ok(Token::new(Type::RBracket, "]", start))
            }
            ',' => {
                self.advance();
                Ok(Token::new(Type::Comma, ",", start))
            }
            ':' => {
                self.advance();
                Ok(Token::new(Type::Colon, ":", start))
            }
            '.' => {
                self.advance();
                // Check for ... (spread)
                if self.pos < self.input.len() && self.peek() == '.' {
                    self.advance();
                    if self.pos < self.input.len() && self.peek() == '.' {
                        self.advance();
                        return Ok(Token::new(Type::Spread, "...", start));
                    }
                }
                Ok(Token::new(Type::Dot, ".", start))
            }
            '$' => self.read_dollar(start),
            '"' => self.read_string(start),
            '\'' => self.read_char(start),
            _ => {
                if ch.is_ascii_digit() || ch == '-' {
                    return self.read_number(start);
                }
                // Identifier or keyword.
                let lit = self.read_until_delimiter();
                // Check for f-string: "f" followed by '"'
                if lit == "f" && self.pos < self.input.len() && self.input[self.pos] == '"' {
                    return self.read_fstring(start);
                }
                if lit == "true" {
                    return Ok(Token::new(Type::Bool, "true", start));
                }
                if lit == "false" {
                    return Ok(Token::new(Type::Bool, "false", start));
                }
                // Check for 'in' keyword (for-in loop)
                if lit == "in" {
                    return Ok(Token::new(Type::In, "in", start));
                }
                Ok(Token::new(Type::Ident, &lit, start))
            }
        }
    }

    fn read_dollar(&mut self, start: usize) -> Result<Token, String> {
        // Order matters: match longer sequences first to avoid prefix collisions

        if self.match_seq("$>>FILE") {
            self.advance_n(7);
            return Ok(Token::new(Type::Print, "$>>FILE", start));
        }

        if self.match_seq("$>>") {
            self.advance_n(3);
            return Ok(Token::new(Type::Print, "$>>", start));
        }

        if self.match_seq("$<<FILE") {
            self.advance_n(7);
            return Ok(Token::new(Type::Read, "$<<FILE", start));
        }

        if self.match_seq("$<<CONFIG") {
            self.advance_n(9);
            return Ok(Token::new(Type::ConfigRead, "$<<CONFIG", start));
        }

        if self.match_seq("$<<") {
            self.advance_n(3);
            return Ok(Token::new(Type::Read, "$<<", start));
        }

        if self.match_seq("$continue") {
            self.advance_n(9);
            return Ok(Token::new(Type::Continue, "$continue", start));
        }

        if self.match_seq("$fn") {
            self.advance_n(3);
            return Ok(Token::new(Type::Fn, "$fn", start));
        }

        if self.match_seq("$while") {
            self.advance_n(6);
            return Ok(Token::new(Type::While, "$while", start));
        }

        if self.match_seq("$break") {
            self.advance_n(6);
            return Ok(Token::new(Type::Break, "$break", start));
        }

        if self.match_seq("$elif") {
            self.advance_n(5);
            return Ok(Token::new(Type::Elif, "$elif", start));
        }

        if self.match_seq("$else") {
            self.advance_n(5);
            return Ok(Token::new(Type::Else, "$else", start));
        }

        if self.match_seq("$loop") {
            self.advance_n(5);
            return Ok(Token::new(Type::Loop, "$loop", start));
        }

        if self.match_seq("$for") {
            self.advance_n(4);
            return Ok(Token::new(Type::For, "$for", start));
        }

        if self.match_seq("$if") {
            self.advance_n(3);
            return Ok(Token::new(Type::If, "$if", start));
        }

        if self.match_seq("$@") {
            self.advance_n(2);
            return Ok(Token::new(Type::ConstDecl, "$@", start));
        }

        if self.match_seq("$#") {
            self.advance_n(2);
            return Ok(Token::new(Type::Return, "$#", start));
        }

        if self.match_seq("$mod") {
            self.advance_n(4);
            return Ok(Token::new(Type::ModDecl, "$mod", start));
        }

        if self.match_seq("$main") {
            self.advance_n(5);
            return Ok(Token::new(Type::MainDecl, "$main", start));
        }

        // Single $ is variable declaration
        self.advance();
        Ok(Token::new(Type::VarDecl, "$", start))
    }

    fn read_string(&mut self, start: usize) -> Result<Token, String> {
        self.advance(); // consume opening "
        let mut result = String::new();

        while self.pos < self.input.len() {
            let ch = self.peek();
            if ch == '"' {
                self.advance(); // consume closing "
                return Ok(Token::new(Type::String, &result, start));
            } else if ch == '\\' {
                // Escape sequence
                self.advance(); // consume '\'
                if self.pos >= self.input.len() {
                    return Err("unclosed string".to_string());
                }
                let next = self.peek();
                let escaped = match next {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '0' => '\0',
                    '\\' => '\\',
                    '"' => '"',
                    _ => return Err(format!("invalid escape sequence \"\\{}\"", next)),
                };
                result.push(escaped);
                self.advance(); // consume escaped char
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err("unclosed string".to_string())
    }

    /// Read an f-string: f"..."
    /// Format: f"Hello {name}, you have {count} items"
    /// - Text outside {} is literal
    /// - Inside {} can be: variable, expression, method call
    fn read_fstring(&mut self, start: usize) -> Result<Token, String> {
        // Check if we need to consume 'f' or if we're already at '"'
        if self.pos < self.input.len() && self.input[self.pos] == 'f' {
            // Consume the 'f' character
            self.advance();
        }
        // Now consume the opening "
        self.advance();

        let mut literal = String::new();
        let mut brace_depth = 0;

        while self.pos < self.input.len() {
            let ch = self.peek();
            match ch {
                '{' => {
                    brace_depth += 1;
                    literal.push('{');
                    self.advance();
                }
                '}' => {
                    if brace_depth == 0 {
                        return Err("f-string syntax error: unexpected \"}\"".to_string());
                    }
                    brace_depth -= 1;
                    literal.push('}');
                    self.advance();
                }
                '"' => {
                    if brace_depth > 0 {
                        return Err("f-string syntax error: unclosed \"{\"".to_string());
                    }
                    self.advance(); // consume closing "
                    return Ok(Token::new(Type::FString, &literal, start));
                }
                '\\' => {
                    // Escape sequence
                    self.advance(); // consume '\'
                    if self.pos >= self.input.len() {
                        return Err("unclosed f-string".to_string());
                    }
                    let next = self.peek();
                    let escaped = match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '0' => '\0',
                        '\\' => '\\',
                        '"' => '"',
                        '{' => '{',
                        '}' => '}',
                        _ => return Err(format!("invalid escape sequence \"\\{}\" in f-string", next)),
                    };
                    literal.push(escaped);
                    self.advance(); // consume escaped char
                }
                _ => {
                    literal.push(ch);
                    self.advance();
                }
            }
        }

        Err("unclosed f-string".to_string())
    }

    fn read_char(&mut self, start: usize) -> Result<Token, String> {
        self.advance(); // consume opening '
        if self.pos >= self.input.len() {
            return Err("unclosed char".to_string());
        }
        let ch = self.peek();

        // Check for escape sequences - not supported in char
        if ch == '\\' {
            return Err("Char type does not support escape sequences".to_string());
        }

        self.advance(); // consume the character
        if self.pos >= self.input.len() || self.peek() != '\'' {
            return Err("unclosed char".to_string());
        }
        self.advance(); // consume closing '
        Ok(Token::new(Type::Char, &ch.to_string(), start))
    }

    fn read_number(&mut self, start: usize) -> Result<Token, String> {
        // Check for negative number
        let mut has_sign = false;
        if self.peek() == '-' {
            self.advance();
            has_sign = true;
        }

        // Read digits before decimal point
        let digits_start = self.pos;
        while self.pos < self.input.len() && self.peek().is_ascii_digit() {
            self.advance();
        }
        let _int_part_len = self.pos - digits_start;

        // Check for decimal point - only treat as float if digits follow
        let mut is_float = false;
        let float_end_pos;
        if self.pos < self.input.len() && self.peek() == '.' {
            let after_dot = self.pos + 1;
            // Check if there are digits after the decimal point
            if after_dot < self.input.len() && self.input[after_dot].is_ascii_digit() {
                is_float = true;
                self.advance();
                // Read digits after decimal point
                while self.pos < self.input.len() && self.peek().is_ascii_digit() {
                    self.advance();
                }
                float_end_pos = self.pos;
            } else {
                // No digits after decimal point - don't treat as float
                // Return integer and let the dot be a separate token
                float_end_pos = self.pos;
            }
        } else {
            float_end_pos = self.pos;
        }

        let literal: String = self.input[start..float_end_pos].iter().collect();
        if literal.is_empty() || (has_sign && literal == "-") {
            return Err("invalid number literal".to_string());
        }

        // Validate the number
        if is_float {
            literal.parse::<f64>().map_err(|_| "invalid float literal".to_string())?;
        } else {
            literal.parse::<i64>().map_err(|_| "invalid integer literal".to_string())?;
        }

        Ok(Token::new(Type::Number, &literal, start))
    }

    fn read_until_delimiter(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() && !is_delimiter(self.peek()) {
            self.advance();
        }
        self.input[start..self.pos].iter().collect()
    }

    fn read_until_rune(&mut self, target: char) -> Vec<char> {
        let start = self.pos;
        while self.pos < self.input.len() {
            if self.peek() == target {
                return self.input[start..self.pos].to_vec();
            }
            self.advance();
        }
        Vec::new()
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.peek();
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '\\' {
                // Skip escape character (handles shell-escaped ! and other chars)
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip single-line comment starting with //
    fn skip_single_line_comment(&mut self) {
        while self.pos < self.input.len() && self.peek() != '\n' {
            self.advance();
        }
    }

    /// Skip multi-line comment starting with /* and ending with */
    fn skip_multi_line_comment(&mut self) -> Result<(), String> {
        while self.pos < self.input.len() {
            if self.peek() == '*' && self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '/' {
                self.advance_n(2); // Skip */
                return Ok(());
            }
            self.advance();
        }
        Err("unterminated multi-line comment".to_string())
    }

    fn peek(&self) -> char {
        self.input[self.pos]
    }


    fn advance(&mut self) {
        self.pos += 1;
    }

    fn advance_n(&mut self, n: usize) {
        self.pos += n;
    }

    fn match_seq(&self, seq: &str) -> bool {
        if self.pos + seq.len() > self.input.len() {
            return false;
        }
        for (i, c) in seq.chars().enumerate() {
            if self.input[self.pos + i] != c {
                return false;
            }
        }
        true
    }
}

fn is_delimiter(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | ';' | '+' | '-' | '*' | '/' | '%'
        | '<' | '>' | '=' | '!' | '&' | '|' | '$' | '"' | '\'' | '{' | '}'
        | '(' | ')' | '[' | ']' | ',' | '.' | ':')
}
