use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use crate::error::{CompilerError, ErrorKind, SourceLocation};
use std::path::PathBuf;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    file_path: PathBuf,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file_path: PathBuf) -> Self {
        Parser {
            tokens,
            current: 0,
            file_path,
        }
    }

    // Helper methods for creating errors with source locations
    fn peek_location(&self) -> SourceLocation {
        let token = self.peek();
        SourceLocation::new(
            self.file_path.clone(),
            token.line,
            token.column,
        )
    }

    fn _previous_location(&self) -> SourceLocation {
        let token = self.previous();
        SourceLocation::new(
            self.file_path.clone(),
            token.line,
            token.column,
        )
    }

    fn error(&self, kind: ErrorKind, message: String) -> CompilerError {
        CompilerError::new(kind, message, self.peek_location())
    }

    fn unexpected_token_error(&self, expected: &str) -> CompilerError {
        let found = format!("{}", self.peek().kind);
        crate::error::unexpected_token(expected, &found, self.peek_location())
    }
    
    pub fn parse(&mut self) -> Result<Program, CompilerError> {
        let mut functions = Vec::new();
        let mut extern_functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut global_variables = Vec::new();
        
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::Comment(_) => {
                    self.advance(); // Skip comments
                    continue;
                }
                TokenKind::Let => {
                    global_variables.push(self.global_variable()?);
                }
                TokenKind::Fn => {
                    functions.push(self.function()?);
                }
                TokenKind::Extern => {
                    extern_functions.push(self.extern_function()?);
                }
                TokenKind::Struct => {
                    structs.push(self.struct_def()?);
                }
                TokenKind::Enum => {
                    enums.push(self.enum_def()?);
                }
                TokenKind::Import => {
                    imports.push(self.import()?);
                }
                TokenKind::Export => {
                    self.consume(TokenKind::Export)?;
                    match self.peek().kind {
                        TokenKind::Fn => {
                            let func = self.function()?;
                            functions.push(func.clone());
                            exports.push(Export {
                                item: ExportItem::Function(func.name),
                            });
                        }
                        TokenKind::Struct => {
                            let strct = self.struct_def()?;
                            structs.push(strct.clone());
                            exports.push(Export {
                                item: ExportItem::Struct(strct.name),
                            });
                        }
                        TokenKind::Enum => {
                            let enm = self.enum_def()?;
                            enums.push(enm.clone());
                            exports.push(Export {
                                item: ExportItem::Enum(enm.name),
                            });
                        }
                        _ => {
                            return Err(self.error(
                                ErrorKind::ExpectedToken,
                                format!("expected `fn`, `struct`, or `enum` after `export`, found `{}`", self.peek().kind),
                            ).with_suggestion(crate::error::Suggestion::with_example(
                                "try exporting a function, struct, or enum",
                                "export fn my_function() {\n    // ...\n}\n\nexport struct MyStruct {\n    // ...\n}\n\nexport enum MyEnum {\n    // ...\n}"
                            )));
                        }
                    }
                }
                _ => {
                    return Err(self.error(
                        ErrorKind::UnexpectedToken,
                        format!("unexpected token `{}`", self.peek().kind),
                    ).with_suggestion(crate::error::Suggestion::simple(
                        "expected a top-level declaration like `fn`, `struct`, `import`, or `export`"
                    )));
                }
            }
        }
        
        Ok(Program {
            imports,
            exports,
            extern_functions,
            functions,
            structs,
            enums,
            global_variables,
        })
    }
    
    fn global_variable(&mut self) -> Result<GlobalVariable, CompilerError> {
        self.consume(TokenKind::Let)?;
        let mutable = self.match_token(TokenKind::Mut);
        let name = self.identifier()?;
        let var_type = if self.match_token(TokenKind::Colon) {
            Some(self.type_annotation()?)
        } else {
            None
        };
        let initializer = if self.match_token(TokenKind::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon)?;
        Ok(GlobalVariable {
            name,
            var_type,
            mutable,
            initializer,
        })
    }
    
    fn function(&mut self) -> Result<Function, CompilerError> {
        self.consume(TokenKind::Fn)?;
        let name = self.identifier()?;
        self.consume(TokenKind::LeftParen)?;
        let parameters = self.parameters()?;
        self.consume(TokenKind::RightParen)?;
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.type_annotation()?)
        } else {
            None
        };
        self.consume(TokenKind::LeftBrace)?;
        let body = self.block()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }
    
    fn extern_parameters(&mut self) -> Result<(Vec<Parameter>, bool), CompilerError> {
        let mut params = Vec::new();
        let mut variadic = false;
        
        if !self.check(TokenKind::RightParen) {
            loop {
                if self.match_token(TokenKind::DotDotDot) {
                    variadic = true;
                    break;
                }
                
                let name = self.identifier()?;
                self.consume(TokenKind::Colon)?;
                let param_type = self.type_annotation()?;
                params.push(Parameter { name, param_type });
                
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        Ok((params, variadic))
    }
    
    fn parameters(&mut self) -> Result<Vec<Parameter>, CompilerError> {
        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                let name = self.identifier()?;
                self.consume(TokenKind::Colon)?;
                let param_type = self.type_annotation()?;
                params.push(Parameter { name, param_type });
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        Ok(params)
    }
    
    fn type_annotation(&mut self) -> Result<Type, CompilerError> {
        let ty: Result<Type, CompilerError> = match &self.peek().kind {
            TokenKind::Int => {
                self.advance();
                Ok(Type::Int)
            }
            TokenKind::Float => {
                self.advance();
                Ok(Type::Float)
            }
            TokenKind::Bool => {
                self.advance();
                Ok(Type::Bool)
            }
            TokenKind::Char => {
                self.advance();
                Ok(Type::Char)
            }
            TokenKind::String => {
                self.advance();
                Ok(Type::String)
            }
            TokenKind::LeftBracket => {
                self.advance();
                let element_type = self.type_annotation()?;
                if self.match_token(TokenKind::Semicolon) {
                    // [type; size] syntax for fixed-size arrays
                    // For now, ignore size
                    if let TokenKind::Integer(_) = self.peek().kind {
                        self.advance();
                    }
                    self.consume(TokenKind::RightBracket)?;
                    Ok(Type::Array(Box::new(element_type)))
                } else {
                    // [type] syntax for array type annotations
                    self.consume(TokenKind::RightBracket)?;
                    Ok(Type::Array(Box::new(element_type)))
                }
            }
            TokenKind::Ampersand => {
                self.advance();
                let pointee = self.type_annotation()?;
                Ok(Type::Pointer(Box::new(pointee)))
            }
            TokenKind::Star => {
                self.advance();
                let pointee = self.type_annotation()?;
                Ok(Type::Pointer(Box::new(pointee)))
            }
            TokenKind::Identifier(name) => {
                let mut ident = name.clone();
                self.advance();
                
                // Support module-qualified types: module.Type or module::Type
                if self.peek().kind == TokenKind::Dot || self.peek().kind == TokenKind::ColonColon {
                    self.advance(); // consume . or ::
                    if let TokenKind::Identifier(type_name) = &self.peek().kind {
                        // Build qualified name: module.Type
                        ident = format!("{}.{}", ident, type_name);
                        self.advance();
                    } else {
                        return Err(self.error(
                            ErrorKind::ExpectedToken,
                            format!("expected type name after `.`, found `{}`", self.peek().kind),
                        ));
                    }
                }
                
                // Support generic type syntax: Option<T>, Result<T, E>
                if self.match_token(TokenKind::Less) {
                    let mut type_params = vec![self.type_annotation()?];
                    
                    // Parse additional type parameters separated by commas
                    while self.match_token(TokenKind::Comma) {
                        type_params.push(self.type_annotation()?);
                    }
                    
                    self.consume(TokenKind::Greater)?;
                    Ok(Type::Generic { 
                        name: ident, 
                        type_params 
                    })
                }
                // Support dynamic array type annotation: DynamicArray[Type]
                else if ident == "DynamicArray" && self.match_token(TokenKind::LeftBracket) {
                    let element_type = self.type_annotation()?;
                    self.consume(TokenKind::RightBracket)?;
                    Ok(Type::DynamicArray(Box::new(element_type)))
                } else {
                    Ok(Type::Struct(ident))
                }
            }
            _ => Err(self.error(
                ErrorKind::InvalidSyntax,
                format!("expected type, found `{}`", self.peek().kind),
            ).with_suggestion(crate::error::Suggestion::with_example(
                "valid types include",
                "int, float, bool, char, string, [int], *int, MyStruct"
            ))),
        };
        
        let mut ty = ty?;
        
        // Handle pointer syntax: type*
        while self.match_token(TokenKind::Star) {
            ty = Type::Pointer(Box::new(ty));
        }
        
        Ok(ty)
    }
    
    fn struct_def(&mut self) -> Result<Struct, CompilerError> {
        self.consume(TokenKind::Struct)?;
        let name = self.identifier()?;
        self.consume(TokenKind::LeftBrace)?;
        let fields = self.fields()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(Struct { name, fields })
    }
    
    fn enum_def(&mut self) -> Result<Enum, CompilerError> {
        self.consume(TokenKind::Enum)?;
        let name = self.identifier()?;
        self.consume(TokenKind::LeftBrace)?;
        let variants = self.enum_variants()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(Enum { name, variants })
    }
    
    fn enum_variants(&mut self) -> Result<Vec<EnumVariant>, CompilerError> {
        let mut variants = Vec::new();
        let mut next_value: i64 = 0;
        
        while !self.check(TokenKind::RightBrace) {
            let variant_name = self.identifier()?;
            let value = if self.match_token(TokenKind::Equal) {
                // Explicit value
                let lit_token = self.advance();
                match &lit_token.kind {
                    TokenKind::Integer(val) => {
                        next_value = *val + 1;
                        Some(*val)
                    }
                    _ => {
                        return Err(self.error(
                            ErrorKind::ExpectedToken,
                            "expected integer literal after `=` in enum variant".to_string(),
                        ));
                    }
                }
            } else {
                // Implicit value
                let val = next_value;
                next_value += 1;
                Some(val)
            };
            
            variants.push(EnumVariant {
                name: variant_name,
                value,
            });
            
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(variants)
    }
    
    fn fields(&mut self) -> Result<Vec<Field>, CompilerError> {
        let mut fields = Vec::new();
        while !self.check(TokenKind::RightBrace) {
            let name = self.identifier()?;
            self.consume(TokenKind::Colon)?;
            let field_type = self.type_annotation()?;
            fields.push(Field { name, field_type });
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(fields)
    }
    
    fn extern_function(&mut self) -> Result<ExternFunction, CompilerError> {
        self.consume(TokenKind::Extern)?;
        self.consume(TokenKind::Fn)?;
        let name = self.identifier()?;
        self.consume(TokenKind::LeftParen)?;
        let (parameters, variadic) = self.extern_parameters()?;
        self.consume(TokenKind::RightParen)?;
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.type_annotation()?)
        } else {
            None
        };
        self.consume(TokenKind::Semicolon)?;
        Ok(ExternFunction {
            name,
            parameters,
            return_type,
            variadic,
        })
    }
    
    fn import(&mut self) -> Result<Import, CompilerError> {
        self.consume(TokenKind::Import)?;
        let mut module = self.module_segment()?;
        while self.match_token(TokenKind::Dot) {
            module.push('.');
            module.push_str(&self.module_segment()?);
        }
        let alias = if self.match_token(TokenKind::As) {
            Some(self.identifier()?)
        } else {
            None
        };
        Ok(Import { module, alias })
    }

    // Accept module path segments that may coincide with keywords like 'char', 'int', etc.
    fn module_segment(&mut self) -> Result<String, CompilerError> {
        match &self.peek().kind {
            TokenKind::Identifier(name) => { let n = name.clone(); self.advance(); Ok(n) }
            TokenKind::Int => { self.advance(); Ok("int".to_string()) }
            TokenKind::Float => { self.advance(); Ok("float".to_string()) }
            TokenKind::Bool => { self.advance(); Ok("bool".to_string()) }
            TokenKind::Char => { self.advance(); Ok("char".to_string()) }
            TokenKind::String => { self.advance(); Ok("string".to_string()) }
            _ => Err(self.error(
                ErrorKind::ExpectedToken,
                format!("expected module path segment, found `{}`", self.peek().kind),
            )),
        }
    }
    
    fn block(&mut self) -> Result<Vec<Statement>, CompilerError> {
        let mut statements = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            if let TokenKind::Comment(_) = self.peek().kind {
                self.advance(); // Skip comments
                continue;
            }
            statements.push(self.statement()?);
        }
        Ok(statements)
    }
    
    fn statement(&mut self) -> Result<Statement, CompilerError> {
        match self.peek().kind {
            TokenKind::Let => {
                let stmt = self.let_statement()?;
                self.consume(TokenKind::Semicolon)?;
                Ok(stmt)
            }
            TokenKind::Const => {
                let stmt = self.const_statement()?;
                self.consume(TokenKind::Semicolon)?;
                Ok(stmt)
            }
            TokenKind::Return => {
                let stmt = self.return_statement()?;
                self.consume(TokenKind::Semicolon)?;
                Ok(stmt)
            }
            TokenKind::Break => {
                self.advance();
                self.consume(TokenKind::Semicolon)?;
                Ok(Statement::Break)
            }
            TokenKind::Continue => {
                self.advance();
                self.consume(TokenKind::Semicolon)?;
                Ok(Statement::Continue)
            }
            TokenKind::If => self.if_statement(),
            TokenKind::While => self.while_statement(),
            TokenKind::For => self.for_statement(),
            _ => {
                let expr = self.expression()?;
                if self.match_token(TokenKind::Equal) {
                    let value = self.expression()?;
                    self.consume(TokenKind::Semicolon)?;
                    Ok(Statement::Assignment {
                        target: expr,
                        value,
                    })
                } else {
                    self.consume(TokenKind::Semicolon)?;
                    Ok(Statement::Expression(expr))
                }
            }
        }
    }
    
    fn let_statement(&mut self) -> Result<Statement, CompilerError> {
        self.consume(TokenKind::Let)?;
        let mutable = self.match_token(TokenKind::Mut);
        let name = self.identifier()?;
        let var_type = if self.match_token(TokenKind::Colon) {
            Some(self.type_annotation()?)
        } else {
            None
        };
        let initializer = if self.match_token(TokenKind::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        Ok(Statement::Let {
            name,
            var_type,
            mutable,
            initializer,
        })
    }
    
    fn const_statement(&mut self) -> Result<Statement, CompilerError> {
        self.consume(TokenKind::Const)?;
        let name = self.identifier()?;
        self.consume(TokenKind::Colon)?;
        let var_type = self.type_annotation()?;
        self.consume(TokenKind::Equal)?;
        let initializer = self.expression()?;
        Ok(Statement::Const {
            name,
            var_type: Some(var_type),
            initializer,
        })
    }
    
    fn return_statement(&mut self) -> Result<Statement, CompilerError> {
        self.consume(TokenKind::Return)?;
        // Support bare return; detect if next token ends the statement or block
        let value = match self.peek().kind {
            TokenKind::Semicolon | TokenKind::RightBrace => None,
            _ => Some(self.expression()?),
        };
        Ok(Statement::Return(value))
    }
    
    fn if_statement(&mut self) -> Result<Statement, CompilerError> {
        self.consume(TokenKind::If)?;
        let condition = self.expression()?;
        self.consume(TokenKind::LeftBrace)?;
        let then_branch = self.block()?;
        self.consume(TokenKind::RightBrace)?;
        let else_branch = if self.match_token(TokenKind::Else) {
            if self.match_token(TokenKind::If) {
                // else if
                Some(vec![self.if_statement()?])
            } else {
                self.consume(TokenKind::LeftBrace)?;
                let block = self.block()?;
                self.consume(TokenKind::RightBrace)?;
                Some(block)
            }
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn while_statement(&mut self) -> Result<Statement, CompilerError> {
        self.consume(TokenKind::While)?;
        let condition = self.expression()?;
        self.consume(TokenKind::LeftBrace)?;
        let body = self.block()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(Statement::While { condition, body })
    }
    
    fn for_statement(&mut self) -> Result<Statement, CompilerError> {
        self.consume(TokenKind::For)?;
        let variable = self.identifier()?;
        self.consume(TokenKind::Colon)?;
        let iterable = self.expression()?;
        self.consume(TokenKind::LeftBrace)?;
        let body = self.block()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }
    
    fn expression(&mut self) -> Result<Expression, CompilerError> {
        self.ternary()
    }
    
    // ternary -> range ( '?' range ':' ternary )?
    fn ternary(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.range()?;
        if self.match_token(TokenKind::Question) {
            let true_expr = self.range()?;
            self.consume(TokenKind::Colon)?;
            let false_expr = self.ternary()?;  // Recursive call for chaining
            expr = Expression::Ternary {
                condition: Box::new(expr),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            };
        }
        Ok(expr)
    }
    
    fn range(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.logical_or()?;
        if self.match_token(TokenKind::DotDot) {
            let end = self.logical_or()?;
            expr = Expression::Range {
                start: Box::new(expr),
                end: Box::new(end),
            };
        }
        Ok(expr)
    }

    // logical_or -> logical_and ( '||' logical_and )*
    fn logical_or(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.logical_and()?;
        while self.match_token(TokenKind::Or) {
            let right = self.logical_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // logical_and -> equality ( '&&' equality )*
    fn logical_and(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.equality()?;
        while self.match_token(TokenKind::And) {
            let right = self.equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.comparison()?;
        while self.match_tokens(&[TokenKind::EqualEqual, TokenKind::NotEqual]) {
            let operator = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    
    fn comparison(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.term()?;
        while self.match_tokens(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let operator = match self.previous().kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.factor()?;
        while self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = match self.previous().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let operator = match self.previous().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<Expression, CompilerError> {
        if self.match_token(TokenKind::New) {
            if self.match_token(TokenKind::LeftBracket) {
                // new [type]() for dynamic arrays
                let element_type = self.type_annotation()?;
                self.consume(TokenKind::RightBracket)?;
                self.consume(TokenKind::LeftParen)?;
                self.consume(TokenKind::RightParen)?;
                Ok(Expression::DynamicArrayLiteral {
                    element_type: Box::new(element_type),
                    elements: Vec::new(),
                })
            } else {
                let expr = self.unary()?;
                Ok(Expression::New(Box::new(expr)))
            }
        } else if self.match_token(TokenKind::Delete) {
            let expr = self.unary()?;
            Ok(Expression::Delete(Box::new(expr)))
        } else if self.match_tokens(&[TokenKind::Minus, TokenKind::Not, TokenKind::Star, TokenKind::Ampersand]) {
            let operator = match self.previous().kind {
                TokenKind::Minus => UnaryOp::Negate,
                TokenKind::Not => UnaryOp::Not,
                TokenKind::Star => UnaryOp::Dereference,
                TokenKind::Ampersand => UnaryOp::AddressOf,
                _ => unreachable!(),
            };
            let operand = self.unary()?;
            Ok(Expression::Unary {
                operator,
                operand: Box::new(operand),
            })
        } else {
            self.call()
        }
    }
    
    fn call(&mut self) -> Result<Expression, CompilerError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(TokenKind::Dot) {
                let field = self.identifier()?;
                expr = Expression::StructAccess {
                    object: Box::new(expr),
                    field,
                };
            } else if self.match_token(TokenKind::Arrow) {
                // -> is syntactic sugar for (*ptr).field
                let field = self.identifier()?;
                expr = Expression::StructAccess {
                    object: Box::new(Expression::Unary {
                        operator: UnaryOp::Dereference,
                        operand: Box::new(expr),
                    }),
                    field,
                };
            } else if self.match_token(TokenKind::LeftBracket) {
                let index = self.expression()?;
                self.consume(TokenKind::RightBracket)?;
                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(TokenKind::As) {
                // Type casting: expr as Type
                let target_type = self.type_annotation()?;
                expr = Expression::Cast {
                    expression: Box::new(expr),
                    target_type,
                };
            } else if self.match_token(TokenKind::Question) {
                // Error propagation: expr?
                expr = Expression::TryOperator {
                    expression: Box::new(expr),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expression) -> Result<Expression, CompilerError> {
        let mut arguments = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RightParen)?;
        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
        })
    }
    
    fn primary(&mut self) -> Result<Expression, CompilerError> {
        match &self.peek().kind {
            TokenKind::Integer(i) => {
                let i = *i;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(i)))
            }
            TokenKind::FloatLiteral(f) => {
                let f = *f;
                self.advance();
                Ok(Expression::Literal(Literal::Float(f)))
            }
            TokenKind::BoolLiteral(b) => {
                let b = *b;
                self.advance();
                Ok(Expression::Literal(Literal::Bool(b)))
            }
            TokenKind::CharLiteral(c) => {
                let c = *c;
                self.advance();
                Ok(Expression::Literal(Literal::Char(c)))
            }
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Enum access: EnumName::Variant
                if self.match_token(TokenKind::ColonColon) {
                    let variant = self.identifier()?;
                    return Ok(Expression::EnumAccess {
                        enum_name: name,
                        variant,
                    });
                }
                
                // Struct literal: Identifier { field: expr, ... }
                // Heuristic: only treat as struct literal if Identifier starts with uppercase
                if self.check(TokenKind::LeftBrace) && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    self.advance(); // consume '{'
                    let mut fields: Vec<(String, Expression)> = Vec::new();
                    if !self.check(TokenKind::RightBrace) {
                        loop {
                            let field_name = self.identifier()?;
                            self.consume(TokenKind::Colon)?;
                            let value_expr = self.expression()?;
                            fields.push((field_name, value_expr));
                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RightBrace)?;
                    Ok(Expression::StructLiteral { name, fields })
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(TokenKind::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RightBracket)?;
                Ok(Expression::ArrayLiteral(elements))
            }
            TokenKind::Match => {
                self.advance(); // consume 'match'
                let scrutinee = Box::new(self.expression()?);
                self.consume(TokenKind::LeftBrace)?;
                
                let mut arms = Vec::new();
                while !self.check(TokenKind::RightBrace) {
                    let pattern = self.parse_pattern()?;
                    self.consume(TokenKind::FatArrow)?;
                    let expression = self.expression()?;
                    arms.push(crate::ast::MatchArm { pattern, expression });
                    
                    // Comma is optional after the last arm
                    if !self.check(TokenKind::RightBrace) {
                        self.consume(TokenKind::Comma)?;
                    }
                }
                
                self.consume(TokenKind::RightBrace)?;
                Ok(Expression::Match { scrutinee, arms })
            }
            _ => Err(self.error(
                ErrorKind::InvalidSyntax,
                format!("expected expression, found `{}`", self.peek().kind),
            ).with_suggestion(crate::error::Suggestion::with_example(
                "valid expressions include",
                "42, \"hello\", true, [1, 2, 3], my_variable, func_call()"
            ))),
        }
    }
    
    fn identifier(&mut self) -> Result<String, CompilerError> {
        if let TokenKind::Identifier(ref name) = self.peek().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(self.error(
                ErrorKind::ExpectedToken,
                format!("expected identifier, found `{}`", self.peek().kind),
            ).with_suggestion(crate::error::Suggestion::simple(
                "identifiers must start with a letter and can contain letters, numbers, and underscores"
            )))
        }
    }
    
    fn parse_pattern(&mut self) -> Result<crate::ast::Pattern, CompilerError> {
        use crate::ast::Pattern;
        
        match &self.peek().kind {
            // Wildcard pattern: _
            TokenKind::Identifier(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            // Enum variant pattern: EnumName::Variant or EnumName::Variant(binding)
            TokenKind::Identifier(enum_name) => {
                let enum_name = enum_name.clone();
                self.advance();
                
                if self.match_token(TokenKind::ColonColon) {
                    let variant = self.identifier()?;
                    
                    // Check for binding: Option::Some(x)
                    let binding = if self.match_token(TokenKind::LeftParen) {
                        let binding_name = self.identifier()?;
                        self.consume(TokenKind::RightParen)?;
                        Some(binding_name)
                    } else {
                        None
                    };
                    
                    Ok(Pattern::EnumVariant { enum_name, variant, binding })
                } else {
                    // It might be a literal or variable, but for now we don't support variable patterns
                    // Treat it as an error
                    Err(self.error(
                        ErrorKind::InvalidSyntax,
                        format!("unexpected identifier in pattern: `{}`", enum_name),
                    ).with_suggestion(crate::error::Suggestion::simple(
                        "patterns must be enum variants (EnumName::Variant), literals, or wildcards (_)"
                    )))
                }
            }
            // Integer literal pattern
            TokenKind::Integer(val) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(crate::ast::Literal::Integer(val)))
            }
            // Char literal pattern
            TokenKind::CharLiteral(val) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(crate::ast::Literal::Char(val)))
            }
            // String literal pattern
            TokenKind::StringLiteral(val) => {
                let val = val.clone();
                self.advance();
                Ok(Pattern::Literal(crate::ast::Literal::String(val)))
            }
            // Bool literal pattern
            TokenKind::BoolLiteral(val) => {
                let val = *val;
                self.advance();
                Ok(Pattern::Literal(crate::ast::Literal::Bool(val)))
            }
            _ => Err(self.error(
                ErrorKind::ExpectedToken,
                format!("expected pattern, found `{}`", self.peek().kind),
            ).with_suggestion(crate::error::Suggestion::with_example(
                "valid patterns include",
                "EnumName::Variant, 42, 'a', \"hello\", true, _"
            ))),
        }
    }
    
    fn consume(&mut self, kind: TokenKind) -> Result<(), CompilerError> {
        if self.check(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(self.unexpected_token_error(&format!("{}", kind)))
        }
    }
    
    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn check(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek().kind == TokenKind::Eof
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

pub fn parse(tokens: Vec<Token>, file_path: PathBuf) -> Result<Program, CompilerError> {
    let mut parser = Parser::new(tokens, file_path);
    parser.parse()
}