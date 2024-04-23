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

    fn visit_print(
        &mut self,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        print!("-- Print ");
        expression.accept(self, ctx);
        println!(" --");

        Ok(ASTValue::None)
    }

    fn visit_block(
        &mut self,
        list: &[Box<Stmt>],
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        self.environment.begin_scope();
        for stmt in list {
            stmt.accept(self, ctx)?;
        }
        self.environment.end_scope();
        Ok(ASTValue::None)
    }

    fn visit_expression(
        &mut self,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
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
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let v = expression.accept(self, ctx)?;

        if ctx.verbose {
            println!("{} {} = {:?}", v.to_type(), name, v);
        }

        self.environment.define(name.to_string(), v);

        Ok(ASTValue::None)
    }

    fn visit_value(
        &mut self,
        value: ASTValue,
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(value)
    }

    fn visit_unary(
        &mut self,
        op: &UnaryOperator,
        expr: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        print!("-- Unary ");
        expr.accept(self, ctx);
        match op {
            UnaryOperator::Negate => print!(" ! "),
            UnaryOperator::Minus => print!(" - "),
        }
        println!(" --");

        Ok(ASTValue::None)
    }

    fn visit_equality(
        &mut self,
        left: &Expr,
        op: &EqualityOperator,
        right: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        print!("-- Equality ");
        left.accept(self, ctx)?;
        match op {
            EqualityOperator::NotEqual => print!(" != "),
            EqualityOperator::Equal => print!(" == "),
        }
        right.accept(self, ctx)?;
        println!(" --");

        Ok(ASTValue::None)
    }

    fn visit_comparison(
        &mut self,
        left: &Expr,
        op: &ComparisonOperator,
        right: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        print!("-- Comparison ");
        left.accept(self, ctx)?;
        match op {
            ComparisonOperator::Greater => print!(" > "),
            ComparisonOperator::GreaterEqual => print!(" >= "),
            ComparisonOperator::Less => print!(" < "),
            ComparisonOperator::LessEqual => print!(" <= "),
        }
        right.accept(self, ctx)?;
        println!(" --");

        Ok(ASTValue::None)
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        op: &BinaryOperator,
        right: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let lv = left.accept(self, ctx)?;
        let rv = right.accept(self, ctx)?;

        if ctx.verbose {
            println!("B {:?} {} {:?}", lv, op.describe(), rv);
        }

        match op {
            BinaryOperator::Add => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => Ok(ASTValue::Int(l + r)),
                _ => Err(format!("Invalid operands for '+' {}", loc.describe())),
            },
            BinaryOperator::Subtract => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => Ok(ASTValue::Int(l - r)),
                _ => Err(format!("Invalid operands for '-' {}", loc.describe())),
            },
            BinaryOperator::Multiply => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => Ok(ASTValue::Int(l * r)),
                _ => Err(format!("Invalid operands for '*' {}", loc.describe())),
            },
            BinaryOperator::Divide => match (lv, rv) {
                (ASTValue::Int(l), ASTValue::Int(r)) => Ok(ASTValue::Int(l / r)),
                _ => Err(format!("Invalid operands for '+' {}", loc.describe())),
            },
        }
    }

    fn visit_grouping(
        &mut self,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        expression.accept(self, ctx)
    }

    fn visit_variable(
        &mut self,
        name: String,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        if let Some(v) = self.environment.get(&name) {
            if ctx.verbose {
                println!("V {} ({:?})", name, v);
            }
            Ok(v)
        } else {
            Err(format!("Unknown variable '{}' {}", name, loc.describe()))
        }
    }

    fn visit_variable_assignment(
        &mut self,
        name: String,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let v = expression.accept(self, ctx)?;
        if ctx.verbose {
            println!("A {} ({:?})", name, v);
        }
        self.environment.assign(&name, v);

        Ok(ASTValue::None)
    }
}
