use crate::prelude::*;

/// CompileVisitor
pub struct CompileVisitor;

impl Visitor for CompileVisitor {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {}
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
        _list: &[Box<Stmt>],
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
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
            //println!("E {:?}", e);
        }
        e
    }

    fn visit_var_declaration(
        &mut self,
        _name: &str,
        _expression: &Expr,
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }

    fn visit_value(
        &mut self,
        value: ASTValue,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let instr = match value {
            ASTValue::Int(i) => format!("(i64.const {})", i),
            ASTValue::Boolean(f) => format!("(f64.const {})", f),
            ASTValue::None => "".to_string(),
        };

        if ctx.verbose {
            println!("V {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));
        Ok(ASTValue::None)
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
        left.accept(self, ctx);
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
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        _ = left.accept(self, ctx)?;
        _ = right.accept(self, ctx)?;

        let instr = match op {
            BinaryOperator::Add => "i64.add",
            BinaryOperator::Subtract => "i64.sub",
            BinaryOperator::Multiply => "i64.mul",
            BinaryOperator::Divide => "i64.div",
        };

        if ctx.verbose {
            println!("B {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));

        Ok(ASTValue::None)
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
        _name: String,
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }

    fn visit_variable_assignment(
        &mut self,
        _name: String,
        _expression: &Expr,
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }
}
