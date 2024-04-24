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

    fn print(
        &mut self,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        print!("-- Print ");
        expression.accept(self, ctx)?;
        println!(" --");

        Ok(ASTValue::None)
    }

    fn block(
        &mut self,
        _list: &[Box<Stmt>],
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }

    fn expression(
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

    fn var_declaration(
        &mut self,
        _name: &str,
        _expression: &Expr,
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }

    fn value(
        &mut self,
        value: ASTValue,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let instr = match value {
            ASTValue::Int(i) => format!("(i{}.const {})", ctx.precision.describe(), i),
            ASTValue::Boolean(f) => format!("(f{}.const {})", ctx.precision.describe(), f),
            ASTValue::None => "".to_string(),
            ASTValue::String(_) => "".to_string(),
            ASTValue::Function(_, _, _) => "".to_string(),
        };

        if ctx.verbose {
            println!("V {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));
        Ok(ASTValue::None)
    }

    fn unary(
        &mut self,
        op: &UnaryOperator,
        expr: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        print!("-- Unary ");
        expr.accept(self, ctx)?;
        match op {
            UnaryOperator::Negate => print!(" ! "),
            UnaryOperator::Minus => print!(" - "),
        }
        println!(" --");

        Ok(ASTValue::None)
    }

    fn equality(
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

    fn comparison(
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

    fn binary(
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
            BinaryOperator::Add => format!("(i{}.add)", ctx.precision.describe()),
            BinaryOperator::Subtract => format!("(i{}.sub)", ctx.precision.describe()),
            BinaryOperator::Multiply => format!("(i{}.mul)", ctx.precision.describe()),
            BinaryOperator::Divide => format!("(i{}.div)", ctx.precision.describe()),
        };

        if ctx.verbose {
            println!("B {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));

        Ok(ASTValue::None)
    }

    fn grouping(
        &mut self,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        expression.accept(self, ctx)
    }

    fn variable(
        &mut self,
        name: String,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let instr = format!("(local.get ${})", name);

        if ctx.verbose {
            println!("V {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));

        Ok(ASTValue::None)
    }

    fn variable_assignment(
        &mut self,
        _name: String,
        _expression: &Expr,
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }

    fn function_call(
        &mut self,
        _callee: &Expr,
        _args: &[Box<Expr>],
        _loc: &Location,
        _ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        Ok(ASTValue::None)
    }

    fn function_declaration(
        &mut self,
        name: &str,
        args: &[Parameter],
        body: &[Box<Stmt>],
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let mut params = String::new();

        for param in args {
            match param {
                Parameter::Int(name) => {
                    params += &format!("{} (param ${} i{})", params, name, ctx.precision.describe())
                }
            }
        }

        let instr = format!(
            "(func ${} (export \"{}\") {} (result i64)",
            name, name, params
        );
        if ctx.verbose {
            println!("{}", instr);
        }
        ctx.wat.push_str(&format!("{}\n", instr));

        for stmt in body {
            stmt.accept(self, ctx)?;
        }

        ctx.wat.push_str(")\n");

        if ctx.verbose {
            println!(")");
        }

        Ok(ASTValue::None)
    }
}
