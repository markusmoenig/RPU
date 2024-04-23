use crate::prelude::*;

/// Values in the AST
#[derive(Clone, Copy, Debug)]
pub enum ASTValue {
    None,
    Boolean(bool),
    Int(i64),
}

impl ASTValue {
    pub fn to_type(&self) -> String {
        match self {
            ASTValue::None => "nil".to_string(),
            ASTValue::Boolean(_) => "bool".to_string(),
            ASTValue::Int(_) => "int".to_string(),
        }
    }
}

/// Statements in the AST
#[derive(Clone, Debug)]
pub enum Stmt {
    Print(Box<Expr>),
    Block(Vec<Box<Stmt>>),
    Expression(Box<Expr>),
    VarDeclaration(String, Box<Expr>),
}

/// Expressions in the AST
#[derive(Clone, Debug)]
pub enum Expr {
    Value(ASTValue),
    Unary(UnaryOperator, Box<Expr>),
    Equality(Box<Expr>, EqualityOperator, Box<Expr>),
    Comparison(Box<Expr>, ComparisonOperator, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Grouping(Box<Expr>),
    Variable(String),
    VariableAssignment(String, Box<Expr>),
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

    fn visit_print(&mut self, expression: &Expr, ctx: &mut Context) -> ASTValue;

    fn visit_block(&mut self, list: &Vec<Box<Stmt>>, ctx: &mut Context) -> ASTValue;

    fn visit_expression(&mut self, expression: &Expr, ctx: &mut Context) -> ASTValue;

    fn visit_var_declaration(
        &mut self,
        name: &str,
        expression: &Expr,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_value(&mut self, value: ASTValue, ctx: &mut Context) -> ASTValue;

    fn visit_unary(&mut self, op: &UnaryOperator, expr: &Expr, ctx: &mut Context) -> ASTValue;

    fn visit_equality(
        &mut self,
        left: &Expr,
        op: &EqualityOperator,
        right: &Expr,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_comparison(
        &mut self,
        left: &Expr,
        op: &ComparisonOperator,
        right: &Expr,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_grouping(&mut self, expression: &Expr, ctx: &mut Context) -> ASTValue;

    fn visit_variable(&mut self, name: String, ctx: &mut Context) -> ASTValue;

    fn visit_variable_assignment(
        &mut self,
        name: String,
        expression: &Expr,
        ctx: &mut Context,
    ) -> ASTValue;
}

impl Stmt {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> ASTValue {
        match self {
            Stmt::Print(expression) => visitor.visit_print(expression, ctx),
            Stmt::Block(list) => visitor.visit_block(list, ctx),
            Stmt::Expression(expression) => visitor.visit_expression(expression, ctx),
            Stmt::VarDeclaration(name, initializer) => {
                visitor.visit_var_declaration(name, initializer, ctx)
            }
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> ASTValue {
        match self {
            Expr::Value(value) => visitor.visit_value(*value, ctx),
            Expr::Unary(op, expr) => visitor.visit_unary(op, expr, ctx),
            Expr::Equality(left, op, right) => visitor.visit_equality(left, op, right, ctx),
            Expr::Comparison(left, op, right) => visitor.visit_comparison(left, op, right, ctx),
            Expr::Binary(left, op, right) => visitor.visit_binary(left, op, right, ctx),
            Expr::Grouping(expr) => visitor.visit_grouping(expr, ctx),
            Expr::Variable(name) => visitor.visit_variable(name.clone(), ctx),
            Expr::VariableAssignment(name, expr) => {
                visitor.visit_variable_assignment(name.clone(), expr, ctx)
            }
        }
    }
}
