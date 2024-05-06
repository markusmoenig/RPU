use crate::empty_expr;
use crate::prelude::*;

/// CompileVisitor
pub struct CompileVisitor {
    environment: Environment,
    functions: FxHashMap<String, ASTValue>,
    break_depth: Vec<i32>,
}

impl Visitor for CompileVisitor {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut functions: FxHashMap<String, ASTValue> = FxHashMap::default();

        functions.insert(
            "length".to_string(),
            ASTValue::Function(
                "length".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "normalize".to_string(),
            ASTValue::Function(
                "normalize".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "smoothstep".to_string(),
            ASTValue::Function(
                "smoothstep".to_string(),
                vec![ASTValue::None, ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "mix".to_string(),
            ASTValue::Function(
                "mix".to_string(),
                vec![ASTValue::None, ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "dot".to_string(),
            ASTValue::Function(
                "dot".to_string(),
                vec![ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "cross".to_string(),
            ASTValue::Function(
                "cross".to_string(),
                vec![ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "sqrt".to_string(),
            ASTValue::Function(
                "sqrt".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "sin".to_string(),
            ASTValue::Function(
                "sin".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "cos".to_string(),
            ASTValue::Function(
                "cos".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "ceil".to_string(),
            ASTValue::Function(
                "ceil".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "floor".to_string(),
            ASTValue::Function(
                "floor".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "fract".to_string(),
            ASTValue::Function(
                "fract".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "abs".to_string(),
            ASTValue::Function(
                "abs".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "tan".to_string(),
            ASTValue::Function(
                "tan".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "degrees".to_string(),
            ASTValue::Function(
                "degrees".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "radians".to_string(),
            ASTValue::Function(
                "radians".to_string(),
                vec![ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "rand".to_string(),
            ASTValue::Function("rand".to_string(), vec![], Box::new(ASTValue::None)),
        );

        functions.insert(
            "max".to_string(),
            ASTValue::Function(
                "max".to_string(),
                vec![ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "min".to_string(),
            ASTValue::Function(
                "min".to_string(),
                vec![ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        functions.insert(
            "pow".to_string(),
            ASTValue::Function(
                "pow".to_string(),
                vec![ASTValue::None, ASTValue::None],
                Box::new(ASTValue::None),
            ),
        );

        Self {
            environment: Environment::default(),
            functions,
            break_depth: vec![],
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
        let instr = "(block".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        if let Some(d) = self.break_depth.last() {
            self.break_depth.push(d + 1);
        }

        self.environment.begin_scope(ASTValue::None);
        for stmt in list {
            stmt.accept(self, ctx)?;
        }
        self.environment.end_scope();

        if let Some(d) = self.break_depth.last() {
            self.break_depth.push(d - 1);
        }

        ctx.remove_indention();
        ctx.add_wat(")");

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
        static_type: &ASTValue,
        expression: &Expr,
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let v = expression.accept(self, ctx)?;

        // Compare incoming expression type with the static type.
        if v.to_type() != static_type.to_type() {
            return Err(format!(
                "Variable '{}' has type '{}', but expression has type '{}' {}",
                name,
                static_type.to_type(),
                v.to_type(),
                loc.describe()
            ));
        }

        // Global function definition. We write these out in the module header in gen_wat().
        if self.environment.is_global_scope() {
            ctx.globals.insert(name.to_string(), v.clone());
            return Ok(ASTValue::None);
        }

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
            ASTValue::Int3(_, _, _, _) => {
                let instr = format!("(local ${}_x i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_y i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_z i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}_z", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_y", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_x", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Int4(_, _, _, _, _) => {
                let instr = format!("(local ${}_x i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_y i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_z i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_w i{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}_w", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_z", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_y", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_x", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Float(_, _) => {
                let instr = format!("(local ${} f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Float2(_, _, _) => {
                let instr = format!("(local ${}_x f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_y f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}_y", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_x", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Float3(_, _, _, _) => {
                let instr = format!("(local ${}_x f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_y f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_z f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}_z", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_y", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_x", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Float4(_, _, _, _, _) => {
                let instr = format!("(local ${}_x f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_y f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_z f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));
                let instr = format!("(local ${}_w f{})", name, ctx.pr);
                ctx.wat_locals.push_str(&format!("        {}\n", instr));

                let instr = format!("local.set ${}_w", name);
                ctx.add_wat(&instr);
                let instr = format!("local.set ${}_z", name);
                ctx.add_wat(&instr);
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
            if swizzle.is_empty() && incoming_components != vv.components() {
                return Err(format!(
                    "Variable '{}' has {} components, but expression has {} {}",
                    name,
                    v.components(),
                    incoming_components,
                    loc.describe()
                ));
            }
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
            ASTValue::Int(_, _) | ASTValue::Float(_, _) => {
                let instr = format!("local.set ${}", name);
                ctx.add_wat(&instr);
            }
            ASTValue::Int2(_, _, _) | ASTValue::Float2(_, _, _) => {
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
            ASTValue::Int3(_, _, _, _) | ASTValue::Float3(_, _, _, _) => {
                if swizzle.is_empty() {
                    let instr = format!("local.set ${}_z", name);
                    ctx.add_wat(&instr);
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
                            2 => {
                                let instr = format!("local.set ${}_z", name);
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
            ASTValue::Int4(_, _, _, _, _) | ASTValue::Float4(_, _, _, _, _) => {
                if swizzle.is_empty() {
                    let instr = format!("local.set ${}_w", name);
                    ctx.add_wat(&instr);
                    let instr = format!("local.set ${}_z", name);
                    ctx.add_wat(&instr);
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
                            2 => {
                                let instr = format!("local.set ${}_z", name);
                                ctx.add_wat(&instr);
                            }
                            3 => {
                                let instr = format!("local.set ${}_w", name);
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

        let mut scope = "local";

        // Check if the variable is in the environment
        let mut vv = self.environment.get(&name);

        // Check if the variable is a global
        if vv.is_none() {
            if let Some(global_value) = ctx.globals.get(&name) {
                scope = "global";
                vv = Some(global_value.clone());
            }
        }

        if let Some(v) = vv {
            if !swizzle.is_empty() {
                rc = ctx.create_value_from_swizzle(&v, swizzle.len());
            }
            match &v {
                ASTValue::Int(_, _) => {
                    let instr = format!("{}.get ${}", scope, name);
                    ctx.add_wat(&instr);
                    rc = ASTValue::Int(None, 0);
                }
                ASTValue::Int2(_, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("{}.get ${}_x", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_y", scope, name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Int2(None, empty_expr!(), empty_expr!());
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("{}.get ${}_x", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("{}.get ${}_y", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ASTValue::Int3(_, _, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("{}.get ${}_x", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_y", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_z", scope, name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Int3(None, empty_expr!(), empty_expr!(), empty_expr!());
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("{}.get ${}_x", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("{}.get ${}_y", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                2 => {
                                    let instr = format!("{}.get ${}_z", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ASTValue::Int4(_, _, _, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("{}.get ${}_x", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_y", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_z", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_w", scope, name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Int4(
                            None,
                            empty_expr!(),
                            empty_expr!(),
                            empty_expr!(),
                            empty_expr!(),
                        );
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("{}.get ${}_x", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("{}.get ${}_y", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                2 => {
                                    let instr = format!("{}.get ${}_z", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                3 => {
                                    let instr = format!("{}.get ${}_w", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ASTValue::Float(_, _) => {
                    let instr = format!("{}.get ${}", scope, name);
                    ctx.add_wat(&instr);
                    rc = ASTValue::Float(None, 0.0);
                }
                ASTValue::Float2(_, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("{}.get ${}_x", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_y", scope, name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Float2(None, empty_expr!(), empty_expr!());
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("{}.get ${}_x", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("{}.get ${}_y", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ASTValue::Float3(_, _, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("{}.get ${}_x", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_y", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_z", scope, name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Float3(None, empty_expr!(), empty_expr!(), empty_expr!());
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("{}.get ${}_x", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("{}.get ${}_y", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                2 => {
                                    let instr = format!("{}.get ${}_z", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ASTValue::Float4(_, _, _, _, _) => {
                    if swizzle.is_empty() {
                        let instr = format!("{}.get ${}_x", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_y", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_z", scope, name);
                        ctx.add_wat(&instr);
                        let instr = format!("{}.get ${}_w", scope, name);
                        ctx.add_wat(&instr);
                        rc = ASTValue::Float4(
                            None,
                            empty_expr!(),
                            empty_expr!(),
                            empty_expr!(),
                            empty_expr!(),
                        );
                    } else {
                        for s in swizzle {
                            match s {
                                0 => {
                                    let instr = format!("{}.get ${}_x", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                1 => {
                                    let instr = format!("{}.get ${}_y", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                2 => {
                                    let instr = format!("{}.get ${}_z", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                3 => {
                                    let instr = format!("{}.get ${}_w", scope, name);
                                    ctx.add_wat(&instr);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                _ => {}
            }
        } else if let Some(ASTValue::Function(name, args, body)) = self.functions.get(&name) {
            rc = ASTValue::Function(name.clone(), args.clone(), body.clone());
        }

        if !instr.is_empty() {
            ctx.add_wat(&instr);
        }

        Ok(rc)
    }

    fn value(
        &mut self,
        value: ASTValue,
        swizzle: &[u8],
        loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let mut rc = ASTValue::None;
        let instr;

        if swizzle.len() > 4 {
            return Err(format!(
                "Maximal swizzle length is 4, got {} for constant {}",
                swizzle.len(),
                loc.describe()
            ));
        }

        if !swizzle.is_empty() {
            rc = ctx.create_value_from_swizzle(&value, swizzle.len());
        }

        match value {
            ASTValue::Boolean(_, f) => {
                instr = format!("(f{}.const {})", ctx.pr, f);
                rc = ASTValue::Boolean(None, f);
            }
            ASTValue::Int(_, i) => {
                instr = format!("(i{}.const {})", ctx.pr, i);
                rc = ASTValue::Int(None, i);
            }
            ASTValue::Int2(comps_txt, x, y) => {
                instr = "".to_string();
                let comps: i32 = comps_txt.unwrap().parse().unwrap();
                if swizzle.is_empty() {
                    let _x = x.accept(self, ctx)?;
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = y.accept(self, ctx)?;
                        }
                    } else {
                        _ = y.accept(self, ctx)?;
                    }
                    rc = ASTValue::Int2(None, x, y);
                } else {
                    for s in swizzle {
                        match s {
                            0 => {
                                _ = x.accept(self, ctx)?;
                            }
                            1 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = y.accept(self, ctx)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ASTValue::Int3(comps_txt, x, y, z) => {
                instr = "".to_string();
                let comps: i32 = comps_txt.unwrap().parse().unwrap();
                if swizzle.is_empty() {
                    let _x = x.accept(self, ctx)?;
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = y.accept(self, ctx)?;
                        }
                    } else {
                        _ = y.accept(self, ctx)?;
                    }
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = z.accept(self, ctx)?;
                        }
                    } else {
                        _ = z.accept(self, ctx)?;
                    }
                    rc = ASTValue::Int3(None, x, y, z);
                } else {
                    for s in swizzle {
                        match s {
                            0 => {
                                _ = x.accept(self, ctx)?;
                            }
                            1 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = y.accept(self, ctx)?;
                                }
                            }
                            2 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = z.accept(self, ctx)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ASTValue::Int4(comps_txt, x, y, z, w) => {
                instr = "".to_string();
                let comps: i32 = comps_txt.unwrap().parse().unwrap();
                if swizzle.is_empty() {
                    let _x = x.accept(self, ctx)?;
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = y.accept(self, ctx)?;
                        }
                    } else {
                        _ = y.accept(self, ctx)?;
                    }
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = z.accept(self, ctx)?;
                        }
                    } else {
                        _ = z.accept(self, ctx)?;
                    }
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = w.accept(self, ctx)?;
                        }
                    } else {
                        _ = w.accept(self, ctx)?;
                    }
                    rc = ASTValue::Int4(None, x, y, z, w);
                } else {
                    for s in swizzle {
                        match s {
                            0 => {
                                _ = x.accept(self, ctx)?;
                            }
                            1 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = y.accept(self, ctx)?;
                                }
                            }
                            2 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = z.accept(self, ctx)?;
                                }
                            }
                            3 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = w.accept(self, ctx)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ASTValue::Float(_, i) => {
                instr = format!("(f{}.const {})", ctx.pr, i);
                rc = ASTValue::Float(None, i);
            }
            ASTValue::Float2(comps_txt, x, y) => {
                instr = "".to_string();
                let comps: i32 = comps_txt.unwrap().parse().unwrap();
                if swizzle.is_empty() {
                    let _x = x.accept(self, ctx)?;
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = y.accept(self, ctx)?;
                        }
                    } else {
                        _ = y.accept(self, ctx)?;
                    }
                    rc = ASTValue::Float2(None, x, y);
                } else {
                    for s in swizzle {
                        match s {
                            0 => {
                                _ = x.accept(self, ctx)?;
                            }
                            1 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = y.accept(self, ctx)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ASTValue::Float3(comps_txt, x, y, z) => {
                instr = "".to_string();
                let comps: i32 = comps_txt.unwrap().parse().unwrap();
                if swizzle.is_empty() {
                    let _x = x.accept(self, ctx)?;
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = y.accept(self, ctx)?;
                        }
                    } else {
                        _ = y.accept(self, ctx)?;
                    }
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = z.accept(self, ctx)?;
                        }
                    } else {
                        _ = z.accept(self, ctx)?;
                    }
                    rc = ASTValue::Float3(None, x, y, z);
                } else {
                    for s in swizzle {
                        match s {
                            0 => {
                                _ = x.accept(self, ctx)?;
                            }
                            1 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = y.accept(self, ctx)?;
                                }
                            }
                            2 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = z.accept(self, ctx)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ASTValue::Float4(comps_txt, x, y, z, w) => {
                instr = "".to_string();
                let comps: i32 = comps_txt.unwrap().parse().unwrap();
                if swizzle.is_empty() {
                    let _x = x.accept(self, ctx)?;
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = y.accept(self, ctx)?;
                        }
                    } else {
                        _ = y.accept(self, ctx)?;
                    }
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = z.accept(self, ctx)?;
                        }
                    } else {
                        _ = z.accept(self, ctx)?;
                    }
                    if comps == 1 {
                        if _x.components() == 1 {
                            // Fill with x
                            _ = x.accept(self, ctx)?;
                        } else {
                            // Fill with zero
                            _ = w.accept(self, ctx)?;
                        }
                    } else {
                        _ = w.accept(self, ctx)?;
                    }
                    rc = ASTValue::Float4(None, x, y, z, w);
                } else {
                    for s in swizzle {
                        match s {
                            0 => {
                                _ = x.accept(self, ctx)?;
                            }
                            1 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = y.accept(self, ctx)?;
                                }
                            }
                            2 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = z.accept(self, ctx)?;
                                }
                            }
                            3 => {
                                if comps == 1 {
                                    _ = x.accept(self, ctx)?;
                                } else {
                                    _ = w.accept(self, ctx)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            _ => {
                instr = "".to_string();
            }
        };

        if !instr.is_empty() && !self.environment.is_global_scope() {
            ctx.add_wat(&instr);
        }
        Ok(rc)
    }

    fn unary(
        &mut self,
        _op: &UnaryOperator,
        expr: &Expr,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        let v = expr.accept(self, ctx)?;

        // !, - have the same behavior right now.
        let func_name = ctx.gen_vec_operation(v.components() as u32, "neg");
        let instr = format!("(call ${})", func_name);
        ctx.add_wat(&instr);

        Ok(v)
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

        // TODO Check if we can compare these two values

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
        let _right_value = right.accept(self, ctx)?;

        // TODO Check if we can compare these two values

        let is_float_based = left_value.is_float_based();

        let instr = if !is_float_based {
            match op {
                ComparisonOperator::Greater => format!("(i{}.gt_s)", ctx.pr),
                ComparisonOperator::GreaterEqual => format!("(i{}.ge_s)", ctx.pr),
                ComparisonOperator::Less => format!("(i{}.lt_s)", ctx.pr),
                ComparisonOperator::LessEqual => format!("(i{}.le_s)", ctx.pr),
            }
        } else {
            match op {
                ComparisonOperator::Greater => format!("(f{}.gt)", ctx.pr),
                ComparisonOperator::GreaterEqual => format!("(f{}.ge)", ctx.pr),
                ComparisonOperator::Less => format!("(f{}.lt)", ctx.pr),
                ComparisonOperator::LessEqual => format!("(f{}.le)", ctx.pr),
            }
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

        let instr = match (&left_type, &right_type) {
            // Int x Int
            (ASTValue::Int(_, _), ASTValue::Int(_, _)) => {
                rc = ASTValue::Int(None, 0);
                match op {
                    BinaryOperator::Add => format!("(i{}.add)", ctx.precision.describe()),
                    BinaryOperator::Subtract => format!("(i{}.sub)", ctx.precision.describe()),
                    BinaryOperator::Multiply => format!("(i{}.mul)", ctx.precision.describe()),
                    BinaryOperator::Divide => format!("(i{}.div)", ctx.precision.describe()),
                }
            }
            // Int x Int2
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
            // Int2 x Int
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
            // Int x Int3
            (ASTValue::Int(_, _), ASTValue::Int3(_, _, _, _)) => {
                rc = ASTValue::Int3(None, empty_expr!(), empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_scalar_vec3(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_scalar_add_vec3_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_scalar_vec3(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_scalar_sub_vec3_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_scalar_vec3(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_scalar_mul_vec3_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_scalar_vec3(&format!("i{}", ctx.pr), "div_s");
                        format!("(call $_rpu_scalar_div_s_vec3_i{})", ctx.pr)
                    }
                }
            }
            // Int3 x Int
            (ASTValue::Int3(_, _, _, _), ASTValue::Int(_, _)) => {
                rc = ASTValue::Int3(None, empty_expr!(), empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec3_scalar(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_vec3_add_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec3_scalar(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec3_sub_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec3_scalar(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec3_mul_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec3_scalar(&format!("i{}", ctx.pr), "div_s");
                        format!("(call $_rpu_vec3_div_s_scalar_i{})", ctx.pr)
                    }
                }
            }
            // Int x Int4
            (ASTValue::Int(_, _), ASTValue::Int4(_, _, _, _, _)) => {
                rc = ASTValue::Int4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                );
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_scalar_vec4(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_scalar_add_vec4_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_scalar_vec4(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_scalar_sub_vec4_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_scalar_vec4(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_scalar_mul_vec4_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_scalar_vec4(&format!("i{}", ctx.pr), "div_s");
                        format!("(call $_rpu_scalar_div_s_vec4_i{})", ctx.pr)
                    }
                }
            }
            // Int4 x Int
            (ASTValue::Int4(_, _, _, _, _), ASTValue::Int(_, _)) => {
                rc = ASTValue::Int4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                );
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec4_scalar(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_vec4_add_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec4_scalar(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec4_sub_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec4_scalar(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec4_mul_scalar_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec4_scalar(&format!("i{}", ctx.pr), "div_s");
                        format!("(call $_rpu_vec4_div_s_scalar_i{})", ctx.pr)
                    }
                }
            }
            // Float x Float
            (ASTValue::Float(_, _), ASTValue::Float(_, _)) => {
                rc = ASTValue::Float(None, 0.0);
                match op {
                    BinaryOperator::Add => format!("(f{}.add)", ctx.precision.describe()),
                    BinaryOperator::Subtract => format!("(f{}.sub)", ctx.precision.describe()),
                    BinaryOperator::Multiply => format!("(f{}.mul)", ctx.precision.describe()),
                    BinaryOperator::Divide => format!("(f{}.div)", ctx.precision.describe()),
                }
            }
            // Float x Float2
            (ASTValue::Float(_, _), ASTValue::Float2(_, _, _)) => {
                rc = ASTValue::Float2(None, empty_expr!(), empty_expr!());
                match op {
                    // Scalar and ivec2
                    BinaryOperator::Add => {
                        ctx.gen_scalar_vec2(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_scalar_add_vec2_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_scalar_vec2(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_scalar_sub_vec2_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_scalar_vec2(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_scalar_mul_vec2_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_scalar_vec2(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_scalar_div_vec2_f{})", ctx.pr)
                    }
                }
            }
            // Float2 x Float
            (ASTValue::Float2(_, _, _), ASTValue::Float(_, _)) => {
                rc = ASTValue::Float2(None, empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec2_scalar(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_vec2_add_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec2_scalar(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec2_sub_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec2_scalar(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec2_mul_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec2_scalar(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_vec2_div_scalar_f{})", ctx.pr)
                    }
                }
            }
            // Float x Float3
            (ASTValue::Float(_, _), ASTValue::Float3(_, _, _, _)) => {
                rc = ASTValue::Float3(None, empty_expr!(), empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_scalar_vec3(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_scalar_add_vec3_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_scalar_vec3(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_scalar_sub_vec3_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_scalar_vec3(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_scalar_mul_vec3_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_scalar_vec3(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_scalar_div_vec3_f{})", ctx.pr)
                    }
                }
            }
            // Float3 x Float
            (ASTValue::Float3(_, _, _, _), ASTValue::Float(_, _)) => {
                rc = ASTValue::Float3(None, empty_expr!(), empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec3_scalar(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_vec3_add_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec3_scalar(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec3_sub_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec3_scalar(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec3_mul_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec3_scalar(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_vec3_div_scalar_f{})", ctx.pr)
                    }
                }
            }
            // Float x Float4
            (ASTValue::Float(_, _), ASTValue::Float4(_, _, _, _, _)) => {
                rc = ASTValue::Float4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                );
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_scalar_vec4(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_scalar_add_vec4_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_scalar_vec4(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_scalar_sub_vec4_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_scalar_vec4(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_scalar_mul_vec4_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_scalar_vec4(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_scalar_div_vec4_f{})", ctx.pr)
                    }
                }
            }
            // Float4 x Float
            (ASTValue::Float4(_, _, _, _, _), ASTValue::Float(_, _)) => {
                rc = ASTValue::Float4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                );
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec4_scalar(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_vec4_add_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec4_scalar(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec4_sub_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec4_scalar(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec4_mul_scalar_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec4_scalar(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_vec4_div_scalar_f{})", ctx.pr)
                    }
                }
            }
            // Int2 x Int2
            (ASTValue::Int2(_, _, _), ASTValue::Int2(_, _, _)) => {
                rc = ASTValue::Int2(None, empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec2_vec2(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_vec2_add_vec2_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec2_vec2(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec2_sub_vec2_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec2_vec2(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec2_mul_vec2_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec2_vec2(&format!("i{}", ctx.pr), "div");
                        format!("(call $_rpu_vec2_div_vec2_i{})", ctx.pr)
                    }
                }
            }
            // Float2 x Float2
            (ASTValue::Float2(_, _, _), ASTValue::Float2(_, _, _)) => {
                rc = ASTValue::Float2(None, empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec2_vec2(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_vec2_add_vec2_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec2_vec2(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec2_sub_vec2_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec2_vec2(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec2_mul_vec2_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec2_vec2(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_vec2_div_vec2_f{})", ctx.pr)
                    }
                }
            }
            // Int3 x Int3
            (ASTValue::Int3(_, _, _, _), ASTValue::Int3(_, _, _, _)) => {
                rc = ASTValue::Int3(None, empty_expr!(), empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec3_vec3(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_vec3_add_vec3_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec3_vec3(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec3_sub_vec3_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec3_vec3(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec3_mul_vec3_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec3_vec3(&format!("i{}", ctx.pr), "div");
                        format!("(call $_rpu_vec3_div_vec3_i{})", ctx.pr)
                    }
                }
            }
            // Float3 x Float3
            (ASTValue::Float3(_, _, _, _), ASTValue::Float3(_, _, _, _)) => {
                rc = ASTValue::Float3(None, empty_expr!(), empty_expr!(), empty_expr!());
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec3_vec3(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_vec3_add_vec3_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec3_vec3(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec3_sub_vec3_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec3_vec3(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec3_mul_vec3_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec3_vec3(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_vec3_div_vec3_f{})", ctx.pr)
                    }
                }
            }
            // Int4 x Int4
            (ASTValue::Int4(_, _, _, _, _), ASTValue::Int4(_, _, _, _, _)) => {
                rc = ASTValue::Int4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                );
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec4_vec4(&format!("i{}", ctx.pr), "add");
                        format!("(call $_rpu_vec4_add_vec4_i{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec4_vec4(&format!("i{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec4_sub_vec4_i{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec4_vec4(&format!("i{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec4_mul_vec4_i{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec4_vec4(&format!("i{}", ctx.pr), "div");
                        format!("(call $_rpu_vec4_div_vec4_i{})", ctx.pr)
                    }
                }
            }
            // Float4 x Float4
            (ASTValue::Float4(_, _, _, _, _), ASTValue::Float4(_, _, _, _, _)) => {
                rc = ASTValue::Float4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                );
                match op {
                    BinaryOperator::Add => {
                        ctx.gen_vec4_vec4(&format!("f{}", ctx.pr), "add");
                        format!("(call $_rpu_vec4_add_vec4_f{})", ctx.pr)
                    }
                    BinaryOperator::Subtract => {
                        ctx.gen_vec4_vec4(&format!("f{}", ctx.pr), "sub");
                        format!("(call $_rpu_vec4_sub_vec4_f{})", ctx.pr)
                    }
                    BinaryOperator::Multiply => {
                        ctx.gen_vec4_vec4(&format!("f{}", ctx.pr), "mul");
                        format!("(call $_rpu_vec4_mul_vec4_f{})", ctx.pr)
                    }
                    BinaryOperator::Divide => {
                        ctx.gen_vec4_vec4(&format!("f{}", ctx.pr), "div");
                        format!("(call $_rpu_vec4_div_vec4_f{})", ctx.pr)
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Invalid types '{}' '{}' for operator '{}' {}",
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

            if name == "length" {
                let v = args[0].accept(self, ctx)?;
                let components = v.components();
                if !(1..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components {} {}",
                        components,
                        loc.describe()
                    ));
                }
                let func_name = ctx.gen_vec_length(v.components() as u32);
                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = ASTValue::Float(None, 0.0);
            } else if name == "normalize" {
                let v = args[0].accept(self, ctx)?;
                let components = v.components();
                if !(1..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components {} {}",
                        components,
                        loc.describe()
                    ));
                }
                let func_name = ctx.gen_vec_normalize(v.components() as u32);
                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = v;
            } else if name == "rand" {
                if !args.is_empty() {
                    return Err(format!(
                        "'rand' does not take any arguments {}",
                        loc.describe()
                    ));
                }
                let instr = "(call $_rpu_rand)";
                ctx.add_wat(instr);
                ctx.imports_hash.insert("$_rpu_rand".to_string());
                rc = ASTValue::Float(None, 0.0);
            } else if name == "sqrt"
                || name == "sin"
                || name == "cos"
                || name == "ceil"
                || name == "floor"
                || name == "fract"
                || name == "abs"
                || name == "tan"
                || name == "degrees"
                || name == "radians"
            {
                let v = args[0].accept(self, ctx)?;
                let components = v.components();
                if !(1..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components '{}' for {}",
                        components,
                        loc.describe()
                    ));
                }
                if !v.is_float_based() {
                    return Err(format!(
                        "'{}' expects a float based parameter {}",
                        name,
                        loc.describe()
                    ));
                }
                let func_name = ctx.gen_vec_operation(v.components() as u32, &name);
                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = v;
            } else if name == "max" || name == "min" || name == "pow" {
                let v = args[0].accept(self, ctx)?;

                if func_args.len() != args.len() {
                    return Err(format!(
                        "Function '{}' expects {} arguments, but {} were provided {}",
                        name,
                        func_args.len(),
                        args.len(),
                        loc.describe()
                    ));
                }

                let components = v.components();
                if !(1..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components '{}' for {}",
                        components,
                        loc.describe()
                    ));
                }
                if !v.is_float_based() {
                    return Err(format!(
                        "'{}' expects a float based parameter {}",
                        name,
                        loc.describe()
                    ));
                }

                let b = args[1].accept(self, ctx)?;
                if b.components() != 1 {
                    return Err(format!(
                        "Invalid second parameter for '{}' (scalars only) {}",
                        name,
                        loc.describe()
                    ));
                }

                let func_name = ctx.gen_vec_operation_scalar(v.components() as u32, &name);
                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = v;
            } else if name == "smoothstep" {
                let a1 = args[0].accept(self, ctx)?;
                let components = a1.components();
                if !(1..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components {} {}",
                        components,
                        loc.describe()
                    ));
                }
                let a2 = args[1].accept(self, ctx)?;

                if a1.to_type() != a2.to_type() || !a1.is_float_based() {
                    return Err(format!(
                        "'smoothstep' expects the first two arguments to be the same float type, but '{}' and '{}' were provided {}",
                        a1.to_type(),
                        a2.to_type(),
                        loc.describe()
                    ));
                }

                let a3 = args[2].accept(self, ctx)?;
                if a3.to_type() != "float" {
                    return Err(format!(
                        "'smoothstep' expects the third argument to be of type 'float', but '{}' was provided {}",
                        a3.to_type(),
                        loc.describe()
                    ));
                }

                let func_name = ctx.gen_vec_smoothstep(components as u32);

                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = a1;
            } else if name == "mix" {
                let a1 = args[0].accept(self, ctx)?;
                let components = a1.components();
                if !(1..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components {} {}",
                        components,
                        loc.describe()
                    ));
                }
                let a2 = args[1].accept(self, ctx)?;

                if a1.to_type() != a2.to_type() || !a1.is_float_based() {
                    return Err(format!(
                        "'mix' expects the first two arguments to be the same float type, but '{}' and '{}' were provided {}",
                        a1.to_type(),
                        a2.to_type(),
                        loc.describe()
                    ));
                }

                let a3 = args[2].accept(self, ctx)?;
                if a3.to_type() != "float" {
                    return Err(format!(
                        "'mix' expects the third argument to be of type 'float', but '{}' was provided {}",
                        a3.to_type(),
                        loc.describe()
                    ));
                }

                let func_name = ctx.gen_vec_mix(components as u32);

                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = a1;
            } else if name == "dot" {
                let a1 = args[0].accept(self, ctx)?;
                let components = a1.components();
                if !(2..=4).contains(&components) {
                    return Err(format!(
                        "Invalid number of components {} for 'dot' {}",
                        components,
                        loc.describe()
                    ));
                }
                let a2 = args[1].accept(self, ctx)?;

                if a1.to_type() != a2.to_type() || !a1.is_float_based() {
                    return Err(format!(
                        "'dot' expects the first two arguments to be the same float type, but '{}' and '{}' were provided {}",
                        a1.to_type(),
                        a2.to_type(),
                        loc.describe()
                    ));
                }

                let func_name = ctx.gen_vec_dot_product(components as u32);

                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = ASTValue::Float(None, 0.0);
            } else if name == "cross" {
                let a1 = args[0].accept(self, ctx)?;
                let components = a1.components();
                if components != 3 {
                    return Err(format!(
                        "Invalid number of components {} for 'dot' {}",
                        components,
                        loc.describe()
                    ));
                }
                let a2 = args[1].accept(self, ctx)?;

                if a1.to_type() != a2.to_type() || !a1.is_float_based() {
                    return Err(format!(
                        "'dot' expects the first two arguments to be the same float type, but '{}' and '{}' were provided {}",
                        a1.to_type(),
                        a2.to_type(),
                        loc.describe()
                    ));
                }

                let func_name = ctx.gen_vec_cross_product();

                let instr = format!("(call ${})", func_name);
                ctx.add_wat(&instr);
                rc = a1;
            } else {
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
            // Save the param into the environment
            if let Some(name) = param.name() {
                self.environment.define(name, param.clone());
            }

            match param {
                ASTValue::Int(name, _) => {
                    params += &format!(
                        "(param ${} i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
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
                ASTValue::Int3(name, _, _, _) => {
                    params += &format!(
                        "(param ${}_x i{}) (param ${}_y i{}) (param ${}_z i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                }
                ASTValue::Int4(name, _, _, _, _) => {
                    params += &format!(
                        "(param ${}_x i{}) (param ${}_y i{}) (param ${}_z i{}) (param ${}_w i{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                }
                ASTValue::Float(name, _) => {
                    params += &format!(
                        "(param ${} f{})",
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                }
                ASTValue::Float2(name, _, _) => {
                    params += &format!(
                        "(param ${}_x f{}) (param ${}_y f{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                }
                ASTValue::Float3(name, _, _, _) => {
                    params += &format!(
                        "(param ${}_x f{}) (param ${}_y f{}) (param ${}_z f{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe()
                    );
                }
                ASTValue::Float4(name, _, _, _, _) => {
                    params += &format!(
                        "(param ${}_x f{}) (param ${}_y f{}) (param ${}_z f{}) (param ${}_w f{})",
                        name.clone().unwrap(),
                        ctx.precision.describe(),
                        name.clone().unwrap(),
                        ctx.precision.describe(),
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

        ctx.wat.push_str("__LOCALS__");

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

        let instr = "(if".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        let instr = "(then".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        if let Some(d) = self.break_depth.last() {
            self.break_depth.push(d + 2);
        }

        let _ = then_stmt.accept(self, ctx)?;

        ctx.remove_indention();
        ctx.add_wat(")");

        if let Some(d) = self.break_depth.last() {
            self.break_depth.push(d - 2);
        }

        if let Some(es) = else_stmt {
            if let Some(d) = self.break_depth.last() {
                self.break_depth.push(d + 2);
            }
            let instr = "(else".to_string();
            ctx.add_wat(&instr);
            ctx.add_indention();
            let _ = es.accept(self, ctx)?;
            ctx.remove_indention();
            ctx.add_wat(")");
            if let Some(d) = self.break_depth.last() {
                self.break_depth.push(d - 2);
            }
        }

        ctx.remove_indention();
        ctx.add_wat(")");
        //ctx.add_line();

        Ok(ASTValue::None)
    }

    fn while_stmt(
        &mut self,
        cond: &Expr,
        body_stmt: &Stmt,
        _loc: &Location,
        ctx: &mut Context,
    ) -> Result<ASTValue, String> {
        ctx.add_line();

        let instr = "(block".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        let instr = "(loop".to_string();
        ctx.add_wat(&instr);
        ctx.add_indention();

        self.break_depth.push(0);

        let _rc = cond.accept(self, ctx)?;

        let instr = "(i32.eqz)".to_string();
        ctx.add_wat(&instr);

        let instr = "(br_if 1)".to_string();
        ctx.add_wat(&instr);

        let _rc = body_stmt.accept(self, ctx)?;

        let instr = "(br 0)".to_string();
        ctx.add_wat(&instr);

        self.break_depth.pop();

        ctx.remove_indention();
        ctx.add_wat(")");

        ctx.remove_indention();
        ctx.add_wat(")");

        Ok(ASTValue::None)
    }

    fn break_stmt(&mut self, _loc: &Location, ctx: &mut Context) -> Result<ASTValue, String> {
        if let Some(d) = self.break_depth.last() {
            let instr = format!("(br {})", d);
            ctx.add_wat(&instr);
        }

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
