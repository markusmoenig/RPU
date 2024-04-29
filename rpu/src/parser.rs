use crate::{empty_expr, prelude::*, zero_expr_float, zero_expr_int};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    current_line: usize,

    high_precision: bool,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
            current_line: 0,

            high_precision: true,
        }
    }

    /// Sets high (64 bit) or low (32 bit) precision.
    pub fn set_high_precision(&mut self, high_precision: bool) {
        self.high_precision = high_precision;
    }

    pub fn parse(&mut self, scanner: Scanner) -> Result<String, String> {
        self.extract_tokens(scanner);

        let mut statements = vec![];

        while !self.is_at_end() {
            let stmt = self.declaration()?;
            statements.push(Box::new(stmt));
        }

        let mut visitor = CompileVisitor::new();
        let mut ctx = Context::default();
        ctx.set_high_precision(self.high_precision);

        for statement in statements {
            _ = statement.accept(&mut visitor, &mut ctx)?;
        }

        Ok(ctx.gen_wat())
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        // if self.match_token(vec![TokenType::Fn]) {
        //     self.function()
        // } else if self.match_token(vec![TokenType::Int]) {
        //     self.var_declaration()
        // }

        let mut export = false;

        if self.match_token(vec![TokenType::Export]) {
            export = true;
        }

        if let Some(token_type) = self.match_token_and_return(vec![
            TokenType::Void,
            TokenType::Int,
            TokenType::Int2,
            TokenType::Int3,
            TokenType::Int4,
            TokenType::Float,
            TokenType::Float2,
            TokenType::Float3,
            TokenType::Float4,
        ]) {
            // Decide between function or var declaration

            if !self.is_at_end() && self.tokens[self.current].kind == TokenType::LeftParen {
                self.current -= 1;
                self.statement()
            } else if self.current + 1 < self.tokens.len() {
                if self.tokens[self.current + 1].kind == TokenType::LeftParen {
                    self.function(export, ASTValue::from_token_type(None, &token_type))
                } else {
                    self.var_declaration(ASTValue::from_token_type(None, &token_type))
                }
            } else {
                self.var_declaration(ASTValue::from_token_type(None, &token_type))
            }
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self, static_type: ASTValue) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let line = self.current_line;

        let mut initializer = None;
        if self.match_token(vec![TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        let init = if let Some(i) = initializer {
            Box::new(i)
        } else {
            // TODO If variable is empty, provide the default value for each type (0)
            // The empty expr is just to prevent crashing the compiler
            empty_expr!()
        };

        Ok(Stmt::VarDeclaration(
            name.lexeme,
            static_type,
            init,
            self.create_loc(line),
        ))
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(vec![TokenType::If]) {
            self.if_statement()
        } else if self.match_token(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(vec![TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(vec![TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let line = self.current_line;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after 'if' at line {}.", line),
        )?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            &format!("Expect ')' after if condition at line {}.", line),
        )?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_token(vec![TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(
            Box::new(condition),
            Box::new(then_branch),
            else_branch,
            self.create_loc(line),
        ))
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        let line = self.current_line;
        self.consume(
            TokenType::Semicolon,
            &format!("Expect ';' after print statement at line {}.", line),
        )?;
        Ok(Stmt::Print(Box::new(value), self.create_loc(line)))
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        let line = self.current_line;
        self.consume(
            TokenType::Semicolon,
            &format!("Expect ';' after return value at line {}.", line),
        )?;
        Ok(Stmt::Return(Box::new(value), self.create_loc(line)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        let line = self.current_line;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(Box::new(value), self.create_loc(line)))
    }

    fn function(&mut self, export: bool, returns: ASTValue) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, "Expect function name.")?;
        let line = self.current_line;
        self.consume(TokenType::LeftParen, "Expect '(' after function name.")?;
        let mut parameters = vec![];

        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(format!(
                        "Cannot have more than 255 parameters at line {}.",
                        line
                    ));
                }

                if let Some(token_type) = self.match_token_and_return(vec![
                    TokenType::Void,
                    TokenType::Int,
                    TokenType::Int2,
                    TokenType::Int3,
                    TokenType::Int4,
                    TokenType::Float,
                    TokenType::Float2,
                    TokenType::Float3,
                    TokenType::Float4,
                ]) {
                    let param_name = self
                        .consume(
                            TokenType::Identifier,
                            &format!("Expect parameter name at line {}.", line),
                        )?
                        .lexeme;
                    parameters.push(ASTValue::from_token_type(Some(param_name), &token_type));
                } else {
                    return Err(format!(
                        "Invalid parameter type '{}' at line {}.",
                        self.tokens[self.current].lexeme, line
                    ));
                }

                if !self.match_token(vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.")?;
        if let Stmt::Block(body, _) = self.block()? {
            Ok(Stmt::FunctionDeclaration(
                name.lexeme,
                parameters,
                body,
                returns,
                export,
                self.create_loc(line),
            ))
        } else {
            // Not reachable
            Err("Expect block statement.".to_string())
        }
    }

    fn block(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => {
                    statements.push(Box::new(stmt));
                }
                Err(error) => {
                    println!("{}", error);
                    break;
                }
            }
        }

        let line = self.current_line;

        self.consume(
            TokenType::RightBrace,
            &format!("Expect '}}' after block at line {}.", line),
        )?;

        Ok(Stmt::Block(statements, self.create_loc(line)))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_token(vec![TokenType::Equal]) {
            let equals = self.previous().unwrap();
            let value = self.assignment()?;

            if let Expr::Variable(name, swizzle, _loc) = expr {
                return Ok(Expr::VariableAssignment(
                    name,
                    swizzle.clone(),
                    Box::new(value),
                    self.create_loc(equals.line),
                ));
            }

            return Err(format!(
                "Invalid assignment target: {:?} at line {}.",
                equals, equals.line
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_token(vec![TokenType::Or]) {
            let operator = self.previous().unwrap();
            let right = self.and()?;
            expr = Expr::Logical(
                Box::new(expr),
                Self::operator_to_logical(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            );
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(vec![TokenType::And]) {
            let operator = self.previous().unwrap();
            let right = self.equality()?;
            expr = Expr::Logical(
                Box::new(expr),
                Self::operator_to_logical(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            );
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().unwrap();
            let right = self.comparison()?;
            expr = Expr::Equality(
                Box::new(expr),
                Self::operator_to_equality(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            );
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().unwrap();
            let right = self.term()?;
            expr = Expr::Comparison(
                Box::new(expr),
                Self::operator_to_comparison(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            );
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().unwrap();
            let right = self.factor()?;
            expr = Expr::Binary(
                Box::new(expr),
                Self::operator_to_binary(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            );
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().unwrap();
            let right = self.unary()?;
            expr = Expr::Binary(
                Box::new(expr),
                Self::operator_to_binary(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            );
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().unwrap();
            let right = self.unary()?;
            return Ok(Expr::Unary(
                Self::operator_to_unary(operator.kind),
                Box::new(right),
                self.create_loc(operator.line),
            ));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = vec![];
        let line = self.current_line;

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err("Cannot have more than 255 arguments.".to_string());
                }

                arguments.push(Box::new(self.expression()?));

                if !self.match_token(vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(
            TokenType::RightParen,
            &format!("Expect ')' after function arguments at line {}.", line),
        )?;
        Ok(Expr::FunctionCall(
            Box::new(callee),
            arguments,
            self.create_loc(paren.line),
        ))
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();
        match token.kind {
            TokenType::False => {
                self.advance();
                Ok(Expr::Value(
                    ASTValue::Boolean(None, false),
                    vec![],
                    self.create_loc(token.line),
                ))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Value(
                    ASTValue::Boolean(None, true),
                    vec![],
                    self.create_loc(token.line),
                ))
            }
            TokenType::Void => {
                self.advance();
                Ok(Expr::Value(
                    ASTValue::None,
                    vec![],
                    self.create_loc(token.line),
                ))
            }
            TokenType::Semicolon => Ok(Expr::Value(
                ASTValue::None,
                vec![],
                self.create_loc(token.line),
            )),
            TokenType::IntegerNumber => {
                self.advance();
                if let Ok(number) = token.lexeme.parse::<i32>() {
                    Ok(Expr::Value(
                        ASTValue::Int(None, number),
                        vec![],
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Invalid integer number at line {}.", token.line))
                }
            }
            TokenType::Int2 => {
                self.advance();
                if self.match_token(vec![TokenType::LeftParen]) {
                    let comps = self.read_vec_components(2, token.line)?;
                    let swizzle: Vec<u8> = self.get_swizzle_at_current();

                    Ok(Expr::Value(
                        ASTValue::Int2(
                            Some(format!("{}", comps.len())),
                            if !comps.is_empty() {
                                Box::new(comps[0].clone())
                            } else {
                                zero_expr_int!()
                            },
                            if comps.len() > 1 {
                                Box::new(comps[1].clone())
                            } else {
                                zero_expr_int!()
                            },
                        ),
                        swizzle,
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Expected '(' after ivec2 at line {}.", token.line))
                }
            }
            TokenType::Int3 => {
                self.advance();
                if self.match_token(vec![TokenType::LeftParen]) {
                    let comps = self.read_vec_components(3, token.line)?;
                    let swizzle: Vec<u8> = self.get_swizzle_at_current();

                    Ok(Expr::Value(
                        ASTValue::Int3(
                            Some(format!("{}", comps.len())),
                            if !comps.is_empty() {
                                Box::new(comps[0].clone())
                            } else {
                                zero_expr_int!()
                            },
                            if comps.len() > 1 {
                                Box::new(comps[1].clone())
                            } else {
                                zero_expr_int!()
                            },
                            if comps.len() > 2 {
                                Box::new(comps[2].clone())
                            } else {
                                zero_expr_int!()
                            },
                        ),
                        swizzle,
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Expected '(' after ivec3 at line {}.", token.line))
                }
            }
            TokenType::Int4 => {
                self.advance();
                if self.match_token(vec![TokenType::LeftParen]) {
                    let comps = self.read_vec_components(4, token.line)?;
                    let swizzle: Vec<u8> = self.get_swizzle_at_current();

                    Ok(Expr::Value(
                        ASTValue::Int4(
                            Some(format!("{}", comps.len())),
                            if !comps.is_empty() {
                                Box::new(comps[0].clone())
                            } else {
                                zero_expr_int!()
                            },
                            if comps.len() > 1 {
                                Box::new(comps[1].clone())
                            } else {
                                zero_expr_int!()
                            },
                            if comps.len() > 2 {
                                Box::new(comps[2].clone())
                            } else {
                                zero_expr_int!()
                            },
                            if comps.len() > 3 {
                                Box::new(comps[3].clone())
                            } else {
                                zero_expr_int!()
                            },
                        ),
                        swizzle,
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Expected '(' after ivec4 at line {}.", token.line))
                }
            }
            TokenType::FloatNumber => {
                self.advance();
                if let Ok(number) = token.lexeme.parse::<f32>() {
                    Ok(Expr::Value(
                        ASTValue::Float(None, number),
                        vec![],
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Invalid float number at line {}.", token.line))
                }
            }
            TokenType::Float2 => {
                self.advance();
                if self.match_token(vec![TokenType::LeftParen]) {
                    let comps = self.read_vec_components(2, token.line)?;
                    let swizzle: Vec<u8> = self.get_swizzle_at_current();

                    Ok(Expr::Value(
                        ASTValue::Float2(
                            Some(format!("{}", comps.len())),
                            if !comps.is_empty() {
                                Box::new(comps[0].clone())
                            } else {
                                zero_expr_float!()
                            },
                            if comps.len() > 1 {
                                Box::new(comps[1].clone())
                            } else {
                                zero_expr_float!()
                            },
                        ),
                        swizzle,
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Expected '(' after vec2 at line {}.", token.line))
                }
            }
            TokenType::Float3 => {
                self.advance();
                if self.match_token(vec![TokenType::LeftParen]) {
                    let comps = self.read_vec_components(3, token.line)?;
                    let swizzle: Vec<u8> = self.get_swizzle_at_current();

                    Ok(Expr::Value(
                        ASTValue::Float3(
                            Some(format!("{}", comps.len())),
                            if !comps.is_empty() {
                                Box::new(comps[0].clone())
                            } else {
                                zero_expr_float!()
                            },
                            if comps.len() > 1 {
                                Box::new(comps[1].clone())
                            } else {
                                zero_expr_float!()
                            },
                            if comps.len() > 2 {
                                Box::new(comps[2].clone())
                            } else {
                                zero_expr_float!()
                            },
                        ),
                        swizzle,
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Expected '(' after vec3 at line {}.", token.line))
                }
            }
            TokenType::Float4 => {
                self.advance();
                if self.match_token(vec![TokenType::LeftParen]) {
                    let comps = self.read_vec_components(4, token.line)?;
                    let swizzle: Vec<u8> = self.get_swizzle_at_current();

                    Ok(Expr::Value(
                        ASTValue::Float4(
                            Some(format!("{}", comps.len())),
                            if !comps.is_empty() {
                                Box::new(comps[0].clone())
                            } else {
                                zero_expr_float!()
                            },
                            if comps.len() > 1 {
                                Box::new(comps[1].clone())
                            } else {
                                zero_expr_float!()
                            },
                            if comps.len() > 2 {
                                Box::new(comps[2].clone())
                            } else {
                                zero_expr_float!()
                            },
                            if comps.len() > 3 {
                                Box::new(comps[3].clone())
                            } else {
                                zero_expr_float!()
                            },
                        ),
                        swizzle,
                        self.create_loc(token.line),
                    ))
                } else {
                    Err(format!("Expected '(' after vec4 at line {}.", token.line))
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                if self.match_token(vec![TokenType::RightParen]) {
                    Ok(Expr::Grouping(Box::new(expr), self.create_loc(token.line)))
                } else {
                    Err("Expected ')' after expression".to_string())
                }
            }
            TokenType::Identifier => {
                self.advance();

                let swizzle: Vec<u8> = self.get_swizzle_at_current();
                Ok(Expr::Variable(
                    token.lexeme,
                    swizzle,
                    self.create_loc(token.line),
                ))
            }
            _ => Err(format!(
                "Unknown identifier {:?} at line {}.",
                token.lexeme, token.line
            )),
        }
    }

    /// Reads the components of a vector up to `max_comps` components. Can terminate early if closing parenthesis is found.
    /// Check for component validity is done in the compiler.
    fn read_vec_components(&mut self, max_comps: usize, line: usize) -> Result<Vec<Expr>, String> {
        let mut components = vec![];
        let mut count = 0;

        if self.match_token(vec![TokenType::RightParen]) {
            return Ok(components);
        }

        while count < max_comps {
            let expr = self.expression()?;
            components.push(expr);
            count += 1;

            if !self.match_token(vec![TokenType::Comma]) {
                if !self.match_token(vec![TokenType::RightParen]) {
                    return Err(format!(
                        "Expected ')' after vector components at line {}.",
                        line
                    ));
                }
                break;
            }
        }

        Ok(components)
    }

    /// Returns the swizzle at the current token if any.
    pub fn get_swizzle_at_current(&mut self) -> Vec<u8> {
        let mut swizzle: Vec<u8> = vec![];

        if self.current + 1 < self.tokens.len()
            && self.tokens[self.current].kind == TokenType::Dot
            && self.tokens[self.current + 1].kind == TokenType::Identifier
        {
            let swizzle_token = self.tokens[self.current + 1].lexeme.clone();
            if swizzle_token
                .chars()
                .all(|c| matches!(c, 'x' | 'y' | 'z' | 'w'))
            {
                swizzle = swizzle_token
                    .chars()
                    .map(|c| match c {
                        'x' => 0,
                        'y' => 1,
                        'z' => 2,
                        'w' => 3,
                        _ => unreachable!(),
                    })
                    .collect();
                self.current += 2;
            }
        }

        swizzle
    }

    /// Extract a potential swizzle from the variable name.
    fn _extract_swizzle(input: &str) -> (&str, Vec<u8>) {
        if let Some(pos) = input.rfind('.') {
            let (base, swizzle) = input.split_at(pos);
            let swizzle = &swizzle[1..]; // Skip the dot

            // Check if all characters in the swizzle are 'x', 'y', 'z', or 'w'
            if swizzle.chars().all(|c| matches!(c, 'x' | 'y' | 'z' | 'w')) {
                // Map 'x', 'y', 'z', 'w' to 0, 1, 2, 3 respectively
                let swizzle_bytes = swizzle
                    .chars()
                    .map(|c| match c {
                        'x' => 0,
                        'y' => 1,
                        'z' => 2,
                        'w' => 3,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<u8>>();

                return (base, swizzle_bytes);
            }
        }
        (input, Vec::new())
    }

    /// Extract all tokens from the scanner.
    pub fn extract_tokens(&mut self, mut scanner: Scanner) {
        // Extract all tokens from the scanner
        let mut tokens = vec![];
        loop {
            let token = scanner.scan_token();
            if token.kind == TokenType::Eof {
                //tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        self.tokens = tokens;
    }

    fn consume(&mut self, kind: TokenType, message: &str) -> Result<Token, String> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            Err(message.to_string())
        }
    }

    fn match_token(&mut self, expected: Vec<TokenType>) -> bool {
        if expected.iter().any(|&kind| self.check(kind)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_token_and_return(&mut self, expected: Vec<TokenType>) -> Option<TokenType> {
        for &kind in &expected {
            if self.check(kind) {
                self.advance();
                return Some(kind);
            }
        }
        None
    }

    fn check(&self, kind: TokenType) -> bool {
        self.current < self.tokens.len() && self.tokens[self.current].kind == kind
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            self.current_line = self.tokens[self.current].line;
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Token {
        if self.is_at_end() {
            Token {
                kind: TokenType::Eof,
                lexeme: "".to_string(),
                line: 0,
            }
        } else {
            self.tokens[self.current].clone()
        }
    }

    fn previous(&self) -> Option<Token> {
        if self.current > 0 {
            Some(self.tokens[self.current - 1].clone())
        } else {
            None
        }
    }

    fn operator_to_unary(operator: TokenType) -> UnaryOperator {
        match operator {
            TokenType::Bang => UnaryOperator::Negate,
            TokenType::Minus => UnaryOperator::Minus,
            _ => unreachable!(),
        }
    }

    fn operator_to_binary(operator: TokenType) -> BinaryOperator {
        match operator {
            TokenType::Plus => BinaryOperator::Add,
            TokenType::Minus => BinaryOperator::Subtract,
            TokenType::Star => BinaryOperator::Multiply,
            TokenType::Slash => BinaryOperator::Divide,
            _ => unreachable!(),
        }
    }

    fn operator_to_comparison(operator: TokenType) -> ComparisonOperator {
        match operator {
            TokenType::Greater => ComparisonOperator::Greater,
            TokenType::GreaterEqual => ComparisonOperator::GreaterEqual,
            TokenType::Less => ComparisonOperator::Less,
            TokenType::LessEqual => ComparisonOperator::LessEqual,
            _ => unreachable!(),
        }
    }

    fn operator_to_equality(operator: TokenType) -> EqualityOperator {
        match operator {
            TokenType::BangEqual => EqualityOperator::NotEqual,
            TokenType::EqualEqual => EqualityOperator::Equal,
            _ => unreachable!(),
        }
    }

    fn operator_to_logical(operator: TokenType) -> LogicalOperator {
        match operator {
            TokenType::And => LogicalOperator::And,
            TokenType::Or => LogicalOperator::Or,
            _ => unreachable!(),
        }
    }

    /// Create a location for the given line number.
    fn create_loc(&self, line: usize) -> Location {
        Location {
            file: "main".to_string(),
            line,
        }
    }
}
