/// Expressions in the AST
#[derive(Clone, Debug)]
pub enum Expr {
    False,
    True,
    Nil,
    Literal(i64),
    Unary(UnaryOperator, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
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

/// Visitor trait
trait Visitor {
    fn visit_literal(&mut self, value: i64);
    fn visit_binary(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr);
}

impl Expr {
    fn accept(&self, visitor: &mut dyn Visitor) {
        match self {
            Expr::Literal(value) => visitor.visit_literal(*value),
            Expr::Binary(left, op, right) => visitor.visit_binary(left, op, right),
            _ => unimplemented!(),
        }
    }
}
