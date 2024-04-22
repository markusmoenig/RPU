/// Values in the AST
#[derive(Clone, Copy, Debug)]
pub enum Value {
    None,
    Boolean(bool),
    I64(i64),
}

/// Statements in the AST
#[derive(Clone, Debug)]
pub enum Stmt {
    Print(Box<Expr>),
    Expression(Box<Expr>),
}

/// Expressions in the AST
#[derive(Clone, Debug)]
pub enum Expr {
    Value(Value),
    Unary(UnaryOperator, Box<Expr>),
    Equality(Box<Expr>, EqualityOperator, Box<Expr>),
    Comparison(Box<Expr>, ComparisonOperator, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Grouping(Box<Expr>),
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
    fn visit_print(&mut self, expression: &Expr) -> Value;
    fn visit_expression(&mut self, expression: &Expr) -> Value;

    fn visit_value(&mut self, value: Value) -> Value;
    fn visit_equality(&mut self, left: &Expr, op: &EqualityOperator, right: &Expr) -> Value;
    fn visit_comparison(&mut self, left: &Expr, op: &ComparisonOperator, right: &Expr) -> Value;
    fn visit_binary(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr) -> Value;
    fn visit_grouping(&mut self, expression: &Expr) -> Value;
}

impl Stmt {
    pub fn accept(&self, visitor: &mut dyn Visitor) -> Value {
        match self {
            Stmt::Print(expression) => visitor.visit_print(expression),
            Stmt::Expression(expression) => visitor.visit_expression(expression),
            _ => unimplemented!(),
        }
    }
}

impl Expr {
    pub fn accept(&self, visitor: &mut dyn Visitor) -> Value {
        match self {
            Expr::Value(value) => visitor.visit_value(*value),
            Expr::Equality(left, op, right) => visitor.visit_equality(left, op, right),
            Expr::Comparison(left, op, right) => visitor.visit_comparison(left, op, right),
            Expr::Binary(left, op, right) => visitor.visit_binary(left, op, right),
            Expr::Grouping(expression) => visitor.visit_grouping(expression),
            _ => unimplemented!(),
        }
    }
}
