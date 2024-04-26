use crate::empty_expr;
use crate::prelude::*;

/// CompileVisitor
pub struct CompileVisitor {
    environment: Environment,
    functions: FxHashMap<String, ASTValue>,
}

impl Visitor for CompileVisitor {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            environment: Environment::default(),
            functions: FxHashMap::default(),
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
        self.environment.begin_scope(ASTValue::None);
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
        } else {
            // Check for function call
            if let Some(ASTValue::Function(name, args, body)) = self.functions.get(&name) {
                rc = ASTValue::Function(name.clone(), args.clone(), body.clone());
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
        left.accept(self, ctx)?;
        right.accept(self, ctx)?;

        let instr = match op {
            EqualityOperator::NotEqual => format!("(i{}.ne)", ctx.pr),
            EqualityOperator::Equal => format!("(i{}.eq)", ctx.pr),
        };

        ctx.add_wat(&instr);

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
        let left_value = left.accept(self, ctx)?;
        let right_value = right.accept(self, ctx)?;

        let instr = match op {
            ComparisonOperator::Greater => format!("(i{}.gt_s)", ctx.pr),
            ComparisonOperator::GreaterEqual => format!("(i{}.ge_s)", ctx.pr),
            ComparisonOperator::Less => format!("(i{}.lt_s)", ctx.pr),
            ComparisonOperator::LessEqual => format!("(i{}.le_s)", ctx.pr),
        };

        ctx.add_wat(&instr);

        Ok(left_value)
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
        callee: &Expr,
        args: &[Box<Expr>],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let callee = callee.accept(self, ctx)?;
        let mut rc = ASTValue::None;

        if let ASTValue::Function(name, func_args, returns) = callee {
            if func_args.len() != args.len() {
                return Err(format!(
                    "Function '{}' expects {} arguments, but {} were provided {}",
                    name,
                    func_args.len(),
                    args.len(),
                    loc.describe()
                ));
            }

            for index in 0..args.len() {
                let rc = args[index].accept(self, ctx)?;
                if rc.to_type() != func_args[index].to_type() {
                    return Err(format!(
                        "Function '{}' expects argument {} to be of type '{}', but '{}' was provided {}",
                        name,
                        index,
                        func_args[index].to_type(),
                        rc.to_type(),
                        loc.describe()
                    ));
                }
            }

            let instr = format!("(call ${})", name);
            ctx.add_wat(&instr);
            rc = *returns;
        }

        Ok(rc)
    }

    fn function_declaration(
        &mut self,
        name: &str,
        args: &[ASTValue],
        body: &[Box<Stmt>],
        returns: &ASTValue,
        export: &bool,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        self.functions.insert(
            name.to_string(),
            ASTValue::Function(name.to_string(), args.to_vec(), Box::new(returns.clone())),
        );

        let mut params = String::new();

        ctx.wat_locals = String::new();
        self.environment.begin_scope(returns.clone());

        for param in args {
            match param {
                ASTValue::Int(name, _) => {
                    params += &format!(
                        "(param ${} i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                    self.environment
                        .define(name.clone().unwrap(), ASTValue::Int(None, 0));
                }
                ASTValue::Int2(name, _, _) => {
                    params += &format!(
                        "(param ${}_x i{}) (param ${}_y i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                }
                _ => {}
            }
        }

        let mut return_type = String::new();

        if let Some(r) = returns.to_wat_type(&ctx.pr) {
            return_type = format!("(result {})", r);
        }

        let export_str = if *export {
            format!(" (export \"{}\")", name)
        } else {
            "".to_string()
        };

        let instr = format!("(func ${}{} {} {}", name, export_str, params, return_type);

        ctx.add_line();
        ctx.add_wat(&format!(";; function '{}'", name));
        ctx.add_wat(&instr);
        ctx.add_indention();

        ctx.wat.push_str("__LOCALS__\n");

        let mut last_value = ASTValue::None;
        for stmt in body {
            last_value = stmt.accept(self, ctx)?;
        }

        if let Some(ret) = self.environment.get_return() {
            if ret.to_type() != "void" && last_value.to_type() != ret.to_type() {
                return Err(format!(
                    "Function '{}' does not end with a 'return' statement {}",
                    name,
                    loc.describe()
                ));
            }
        }

        self.environment.end_scope();

        ctx.wat = ctx.wat.replace("__LOCALS__", &ctx.wat_locals);
        ctx.wat_locals = String::new();

        ctx.remove_indention();
        ctx.add_wat(")");

        Ok(ASTValue::None)
    }

    fn return_stmt(
        &mut self,
        expr: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let rc = expr.accept(self, ctx)?;

        if let Some(ret) = self.environment.get_return() {
            if rc.to_type() != ret.to_type() {
                return Err(format!(
                    "Invalid return type '{}', should be '{}' {}",
                    rc.to_type(),
                    ret.to_type(),
                    loc.describe()
                ));
            }
        }

        ctx.add_wat("(return)");

        Ok(rc)
    }

    fn if_stmt(
        &mut self,
        cond: &Expr,
        then_stmt: &Stmt,
        else_stmt: &Option<Box<Stmt>>,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        ctx.add_line();
        let _rc = cond.accept(self, ctx)?;

        ctx.add_line();

        // let instr = format!("(if (result i{})", ctx.pr);
        let instr = "(if".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        let instr = "(then".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        let _ = then_stmt.accept(self, ctx)?;

        ctx.remove_indention();
        ctx.add_wat(")");
        if let Some(es) = else_stmt {
            let instr = "(else".to_string();
            ctx.add_wat(&instr);
            ctx.add_indention();
            let _ = es.accept(self, ctx)?;
            ctx.remove_indention();
            ctx.add_wat(")");
        }

        ctx.remove_indention();
        ctx.add_wat(")");
        ctx.add_line();

        Ok(ASTValue::None)
    }

    fn logical_expr(
        &mut self,
        left: &Expr,
        _op: &LogicalOperator,
        right: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let _l = left.accept(self, ctx)?;
        let _r = right.accept(self, ctx)?;

        //if op == &LogicalOperator::And {}

        Ok(ASTValue::None)
    }
}
