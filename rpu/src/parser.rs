use crate::prelude::*;
use wasmer::{imports, Instance, Module, Store, Value};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    current_line: usize,
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
        }
    }

    pub fn parse(&mut self, scanner: Scanner) {
        self.extract_tokens(scanner);

        //println!("{:?}", self.tokens);

        let mut statements = vec![];

        while !self.is_at_end() {
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

        let mut visitor = CompileVisitor::new();
        let mut ctx = Context::default();

        for statement in statements {
            let _rc = statement.accept(&mut visitor, &mut ctx);

            let wat = ctx.gen_wat();
            let mut store = Store::default();
            let module_rc = Module::new(&store, &wat);
            match module_rc {
                Ok(module) => {
                    let import_object = imports! {};
                    if let Ok(instance) = Instance::new(&mut store, &module, &import_object) {
                        if let Ok(main) = instance.exports.get_function("main") {
                            let rc = main.call(&mut store, &[Value::I64(42)]);
                            println!("rc {:?}", rc);
                            //println!("Result: {:?}", result);
                            //}
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {:?}", err.to_string());
                }
            }
            //println!("{:?}", rc);
        }
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(vec![TokenType::Fn]) {
            self.function()
        } else if self.match_token(vec![TokenType::Int]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
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

        Ok(Stmt::VarDeclaration(
            name.lexeme,
            Box::new(initializer.unwrap()),
            self.create_loc(line),
        ))
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.match_token(vec![TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        let line = self.current_line;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Box::new(value), self.create_loc(line)))
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        let line = self.current_line;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(Box::new(value), self.create_loc(line)))
    }

    fn function(&mut self) -> Result<Stmt, String> {
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

                if self.check(TokenType::Int) {
                    self.advance();
                    let param_name = self
                        .consume(
                            TokenType::Identifier,
                            &format!("Expect parameter name at line {}.", line),
                        )?
                        .lexeme;
                    parameters.push(Parameter::Int(param_name));
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
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(Stmt::Block(statements, self.create_loc(line)))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;

        if self.match_token(vec![TokenType::Equal]) {
            let equals = self.previous().unwrap();
            let value = self.assignment()?;

            if let Expr::Variable(name, _loc) = expr {
                return Ok(Expr::VariableAssignment(
                    name,
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

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
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
                    ASTValue::Boolean(false),
                    self.create_loc(token.line),
                ))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Value(
                    ASTValue::Boolean(true),
                    self.create_loc(token.line),
                ))
            }
            TokenType::Number => {
                self.advance();
                if let Ok(number) = token.lexeme.parse::<i64>() {
                    Ok(Expr::Value(
                        ASTValue::Int(number),
                        self.create_loc(token.line),
                    ))
                } else {
                    Err("Invalid Number".to_string())
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
                Ok(Expr::Variable(token.lexeme, self.create_loc(token.line)))
            }
            _ => Err(format!(
                "Unknown identifier {:?} at line {}.",
                token.lexeme, token.line
            )),
        }
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

    /// Create a location for the given line number.
    fn create_loc(&self, line: usize) -> Location {
        Location {
            file: "main".to_string(),
            line,
        }
    }
}
