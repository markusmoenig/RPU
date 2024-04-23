use crate::prelude::*;

/// InterpretVisitor
pub struct InterpretVisitor {
    environment: Environment,
}

impl Visitor for InterpretVisitor {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            environment: Environment::default(),
        }
    }

    fn visit_print(&mut self, expression: &Expr, ctx: &mut Context) -> ASTValue {
        print!("-- Print ");
        expression.accept(self, ctx);
        println!(" --");

        ASTValue::None
    }

    fn visit_block(&mut self, list: &Vec<Box<Stmt>>, ctx: &mut Context) -> ASTValue {
        self.environment.begin_scope();
        for stmt in list {
            stmt.accept(self, ctx);
        }
        self.environment.end_scope();
        ASTValue::None
    }

    fn visit_expression(&mut self, expression: &Expr, ctx: &mut Context) -> ASTValue {
        let e = expression.accept(self, ctx);
        if ctx.verbose {
            println!("E {:?}", e);
        }
        e
    }

    fn visit_var_declaration(
        &mut self,
        name: &str,
        expression: &Expr,
        ctx: &mut Context,
    ) -> ASTValue {
        let v = expression.accept(self, ctx);

        if ctx.verbose {
            println!("{} {} = {:?}", v.to_type(), name, v);
        }

        self.environment.define(name.to_string(), v);

        ASTValue::None
    }

    fn visit_value(&mut self, value: ASTValue, _ctx: &mut Context) -> ASTValue {
        value
    }

    fn visit_unary(&mut self, op: &UnaryOperator, expr: &Expr, ctx: &mut Context) -> ASTValue {
        print!("-- Unary ");
        expr.accept(self, ctx);
        match op {
            UnaryOperator::Negate => print!(" ! "),
            UnaryOperator::Minus => print!(" - "),
        }
        println!(" --");

        ASTValue::None
    }

    fn visit_equality(
        &mut self,
        left: &Expr,
        op: &EqualityOperator,
        right: &Expr,
        ctx: &mut Context,
    ) -> ASTValue {
        print!("-- Equality ");
        left.accept(self, ctx);
        match op {
            EqualityOperator::NotEqual => print!(" != "),
            EqualityOperator::Equal => print!(" == "),
        }
        right.accept(self, ctx);
        println!(" --");

        ASTValue::None
    }

    fn visit_comparison(
        &mut self,
        left: &Expr,
        op: &ComparisonOperator,
        right: &Expr,
        ctx: &mut Context,
    ) -> ASTValue {
        print!("-- Comparison ");
        left.accept(self, ctx);
        match op {
            ComparisonOperator::Greater => print!(" > "),
            ComparisonOperator::GreaterEqual => print!(" >= "),
            ComparisonOperator::Less => print!(" < "),
            ComparisonOperator::LessEqual => print!(" <= "),
        }
        right.accept(self, ctx);
        println!(" --");

        ASTValue::None
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
        ctx: &mut Context,
    ) -> ASTValue {
        let lv = left.accept(self, ctx);
        let rv = right.accept(self, ctx);

        if ctx.verbose {
            println!("B {:?} {} {:?}", lv, op.describe(), rv);
        }

        match op {
            BinaryOperator::Add => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => ASTValue::Int(l + r),
                _ => ASTValue::None,
            },
            BinaryOperator::Subtract => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => ASTValue::Int(l - r),
                _ => ASTValue::None,
            },
            BinaryOperator::Multiply => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => ASTValue::Int(l * r),
                _ => ASTValue::None,
            },
            BinaryOperator::Divide => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => ASTValue::Int(l / r),
                _ => ASTValue::None,
            },
        }
    }

    fn visit_grouping(&mut self, expression: &Expr, ctx: &mut Context) -> ASTValue {
        expression.accept(self, ctx)
    }

    fn visit_variable(&mut self, name: String, ctx: &mut Context) -> ASTValue {
        if let Some(v) = self.environment.get(&name) {
            if ctx.verbose {
                println!("V {} ({:?})", name, v);
            }
            v
        } else {
            ASTValue::None
        }
    }

    fn visit_variable_assignment(
        &mut self,
        name: String,
        expression: &Expr,
        ctx: &mut Context,
    ) -> ASTValue {
        let v = expression.accept(self, ctx);
        if ctx.verbose {
            println!("A {} ({:?})", name, v);
        }
        self.environment.assign(&name, v);

        ASTValue::None
    }
}
