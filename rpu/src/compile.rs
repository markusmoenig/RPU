use crate::empty_expr;
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
        let mut rc = ASTValue::None;
        let instr;

        match value {
            ASTValue::Boolean(_, f) => {
                instr = format!("(f{}.const {})", ctx.pr, f);
                rc = ASTValue::Boolean(None, f);
            }
            ASTValue::Int(_, i) => {
                instr = format!("(i{}.const {})", ctx.pr, i);
                rc = ASTValue::Int(None, i);
            }
            ASTValue::Int2(_, x, y) => {
                _ = x.accept(self, ctx)?;
                _ = y.accept(self, ctx)?;
                instr = "".to_string();
                rc = ASTValue::Int2(None, empty_expr!(), empty_expr!());
            }
            _ => {
                instr = "".to_string();
            }
        };

        if ctx.verbose {
            println!("V {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));
        Ok(rc)
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
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let left_type = left.accept(self, ctx)?;
        let right_type = right.accept(self, ctx)?;
        let mut rc = ASTValue::None;

        //println!("{:?} {:?}", left_type, right_type);

        let instr = match (&left_type, &right_type) {
            (ASTValue::Int(_, _), ASTValue::Int(_, _)) => {
                rc = ASTValue::Int(None, 0);
                match op {
                    BinaryOperator::Add => format!("(i{}.add)", ctx.precision.describe()),
                    BinaryOperator::Subtract => format!("(i{}.sub)", ctx.precision.describe()),
                    BinaryOperator::Multiply => format!("(i{}.mul)", ctx.precision.describe()),
                    BinaryOperator::Divide => format!("(i{}.div)", ctx.precision.describe()),
                }
            }
            (ASTValue::Int(_, _), ASTValue::Int2(_, _, _)) => {
                rc = ASTValue::Int2(None, empty_expr!(), empty_expr!());
                match op {
                    // Scalar and ivec2
                    BinaryOperator::Add => {
                        ctx.gen_scalar_vec2(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_scalar_add_vec2_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_scalar_vec2(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_scalar_sub_vec2_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_scalar_vec2(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_scalar_mul_vec2_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_scalar_vec2(&format!("i{}", ctx.pr), "div_s");
                        format!("(call $_rpu_scalar_div_s_vec2_i{})", ctx.pr)
                    }
                }
            }
            (ASTValue::Int2(_, _, _), ASTValue::Int(_, _)) => {
                rc = ASTValue::Int2(None, empty_expr!(), empty_expr!());
                match op {
                    // Scalar and ivec2
                    BinaryOperator::Add => {
                        ctx.gen_vec2_scalar(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_vec2_add_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec2_scalar(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec2_sub_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec2_scalar(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec2_mul_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec2_scalar(&format!("i{}", ctx.pr), "div_s");
                        format!("(call $_rpu_vec2_div_s_scalar_i{})", ctx.pr)
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Invalid types {:?} {:?} for operator '{}' {}",
                    left_type.to_type(),
                    right_type.to_type(),
                    op.describe(),
                    loc.describe()
                ))
            }
        };

        if ctx.verbose {
            println!("B {}", instr);
        }

        ctx.wat.push_str(&format!("{}\n", instr));

        Ok(rc)
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
        args: &[ASTValue],
        body: &[Box<Stmt>],
        returns: &ASTValue,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let mut params = String::new();

        for param in args {
            match param {
                ASTValue::Int(name, _) => {
                    params += &format!(
                        "{} (param ${} i{})",
                        params,
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    )
                }
                _ => {}
            }
        }

        let mut return_type = String::new();

        if let Some(r) = returns.to_wat_type(&ctx.pr) {
            return_type = format!("(result {})", r);
        }

        let instr = format!(
            "(func ${} (export \"{}\") {} {}",
            name, name, params, return_type
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
