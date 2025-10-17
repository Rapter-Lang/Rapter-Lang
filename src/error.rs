use std::fmt;
use std::path::PathBuf;
use std::error::Error;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    // Lexical errors
    UnexpectedCharacter,
    UnterminatedString,
    InvalidNumber,
    InvalidEscapeSequence,

    // Parse errors
    UnexpectedToken,
    ExpectedToken,
    MissingSemicolon,
    UnclosedDelimiter,
    InvalidSyntax,

    // Semantic errors
    UndefinedVariable,
    UndefinedFunction,
    UndefinedType,
    UndefinedModule,
    DuplicateDefinition,
    TypeMismatch,
    InvalidOperation,
    WrongArgumentCount,
    ImmutableAssignment,
    MissingReturnType,

    // Module errors
    ModuleNotFound,
    ModuleLoadError,
    ModuleExportError,
    CircularImport,
    ExportNotFound,
    ImportConflict,

    // Code generation errors
    UnsupportedFeature,
    InternalError,
}

impl ErrorKind {
    pub fn code(&self) -> &'static str {
        match self {
            ErrorKind::UnexpectedCharacter => "E001",
            ErrorKind::UnterminatedString => "E002",
            ErrorKind::InvalidNumber => "E003",
            ErrorKind::InvalidEscapeSequence => "E004",
            ErrorKind::UnexpectedToken => "E101",
            ErrorKind::ExpectedToken => "E102",
            ErrorKind::MissingSemicolon => "E103",
            ErrorKind::UnclosedDelimiter => "E104",
            ErrorKind::InvalidSyntax => "E105",
            ErrorKind::UndefinedVariable => "E201",
            ErrorKind::UndefinedFunction => "E202",
            ErrorKind::UndefinedType => "E203",
            ErrorKind::UndefinedModule => "E204",
            ErrorKind::DuplicateDefinition => "E205",
            ErrorKind::TypeMismatch => "E206",
            ErrorKind::InvalidOperation => "E207",
            ErrorKind::WrongArgumentCount => "E208",
            ErrorKind::ImmutableAssignment => "E209",
            ErrorKind::MissingReturnType => "E210",
            ErrorKind::ModuleNotFound => "E301",
            ErrorKind::ModuleLoadError => "E302",
            ErrorKind::ModuleExportError => "E303",
            ErrorKind::CircularImport => "E304",
            ErrorKind::ExportNotFound => "E305",
            ErrorKind::ImportConflict => "E306",
            ErrorKind::UnsupportedFeature => "E401",
            ErrorKind::InternalError => "E500",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            ErrorKind::UnexpectedCharacter => "unexpected character",
            ErrorKind::UnterminatedString => "unterminated string literal",
            ErrorKind::InvalidNumber => "invalid number literal",
            ErrorKind::InvalidEscapeSequence => "invalid escape sequence",
            ErrorKind::UnexpectedToken => "unexpected token",
            ErrorKind::ExpectedToken => "expected token",
            ErrorKind::MissingSemicolon => "missing semicolon",
            ErrorKind::UnclosedDelimiter => "unclosed delimiter",
            ErrorKind::InvalidSyntax => "invalid syntax",
            ErrorKind::UndefinedVariable => "undefined variable",
            ErrorKind::UndefinedFunction => "undefined function",
            ErrorKind::UndefinedType => "undefined type",
            ErrorKind::UndefinedModule => "undefined module",
            ErrorKind::DuplicateDefinition => "duplicate definition",
            ErrorKind::TypeMismatch => "type mismatch",
            ErrorKind::InvalidOperation => "invalid operation",
            ErrorKind::WrongArgumentCount => "wrong number of arguments",
            ErrorKind::ImmutableAssignment => "cannot assign to immutable variable",
            ErrorKind::MissingReturnType => "missing return type",
            ErrorKind::ModuleNotFound => "module not found",
            ErrorKind::ModuleLoadError => "module load error",
            ErrorKind::ModuleExportError => "module export error",
            ErrorKind::CircularImport => "circular import detected",
            ErrorKind::ExportNotFound => "export not found",
            ErrorKind::ImportConflict => "import conflict",
            ErrorKind::UnsupportedFeature => "unsupported feature",
            ErrorKind::InternalError => "internal compiler error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: Option<usize>,
}

impl SourceLocation {
    pub fn new(file: PathBuf, line: usize, column: usize) -> Self {
        SourceLocation {
            file,
            line,
            column,
            length: None,
        }
    }

    pub fn with_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
    }

    pub fn span(&self) -> String {
        format!("{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub code_example: Option<String>,
    pub help_link: Option<String>,
}

impl Suggestion {
    pub fn simple(message: impl Into<String>) -> Self {
        Suggestion {
            message: message.into(),
            code_example: None,
            help_link: None,
        }
    }

    pub fn with_example(message: impl Into<String>, example: impl Into<String>) -> Self {
        Suggestion {
            message: message.into(),
            code_example: Some(example.into()),
            help_link: None,
        }
    }

    pub fn with_help(message: impl Into<String>, help_link: impl Into<String>) -> Self {
        Suggestion {
            message: message.into(),
            code_example: None,
            help_link: Some(help_link.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: SourceLocation,
    pub context: Option<String>, // Additional context about the error
    pub suggestions: Vec<Suggestion>,
    pub related_errors: Vec<CompilerError>,
}

impl CompilerError {
    pub fn new(kind: ErrorKind, message: String, location: SourceLocation) -> Self {
        CompilerError {
            kind,
            message,
            location,
            context: None,
            suggestions: Vec::new(),
            related_errors: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_related_error(mut self, error: CompilerError) -> Self {
        self.related_errors.push(error);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }
}

impl Error for CompilerError {}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Error header with code and title
        writeln!(f, "\x1b[1;31merror[{}]\x1b[0m: {}", self.kind.code(), self.kind.title())?;
        writeln!(f, "  \x1b[1m{}\x1b[0m", self.message)?;
        writeln!(f, "  \x1b[36m-->\x1b[0m {}", self.location.span())?;

        // Try to read and display the source line
        if let Ok(source_content) = fs::read_to_string(&self.location.file) {
            let lines: Vec<&str> = source_content.lines().collect();
            if self.location.line > 0 && self.location.line <= lines.len() {
                let line_content = lines[self.location.line - 1];
                let line_number = self.location.line;

                // Display the source line
                writeln!(f, "  \x1b[36m|\x1b[0m")?;
                writeln!(f, "  \x1b[36m{} |\x1b[0m {}", line_number, line_content)?;

                // Display the caret pointing to the error
                let caret_position = self.location.column.saturating_sub(1);
                let caret_len = self.location.length.unwrap_or(1);
                let spaces = " ".repeat(caret_position);
                let carets = "^".repeat(caret_len);
                writeln!(f, "  \x1b[36m{} |\x1b[0m     \x1b[1;31m{}{}\x1b[0m", " ".repeat(line_number.to_string().len()), spaces, carets)?;
            }
        } else {
            // Fallback if we can't read the file
            writeln!(f, "  \x1b[36m|\x1b[0m")?;
            let caret_len = self.location.length.unwrap_or(1);
            writeln!(f, "  \x1b[36m|\x1b[0m     \x1b[1;31m{}\x1b[0m", "^".repeat(caret_len))?;
        }

        // Context if available
        if let Some(context) = &self.context {
            writeln!(f, "  \x1b[36m|\x1b[0m")?;
            writeln!(f, "  \x1b[36m|\x1b[0m {}", context)?;
        }

        // Suggestions
        for suggestion in &self.suggestions {
            writeln!(f, "  \x1b[36m|\x1b[0m")?;
            writeln!(f, "  \x1b[32mhelp\x1b[0m: {}", suggestion.message)?;

            if let Some(example) = &suggestion.code_example {
                writeln!(f, "  \x1b[36m|\x1b[0m")?;
                for line in example.lines() {
                    writeln!(f, "  \x1b[36m|\x1b[0m     {}", line)?;
                }
            }

            if let Some(link) = &suggestion.help_link {
                writeln!(f, "  \x1b[36m|\x1b[0m     \x1b[2mSee: {}\x1b[0m", link)?;
            }
        }

        // Related errors
        for related in &self.related_errors {
            writeln!(f)?;
            write!(f, "{}", related)?;
        }

        Ok(())
    }
}

pub fn report_error(error: &CompilerError) {
    eprintln!("{}", error);
}

pub fn report_errors(errors: &[CompilerError]) {
    for (i, error) in errors.iter().enumerate() {
        if i > 0 {
            eprintln!();
        }
        report_error(error);
    }
}

// Convenience functions for creating common errors
pub fn undefined_variable(name: &str, location: SourceLocation) -> CompilerError {
    let mut error = CompilerError::new(
        ErrorKind::UndefinedVariable,
        format!("cannot find variable `{}` in this scope", name),
        location,
    );

    error.suggestions.push(Suggestion::simple(
        "consider declaring the variable with `let` or check if it's spelled correctly"
    ));

    // Could add more suggestions based on similar variable names, etc.
    error
}

pub fn type_mismatch(expected: &str, found: &str, location: SourceLocation) -> CompilerError {
    let mut error = CompilerError::new(
        ErrorKind::TypeMismatch,
        format!("expected `{}`, found `{}`", expected, found),
        location,
    );

    error.suggestions.push(Suggestion::simple(
        "consider converting the value to the expected type or changing the expected type"
    ));

    error
}

pub fn unexpected_token(expected: &str, found: &str, location: SourceLocation) -> CompilerError {
    let mut error = CompilerError::new(
        ErrorKind::UnexpectedToken,
        format!("expected `{}`, found `{}`", expected, found),
        location,
    );

    // Could add specific suggestions based on the tokens
    error.suggestions.push(Suggestion::simple(
        "check for missing or extra punctuation"
    ));

    error
}

pub fn duplicate_definition(name: &str, location: SourceLocation, previous_location: SourceLocation) -> CompilerError {
    let mut error = CompilerError::new(
        ErrorKind::DuplicateDefinition,
        format!("the name `{}` is already defined", name),
        location,
    );

    error.related_errors.push(CompilerError::new(
        ErrorKind::DuplicateDefinition,
        format!("previous definition of `{}`", name),
        previous_location,
    ));

    error.suggestions.push(Suggestion::simple(
        "consider renaming one of the definitions or using different scopes"
    ));

    error
}