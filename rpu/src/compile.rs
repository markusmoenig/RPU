use crate::empty_expr;
use crate::prelude::*;

/// CompileVisitor
pub struct CompileVisitor {
    environment: Environment,
}

impl Visitor for CompileVisitor {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            environment: Environment::default(),
        }
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

    fn expression(
        &mut self,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        expression.accept(self, ctx)
    }

    fn var_declaration(
        &mut self,
        name: &str,
        expression: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let v = expression.accept(self, ctx)?;

        match &v {
            ASTValue::Int(_, _) => {
                let instr = format!("(local ${} i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Int2(_, _, _) => {
                let instr = format!("(local ${}_x i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_y i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}_y", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_x", name);
                ctx.add_wat(&instr);
            }
            _ => {}
        }

        self.environment.define(name.to_string(), v);

        Ok(ASTValue::None)
    }

    fn variable_assignment(
        &mut self,
        name: String,
        swizzle: &[u8],
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let mut v = expression.accept(self, ctx)?;

        let incoming_components = v.components();

        // Use the type of the variable
        if let Some(vv) = self.environment.get(&name) {
            v = vv;
        }

        if swizzle.is_empty() {
            if incoming_components != v.components() {
                return Err(format!(
                    "Variable '{}' has {} components, but expression has {} {}",
                    name,
                    v.components(),
                    incoming_components,
                    loc.describe()
                ));
            }
        } else if incoming_components != swizzle.len() {
            return Err(format!(
                "Variable '{}' has {} swizzle, but expression has {} components {}",
                name,
                swizzle.len(),
                incoming_components,
                loc.describe()
            ));
        }

        match &v {
            ASTValue::Int(_, _) => {
                let instr = format!("local.set ${}", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Int2(_, _, _) => {
                if swizzle.is_empty() {
                    let instr = format!("local.set ${}_y", name);
                    ctx.add_wat(&instr);
                    let instr = format!("local.set ${}_x", name);
                    ctx.add_wat(&instr);
                } else {
                    for s in swizzle.iter().rev() {
                        match s {
                            0 => {
                                let instr = format!("local.set ${}_x", name);
                                ctx.add_wat(&instr);
                            }
                            1 => {
                                let instr = format!("local.set ${}_y", name);
                                ctx.add_wat(&instr);
                            }
                            _ => {
                                return Err(format!(
                                    "Swizzle '{}' out of range for '{}' {}",
                                    ctx.deswizzle(*s),
                                    name,
                                    loc.describe()
                                ))
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        self.environment.assign(&name, v);

        Ok(ASTValue::None)
    }

    fn variable(
        &mut self,
        name: String,
        swizzle: &[u8],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let instr = String::new();
        let mut rc = ASTValue::None;

        if swizzle.len() > 4 {
            return Err(format!(
                "Maximal swizzle length is 4, got {} for '{}' {}",
                swizzle.len(),
                name,
                loc.describe()
            ));
        }

        if !swizzle.is_empty() {
            rc = ctx.create_value_from_swizzle(swizzle.len());
        }

        if let Some(v) = self.environment.get(&name) {
            match &v {
                ASTValue::Int(_, _) => {
                    let instr = format!("local.get ${}", name);
                    ctx.add_wat(&instr);
                    rc = ASTValue::Int(None, 0);
                }
                ASTValue::Int2(_, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("local.get ${}_x", name);
                        ctx.add_wat(&instr);
                        let instr = format!("local.get ${}_y", name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Int2(None, empty_expr!(), empty_expr!());
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("local.get ${}_x", name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("local.get ${}_y", name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        ctx.add_wat(&instr);

        Ok(rc)
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

        ctx.add_wat(&instr);
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
        let rc;

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

        ctx.add_wat(&instr);

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
                        "(param ${} i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    )
                }
                ASTValue::Int2(name, _, _) => {
                    params += &format!(
                        "(param ${}_x i{}) (param ${}_y i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
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

        ctx.add_line();
        ctx.add_wat(&format!(";; function '{}'", name));
        ctx.add_wat(&instr);
        ctx.add_indention();

        ctx.wat_locals = String::new();

        ctx.wat.push_str("__LOCALS__\n");

        for stmt in body {
            stmt.accept(self, ctx)?;
        }

        ctx.wat = ctx.wat.replace("__LOCALS__", &ctx.wat_locals);

        ctx.remove_indention();
        ctx.add_wat(")");

        Ok(ASTValue::None)
    }
}
