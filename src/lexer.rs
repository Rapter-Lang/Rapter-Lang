use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use crate::error::{CompilerError, ErrorKind, SourceLocation, Suggestion};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Const,
    Mut,
    If,
    Else,
    While,
    For,
    Return,
    Break,
    Continue,
    Match,
    Struct,
    Enum,
    Class,
    Public,
    Private,
    Protected,
    New,
    Delete,
    Import,
    As,
    Export,
    Extern,
    
    // Types
    Int,
    Float,
    Bool,
    Char,
    String,
    
    // Literals
    Integer(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BoolLiteral(bool),
    CharLiteral(char),
    
    // Identifiers
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    Ampersand,
    Pipe,
    
    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    ColonColon, // ::
    Comma,
    Dot,
    DotDot, // ..
    DotDotDot, // ...
    Arrow, // ->
    FatArrow, // =>
    Question, // ?
    
    // Comments
    Comment(String),
    
    // EOF
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Public => write!(f, "public"),
            TokenKind::Private => write!(f, "private"),
            TokenKind::Protected => write!(f, "protected"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Delete => write!(f, "delete"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Extern => write!(f, "extern"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Integer(_) => write!(f, "integer literal"),
            TokenKind::FloatLiteral(_) => write!(f, "float literal"),
            TokenKind::StringLiteral(_) => write!(f, "string literal"),
            TokenKind::BoolLiteral(_) => write!(f, "bool literal"),
            TokenKind::CharLiteral(_) => write!(f, "char literal"),
            TokenKind::Identifier(_) => write!(f, "identifier"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Equal => write!(f, "="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotDot => write!(f, "..."),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::Comment(_) => write!(f, "comment"),
            TokenKind::Eof => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub fn tokenize(source: &str, file_path: &PathBuf) -> Result<Vec<Token>, CompilerError> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut column = 1;
    
    let keywords = HashMap::from([
        ("fn", TokenKind::Fn),
        ("let", TokenKind::Let),
        ("const", TokenKind::Const),
        ("mut", TokenKind::Mut),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("while", TokenKind::While),
        ("for", TokenKind::For),
        ("return", TokenKind::Return),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
        ("match", TokenKind::Match),
        ("struct", TokenKind::Struct),
        ("enum", TokenKind::Enum),
        ("class", TokenKind::Class),
        ("public", TokenKind::Public),
        ("private", TokenKind::Private),
        ("protected", TokenKind::Protected),
        ("new", TokenKind::New),
        ("delete", TokenKind::Delete),
        ("import", TokenKind::Import),
        ("as", TokenKind::As),
        ("export", TokenKind::Export),
        ("extern", TokenKind::Extern),
        ("int", TokenKind::Int),
        ("float", TokenKind::Float),
        ("bool", TokenKind::Bool),
        ("char", TokenKind::Char),
        ("string", TokenKind::String),
        ("true", TokenKind::BoolLiteral(true)),
        ("false", TokenKind::BoolLiteral(false)),
    ]);
    
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\r' => {
                chars.next();
                column += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                column = 1;
            }
            '/' => {
                chars.next();
                column += 1;
                if let Some(&'/') = chars.peek() {
                    // Single line comment
                    chars.next();
                    column += 1;
                    let mut comment = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' {
                            break;
                        }
                        comment.push(ch);
                        chars.next();
                        column += 1;
                    }
                    tokens.push(Token { kind: TokenKind::Comment(comment), line, column });
                } else if let Some(&'*') = chars.peek() {
                    // Multi line comment
                    chars.next();
                    column += 1;
                    let mut comment = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '*' {
                            chars.next();
                            column += 1;
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                column += 1;
                                break;
                            } else {
                                comment.push('*');
                            }
                        } else {
                            comment.push(ch);
                            chars.next();
                            if ch == '\n' {
                                line += 1;
                                column = 1;
                            } else {
                                column += 1;
                            }
                        }
                    }
                    tokens.push(Token { kind: TokenKind::Comment(comment), line, column });
                } else {
                    tokens.push(Token { kind: TokenKind::Slash, line, column });
                }
            }
            '+' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::Plus, line, column });
                column += 1;
            }
            '-' => {
                chars.next();
                column += 1;
                if let Some(&'>') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::Arrow, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Minus, line, column: column - 1 });
                }
            }
            '*' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::Star, line, column });
                column += 1;
            }
            '%' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::Percent, line, column });
                column += 1;
            }
            '=' => {
                chars.next();
                column += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::EqualEqual, line, column: column - 1 });
                    column += 1;
                } else if let Some(&'>') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::FatArrow, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Equal, line, column: column - 1 });
                }
            }
            '!' => {
                chars.next();
                column += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::NotEqual, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Not, line, column: column - 1 });
                }
            }
            '<' => {
                chars.next();
                column += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::LessEqual, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Less, line, column: column - 1 });
                }
            }
            '>' => {
                chars.next();
                column += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::GreaterEqual, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Greater, line, column: column - 1 });
                }
            }
            '&' => {
                chars.next();
                column += 1;
                if let Some(&'&') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::And, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Ampersand, line, column: column - 1 });
                }
            }
            '|' => {
                chars.next();
                column += 1;
                if let Some(&'|') = chars.peek() {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::Or, line, column: column - 1 });
                    column += 1;
                } else {
                    tokens.push(Token { kind: TokenKind::Pipe, line, column: column - 1 });
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::LeftParen, line, column });
                column += 1;
            }
            ')' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::RightParen, line, column });
                column += 1;
            }
            '{' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::LeftBrace, line, column });
                column += 1;
            }
            '}' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::RightBrace, line, column });
                column += 1;
            }
            '[' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::LeftBracket, line, column });
                column += 1;
            }
            ']' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::RightBracket, line, column });
                column += 1;
            }
            ';' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::Semicolon, line, column });
                column += 1;
            }
            ':' => {
                chars.next();
                if chars.peek() == Some(&':') {
                    chars.next();
                    tokens.push(Token { kind: TokenKind::ColonColon, line, column });
                    column += 2;
                } else {
                    tokens.push(Token { kind: TokenKind::Colon, line, column });
                    column += 1;
                }
            }
            '?' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::Question, line, column });
                column += 1;
            }
            ',' => {
                chars.next();
                tokens.push(Token { kind: TokenKind::Comma, line, column });
                column += 1;
            }
            '.' => {
                chars.next();
                column += 1;
                if let Some(&'.') = chars.peek() {
                    chars.next();
                    column += 1;
                    if let Some(&'.') = chars.peek() {
                        // Three dots: ...
                        chars.next();
                        tokens.push(Token { kind: TokenKind::DotDotDot, line, column: column - 2 });
                        column += 1;
                    } else {
                        // Two dots: ..
                        tokens.push(Token { kind: TokenKind::DotDot, line, column: column - 1 });
                    }
                } else {
                    tokens.push(Token { kind: TokenKind::Dot, line, column: column - 1 });
                }
            }
            '"' => {
                chars.next();
                column += 1;
                let mut string = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        column += 1;
                        break;
                    }
                    if ch == '\\' {
                        // Escape sequence in string
                        chars.next();
                        column += 1;
                        if let Some(&esc) = chars.peek() {
                            chars.next();
                            column += 1;
                            match esc {
                                'n' => string.push('\n'),
                                't' => string.push('\t'),
                                'r' => string.push('\r'),
                                '0' => string.push('\0'),
                                '\\' => string.push('\\'),
                                '"' => string.push('"'),
                                other => string.push(other),
                            }
                        }
                    } else {
                        string.push(ch);
                        chars.next();
                        column += 1;
                    }
                }
                tokens.push(Token { kind: TokenKind::StringLiteral(string), line, column });
            }
            '\'' => {
                chars.next();
                column += 1;
                // Support simple escape sequences like '\\n', '\\t', '\\r', '\\\\', '\\''
                let ch_value: Option<char> = if let Some(ch) = chars.next() {
                    if ch == '\\' {
                        // Escape sequence
                        if let Some(esc) = chars.next() {
                            column += 2; // consumed two characters after opening quote
                            match esc {
                                'n' => Some('\n'),
                                't' => Some('\t'),
                                'r' => Some('\r'),
                                '0' => Some('\0'),
                                '\\' => Some('\\'),
                                '\'' => Some('\''),
                                // fall back to the escaped char itself
                                other => Some(other),
                            }
                        } else {
                            None
                        }
                    } else {
                        // Regular single character
                        column += 1;
                        Some(ch)
                    }
                } else { None };

                if let Some(&'\'') = chars.peek() {
                    chars.next();
                    // column already accounted above for inner chars; add one for closing quote
                    column += 1;
                    if let Some(val) = ch_value {
                        tokens.push(Token { kind: TokenKind::CharLiteral(val), line, column: column - 2 });
                    } else {
                        let location = SourceLocation::new(file_path.clone(), line, column);
                        return Err(CompilerError::new(
                            ErrorKind::UnterminatedString,
                            "unterminated char literal".to_string(),
                            location,
                        ).with_suggestion(Suggestion::simple(
                            "add a valid character and closing single quote (')"
                        )));
                    }
                } else {
                    let location = SourceLocation::new(file_path.clone(), line, column);
                    return Err(CompilerError::new(
                        ErrorKind::UnterminatedString,
                        "unterminated char literal".to_string(),
                        location,
                    ).with_suggestion(Suggestion::simple(
                        "add a closing single quote (') to complete the char literal"
                    )));
                }
            }
            '0'..='9' => {
                let mut number = String::new();
                let start_column = column;
                number.push(ch);
                chars.next();
                column += 1;
                let mut is_float = false;
                while let Some(&ch) = chars.peek() {
                    if ch.is_digit(10) {
                        number.push(ch);
                        chars.next();
                        column += 1;
                    } else if ch == '.' && !is_float {
                        // Check if this is followed by another dot (range operator)
                        let mut temp_chars = chars.clone();
                        temp_chars.next(); // consume the first dot
                        if let Some(&'.') = temp_chars.peek() {
                            // This is .. range operator, don't treat as float
                            break;
                        } else {
                            // This is a decimal point
                            is_float = true;
                            number.push(ch);
                            chars.next();
                            column += 1;
                        }
                    } else {
                        break;
                    }
                }
                if is_float {
                    match number.parse::<f64>() {
                        Ok(f) => tokens.push(Token { kind: TokenKind::FloatLiteral(f), line, column: start_column }),
                        Err(_) => {
                            let location = SourceLocation::new(file_path.clone(), line, start_column);
                            return Err(CompilerError::new(
                                ErrorKind::InvalidNumber,
                                format!("invalid float literal '{}'", number),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                "ensure the number is a valid floating-point literal (e.g., 3.14, 1.0, 0.5)"
                            )));
                        }
                    }
                } else {
                    match number.parse::<i64>() {
                        Ok(i) => tokens.push(Token { kind: TokenKind::Integer(i), line, column: start_column }),
                        Err(_) => {
                            let location = SourceLocation::new(file_path.clone(), line, start_column);
                            return Err(CompilerError::new(
                                ErrorKind::InvalidNumber,
                                format!("invalid integer literal '{}'", number),
                                location,
                            ).with_suggestion(Suggestion::simple(
                                "ensure the number is a valid integer (e.g., 42, 0, -123)"
                            )));
                        }
                    }
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                let start_column = column;
                ident.push(ch);
                chars.next();
                column += 1;
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                        column += 1;
                    } else {
                        break;
                    }
                }
                if let Some(kind) = keywords.get(&ident.as_str()) {
                    tokens.push(Token { kind: kind.clone(), line, column: start_column });
                } else {
                    tokens.push(Token { kind: TokenKind::Identifier(ident), line, column: start_column });
                }
            }
            _ => {
                let location = SourceLocation::new(file_path.clone(), line, column);
                return Err(CompilerError::new(
                    ErrorKind::UnexpectedCharacter,
                    format!("unexpected character '{}'", ch),
                    location,
                ).with_suggestion(Suggestion::simple(
                    "remove or replace the unexpected character with valid Rapter syntax"
                )));
            }
        }
    }
    
    tokens.push(Token { kind: TokenKind::Eof, line, column });
    Ok(tokens)
}