use crate::empty_expr;
use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum Precision {
    P32,
    P64,
}

impl Precision {
    pub fn size(&self) -> usize {
        match self {
            Precision::P32 => 4,
            Precision::P64 => 8,
        }
    }
    pub fn describe(&self) -> String {
        match self {
            Precision::P32 => "32".to_string(),
            Precision::P64 => "64".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    /// Precision of the compilation
    pub precision: Precision,

    /// Precision as a string
    pub pr: String,

    math_funcs_included: rustc_hash::FxHashSet<String>,
    math_funcs: String,

    /// The current indentation level
    indention: usize,

    /// The generated WAT code for function locals.
    pub wat_locals: String,

    /// The generated WAT code
    pub wat: String,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            precision: Precision::P64,
            pr: "64".to_string(),

            math_funcs_included: rustc_hash::FxHashSet::default(),
            math_funcs: String::new(),

            indention: 1,

            wat_locals: String::new(),
            wat: String::new(),
        }
    }

    /// Adds indention
    pub fn add_indention(&mut self) {
        self.indention += 1;
    }

    /// Removes indention
    pub fn remove_indention(&mut self) {
        if self.indention > 0 {
            self.indention -= 1;
        }
    }

    pub fn add_line(&mut self) {
        self.wat.push('\n');
    }

    /// Adds wat code
    pub fn add_wat(&mut self, wat: &str) {
        let spaces = " ".repeat(self.indention * 4);
        self.wat.push_str(&format!("{}{}\n", spaces, wat));
    }

    /// Deswizzle a swizzle component.
    pub fn deswizzle(&self, s: u8) -> String {
        match s {
            0 => "x".to_string(),
            1 => "y".to_string(),
            2 => "z".to_string(),
            3 => "w".to_string(),
            _ => panic!("Invalid swizzle component"),
        }
    }

    pub fn create_value_from_swizzle(&self, components: usize) -> ASTValue {
        match components {
            1 => ASTValue::Int(None, 0),
            2 => ASTValue::Int2(None, empty_expr!(), empty_expr!()),
            3 => ASTValue::Int3(None, empty_expr!(), empty_expr!(), empty_expr!()),
            4 => ASTValue::Int4(
                None,
                empty_expr!(),
                empty_expr!(),
                empty_expr!(),
                empty_expr!(),
            ),
            _ => panic!("Invalid swizzle components"),
        }
    }

    /// Generate the final wat code
    pub fn gen_wat(&mut self) -> String {
        let mut output = "(module\n    (memory 1)\n".to_string();
        output += &self.math_funcs;
        output += &self.wat;
        output += &")\n";

        output
    }

    pub fn gen_scalar_vec2(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_scalar_{}_vec2_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; scalar {op} vec2 ({data_type})
    (func ${func_name}
        (param $scalar {data_type})  ;; Scalar
        (param $vec2_x {data_type})  ;; x component of vec2
        (param $vec2_y {data_type})  ;; y component of vec2
        (result {data_type} {data_type})  ;; Return two {data_type} results, the new x and y components

        ;; Calculate the new x component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec2_y)  ;; Get the y component
        )
    )
"#,
            func_name = func_name,
            data_type = data_type,
            op = op
        );

        self.math_funcs_included.insert(func_name);
        self.math_funcs.push_str(&str);
    }

    pub fn gen_vec2_scalar(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_vec2_{}_scalar_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; vec2 {op} scalar ({data_type})
    (func ${func_name}
        (param $vec2_x {data_type})    ;; x component of vec2
        (param $vec2_y {data_type})    ;; y component of vec2
        (param $scalar {data_type})    ;; Scalar
        (result {data_type} {data_type})       ;; Return two {data_type} results, the new x and y components

        ;; Calculate the new x component and return it
        ({data_type}.{op}
            (local.get $vec2_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        ({data_type}.{op}
            (local.get $vec2_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )
    )
"#,
            func_name = func_name,
            data_type = data_type,
            op = op
        );

        self.math_funcs_included.insert(func_name);
        self.math_funcs.push_str(&str);
    }

    pub fn gen_scalar_vec3(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_scalar_{}_vec3_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; scalar {op} vec3 ({data_type})
    (func ${func_name}
        (param $scalar {data_type})  ;; Scalar
        (param $vec3_x {data_type})  ;; x component of vec3
        (param $vec3_y {data_type})  ;; y component of vec3
        (param $vec3_z {data_type})  ;; y component of vec3
        (result {data_type} {data_type} {data_type})  ;; Return three {data_type} results, the new x, y and z components

        ;; Calculate the new x component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_y)  ;; Get the y component
        )

        ;; Calculate the new z component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec3_z)  ;; Get the z component
        )
    )
"#,
            func_name = func_name,
            data_type = data_type,
            op = op
        );

        self.math_funcs_included.insert(func_name);
        self.math_funcs.push_str(&str);
    }

    pub fn gen_vec3_scalar(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_vec3_{}_scalar_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; vec3 {op} scalar ({data_type})
    (func ${func_name}
        (param $vec3_x {data_type})    ;; x component of vec3
        (param $vec3_y {data_type})    ;; y component of vec3
        (param $vec3_z {data_type})    ;; z component of vec3
        (param $scalar {data_type})    ;; Scalar
        (result {data_type} {data_type} {data_type})       ;; Return three {data_type} results, the new x, y and z components

        ;; Calculate the new x component and return it
        ({data_type}.{op}
            (local.get $vec3_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        ({data_type}.{op}
            (local.get $vec3_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        ({data_type}.{op}
            (local.get $vec3_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
        )
    )
"#,
            func_name = func_name,
            data_type = data_type,
            op = op
        );

        self.math_funcs_included.insert(func_name);
        self.math_funcs.push_str(&str);
    }

    pub fn gen_scalar_vec4(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_scalar_{}_vec4_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; scalar {op} vec4 ({data_type})
    (func ${func_name}
        (param $scalar {data_type})  ;; Scalar
        (param $vec4_x {data_type})  ;; x component of vec4
        (param $vec4_y {data_type})  ;; y component of vec4
        (param $vec4_z {data_type})  ;; z component of vec4
        (param $vec4_w {data_type})  ;; w component of vec4
        (result {data_type} {data_type} {data_type} {data_type})  ;; Return four {data_type} results, the new x, y, z and w components

        ;; Calculate the new x component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_x)  ;; Get the x component
        )

        ;; Calculate the new y component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_y)  ;; Get the y component
        )

        ;; Calculate the new z component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_z)  ;; Get the z component
        )

        ;; Calculate the new w component and return it
        ({data_type}.{op}
            (local.get $scalar)  ;; Get the scalar
            (local.get $vec4_w)  ;; Get the w component
        )
    )
"#,
            func_name = func_name,
            data_type = data_type,
            op = op
        );

        self.math_funcs_included.insert(func_name);
        self.math_funcs.push_str(&str);
    }

    pub fn gen_vec4_scalar(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_vec4_{}_scalar_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; vec4 {op} scalar ({data_type})
    (func ${func_name}
        (param $vec4_x {data_type})    ;; x component of vec4
        (param $vec4_y {data_type})    ;; y component of vec4
        (param $vec4_z {data_type})    ;; z component of vec4
        (param $vec4_w {data_type})    ;; w component of vec4
        (param $scalar {data_type})    ;; Scalar
        (result {data_type} {data_type} {data_type} {data_type})       ;; Return four {data_type} results, the new x, y, z and w components

        ;; Calculate the new x component and return it
        ({data_type}.{op}
            (local.get $vec4_x)  ;; Get the x component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new y component and return it
        ({data_type}.{op}
            (local.get $vec4_y)  ;; Get the y component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new z component and return it
        ({data_type}.{op}
            (local.get $vec4_z)  ;; Get the z component
            (local.get $scalar)  ;; Get the scalar
        )

        ;; Calculate the new w component and return it
        ({data_type}.{op}
            (local.get $vec4_w)  ;; Get the w component
            (local.get $scalar)  ;; Get the scalar
        )
    )
"#,
            func_name = func_name,
            data_type = data_type,
            op = op
        );

        self.math_funcs_included.insert(func_name);
        self.math_funcs.push_str(&str);
    }
}
