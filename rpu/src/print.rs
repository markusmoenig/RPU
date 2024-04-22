use crate::prelude::*;

/// PrintVisitor
pub struct PrintVisitor;

impl Visitor for PrintVisitor {
    fn visit_print(&mut self, expression: &Expr) -> Value {
        print!("-- Print ");
        expression.accept(self);
        println!(" --");

        Value::None
    }

    fn visit_expression(&mut self, expression: &Expr) -> Value {
        print!("-- Expression ");
        expression.accept(self);
        println!(" --");

        Value::None
    }

    fn visit_value(&mut self, value: Value) -> Value {
        print!("{:?}", value);
        Value::None
    }

    fn visit_equality(&mut self, left: &Expr, op: &EqualityOperator, right: &Expr) -> Value {
        print!("-- Equality ");
        left.accept(self);
        match op {
            EqualityOperator::NotEqual => print!(" != "),
            EqualityOperator::Equal => print!(" == "),
        }
        right.accept(self);
        println!(" --");

        Value::None
    }

    fn visit_comparison(&mut self, left: &Expr, op: &ComparisonOperator, right: &Expr) -> Value {
        print!("-- Comparison ");
        left.accept(self);
        match op {
            ComparisonOperator::Greater => print!(" > "),
            ComparisonOperator::GreaterEqual => print!(" >= "),
            ComparisonOperator::Less => print!(" < "),
            ComparisonOperator::LessEqual => print!(" <= "),
        }
        right.accept(self);
        println!(" --");

        Value::None
    }

    fn visit_binary(&mut self, left: &Expr, op: &BinaryOperator, right: &Expr) -> Value {
        print!("-- Binary ");
        left.accept(self);
        match op {
            BinaryOperator::Add => print!(" + "),
            BinaryOperator::Subtract => print!(" - "),
            BinaryOperator::Multiply => print!(" * "),
            BinaryOperator::Divide => print!(" / "),
        }
        right.accept(self);
        println!(" --");

        Value::None
    }

    fn visit_grouping(&mut self, expression: &Expr) -> Value {
        print!("-- ( ");
        expression.accept(self);
        println!(" ) --");

        Value::None
    }
}
