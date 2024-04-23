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
    Print(Box<Expr>, Location),
    Block(Vec<Box<Stmt>>, Location),
    Expression(Box<Expr>, Location),
    VarDeclaration(String, Box<Expr>, Location),
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

    fn visit_print(&mut self, expression: &Expr, loc: &Location, ctx: &mut Context) -> ASTValue;

    fn visit_block(&mut self, list: &[Box<Stmt>], loc: &Location, ctx: &mut Context) -> ASTValue;

    fn visit_expression(
        &mut self,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_var_declaration(
        &mut self,
        name: &str,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_value(&mut self, value: ASTValue, loc: &Location, ctx: &mut Context) -> ASTValue;

    fn visit_unary(
        &mut self,
        op: &UnaryOperator,
        expr: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_equality(
        &mut self,
        left: &Expr,
        op: &EqualityOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_comparison(
        &mut self,
        left: &Expr,
        op: &ComparisonOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;

    fn visit_grouping(&mut self, expression: &Expr, loc: &Location, ctx: &mut Context) -> ASTValue;

    fn visit_variable(&mut self, name: String, loc: &Location, ctx: &mut Context) -> ASTValue;

    fn visit_variable_assignment(
        &mut self,
        name: String,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> ASTValue;
}

impl Stmt {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> ASTValue {
        match self {
            Stmt::Print(expression, loc) => visitor.visit_print(expression, loc, ctx),
            Stmt::Block(list, loc) => visitor.visit_block(list, loc, ctx),
            Stmt::Expression(expression, loc) => visitor.visit_expression(expression, loc, ctx),
            Stmt::VarDeclaration(name, initializer, loc) => {
                visitor.visit_var_declaration(name, initializer, loc, ctx)
            }
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut dyn Visitor, ctx: &mut Context) -> ASTValue {
        match self {
            Expr::Value(value, loc) => visitor.visit_value(*value, loc, ctx),
            Expr::Unary(op, expr, loc) => visitor.visit_unary(op, expr, loc, ctx),
            Expr::Equality(left, op, right, loc) => {
                visitor.visit_equality(left, op, right, loc, ctx)
            }
            Expr::Comparison(left, op, right, loc) => {
                visitor.visit_comparison(left, op, right, loc, ctx)
            }
            Expr::Binary(left, op, right, loc) => visitor.visit_binary(left, op, right, loc, ctx),
            Expr::Grouping(expr, loc) => visitor.visit_grouping(expr, loc, ctx),
            Expr::Variable(name, loc) => visitor.visit_variable(name.clone(), loc, ctx),
            Expr::VariableAssignment(name, expr, loc) => {
                visitor.visit_variable_assignment(name.clone(), expr, loc, ctx)
            }
        }
    }
}

/// Location in the source code
#[derive(Clone, Debug)]
pub struct Location {
    pub file: String,
    pub line: usize,
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
