use crate::prelude::*;

#[macro_export]
macro_rules! empty_expr {
    () => {
        Box::new(Expr::Value(ASTValue::None, Location::default()))
    };
}

/// Values in the AST
#[derive(Clone, Debug)]
pub enum ASTValue {
    None,
    Boolean(Option<String>, bool),
    Int(Option<String>, i32),
    Int2(Option<String>, Box<Expr>, Box<Expr>),
    Int3(Option<String>, Vec3i),
    Int4(Option<String>, Vec4i),
    String(Option<String>, String),
    Function(String, Vec<ASTValue>, Vec<Box<Stmt>>),
}

impl ASTValue {
    /// Returns the RPU type of the given value.
    pub fn to_type(&self) -> String {
        match self {
            ASTValue::None => "nil".to_string(),
            ASTValue::Boolean(_, _) => "bool".to_string(),
            ASTValue::Int(_, _) => "int".to_string(),
            ASTValue::Int2(_, _, _) => "ivec2".to_string(),
            ASTValue::Int3(_, _) => "ivec4".to_string(),
            ASTValue::Int4(_, _) => "ivec4".to_string(),
            ASTValue::String(_, _) => "string".to_string(),
            ASTValue::Function(_, _, _) => "fn".to_string(),
        }
    }

    /// Returns the WAT type of the given value.
    pub fn to_wat_type(&self, pr: &str) -> Option<String> {
        match self {
            ASTValue::Boolean(_, _) => Some(format!("(i{}", pr)),
            ASTValue::Int(_, _) => Some(format!("i{}", pr)),
            ASTValue::Int2(_, _, _) => Some(format!("i{} i{}", pr, pr)),
            ASTValue::Int3(_, _) => Some(format!("i{} i{} i{}", pr, pr, pr)),
            ASTValue::Int4(_, _) => Some(format!("i{} i{} i{} i{}", pr, pr, pr, pr)),
            _ => None,
        }
    }

    /// Creates an ASTValue from a TokenType.
    pub fn from_token_type(token_type: &TokenType) -> ASTValue {
        match token_type {
            TokenType::Nil => ASTValue::None,
            TokenType::True => ASTValue::Boolean(None, true),
            TokenType::False => ASTValue::Boolean(None, false),
            TokenType::Int => ASTValue::Int(None, 0),
            TokenType::Int2 => ASTValue::Int2(None, empty_expr!(), empty_expr!()),
            TokenType::Int3 => ASTValue::Int3(None, Vec3i::new(0, 0, 0)),
            TokenType::Int4 => ASTValue::Int4(None, Vec4i::new(0, 0, 0, 0)),
            TokenType::String => ASTValue::String(None, "".to_string()),
            _ => ASTValue::None,
        }
    }
}

/// Statements in the AST
#[derive(Clone, Debug)]
pub enum Stmt {
    Print(Box<Expr>, Location),
    Block(Vec<Box<Stmt>>, Location),
    Expression(Box<Expr>, Location),
    VarDeclaration(String, Box<Expr>, Location),
    FunctionDeclaration(String, Vec<ASTValue>, Vec<Box<Stmt>>, ASTValue, Location),
}

/// Expressions in the AST
#[derive(Clone, Debug)]
pub enum Expr {
    Value(ASTValue, Location),
    Unary(UnaryOperator, Box<Expr>, Location),
    Equality(Box<Expr>, EqualityOperator, Box<Expr>, Location),
    Comparison(Box<Expr>, ComparisonOperator, Box<Expr>, Location),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>, Location),
    Grouping(Box<Expr>, Location),
    Variable(String, Location),
    VariableAssignment(String, Box<Expr>, Location),
    FunctionCall(Box<Expr>, Vec<Box<Expr>>, Location),
}

/// Unary operators in the AST
#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Negate,
    Minus,
}

/// Binary operators in the AST
#[derive(Clone, Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOperator {
    pub fn describe(&self) -> &str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
        }
    }
}

/// Comparison operators in the AST
#[derive(Clone, Debug)]
pub enum ComparisonOperator {
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

/// Equality operators in the AST
#[derive(Clone, Debug)]
pub enum EqualityOperator {
    NotEqual,
    Equal,
}

/// Visitor trait
pub trait Visitor {
    fn new() -> Self
    where
        Self: Sized;

    fn print(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn block(
        &mut self,
        list: &[Box<Stmt>],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn expression(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn var_declaration(
        &mut self,
        name: &str,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn value(
        &mut self,
        value: ASTValue,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn unary(
        &mut self,
        op: &UnaryOperator,
        expr: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn equality(
        &mut self,
        left: &Expr,
        op: &EqualityOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn comparison(
        &mut self,
        left: &Expr,
        op: &ComparisonOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn grouping(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn variable(
        &mut self,
        name: String,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn variable_assignment(
        &mut self,
        name: String,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn function_call(
        &mut self,
        callee: &Expr,
        args: &[Box<Expr>],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;

    fn function_declaration(
        &mut self,
        name: &str,
        args: &[ASTValue],
        body: &[Box<Stmt>],
        returns: &ASTValue,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String>;
}

impl Stmt {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> Result<ASTValue, String> {
        match self {
            Stmt::Print(expression, loc) => visitor.print(expression, loc, ctx),
            Stmt::Block(list, loc) => visitor.block(list, loc, ctx),
            Stmt::Expression(expression, loc) => visitor.expression(expression, loc, ctx),
            Stmt::VarDeclaration(name, initializer, loc) => {
                visitor.var_declaration(name, initializer, loc, ctx)
            }
            Stmt::FunctionDeclaration(name, args, body, returns, loc) => {
                visitor.function_declaration(name, args, body, returns, loc, ctx)
            }
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> Result<ASTValue, String> {
        match self {
            Expr::Value(value, loc) => visitor.value(value.clone(), loc, ctx),
            Expr::Unary(op, expr, loc) => visitor.unary(op, expr, loc, ctx),
            Expr::Equality(left, op, right, loc) => visitor.equality(left, op, right, loc, ctx),
            Expr::Comparison(left, op, right, loc) => visitor.comparison(left, op, right, loc, ctx),
            Expr::Binary(left, op, right, loc) => visitor.binary(left, op, right, loc, ctx),
            Expr::Grouping(expr, loc) => visitor.grouping(expr, loc, ctx),
            Expr::Variable(name, loc) => visitor.variable(name.clone(), loc, ctx),
            Expr::VariableAssignment(name, expr, loc) => {
                visitor.variable_assignment(name.clone(), expr, loc, ctx)
            }
            Expr::FunctionCall(callee, args, loc) => visitor.function_call(callee, args, loc, ctx),
        }
    }
}

/// Location in the source code
#[derive(Clone, Debug)]
pub struct Location {
    pub file: String,
    pub line: usize,
}

impl Default for Location {
    fn default() -> Self {
        Location {
            file: String::new(),
            line: 0,
        }
    }
}

impl Location {
    pub fn new(file: String, line: usize) -> Self {
        Location { file, line }
    }

    pub fn describe(&self) -> String {
        // format!("in '{}' at line {}.", self.file, self.line)
        format!("at line {}.", self.line)
    }
}
