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

    /// Global variables
    pub globals: FxHashMap<String, ASTValue>,

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

            globals: FxHashMap::default(),

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

        for (name, value) in self.globals.iter() {
            if let ASTValue::Float(_, value) = value {
                let mut str_value = format!("{:.}", value);
                if !str_value.contains('.') {
                    str_value.push_str(".0");
                }
                output += &format!(
                    "    (global ${} (mut f{}) (f{}.const {}))\n",
                    name, self.pr, self.pr, str_value
                );
            } else if let ASTValue::Int(_, value) = value {
                output += &format!(
                    "    (global ${} (mut i{}) (i{}.const {}))\n",
                    name, self.pr, self.pr, value
                );
            }
        }

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

    pub fn gen_vec_length(&mut self, components: u32) -> String {
        let func_name = format!("_rpu_vec{}_length_f{}", components, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_vector_length(components);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_vector_length(&self, dim: u32) -> String {
        let full_precision = format!("f{}", self.pr);

        let mut params = String::new();
        let mut body = String::new();

        // Generate parameters and calculation body
        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params.push_str(&format!(" (param ${} {})", coord, full_precision));
            if i == 0 {
                body.push_str(&format!(
                    "\n        local.get ${}\n        local.get ${}\n        {full_precision}.mul",
                    coord,
                    coord,
                    full_precision = full_precision
                ));
            } else {
                body.push_str(&format!("\n        local.get ${}\n        local.get ${}\n        {full_precision}.mul\n        {full_precision}.add", coord, coord, full_precision=full_precision));
            }
        }

        body.push_str(&format!(
            "\n        {full_precision}.sqrt",
            full_precision = full_precision
        ));

        // Assemble the complete function with indented formatting
        format!(
            "\n    ;; vec{} length\n    (func $_rpu_vec{}_length_{}{} (result {})        {})\n",
            dim, dim, full_precision, params, full_precision, body
        )
    }

    // Smoothstep function

    pub fn gen_vec_smoothstep(&mut self, components: u32) -> String {
        let func_name = format!("_rpu_smoothstep_vec{}_f{}", components, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_smoothstep(components);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_smoothstep(&self, dim: u32) -> String {
        let full_precision = format!("f{}", self.pr); // Prepends 'f' to the precision

        let mut params = String::new();
        let mut body = String::new();
        let mut locals = String::new();
        let mut params_edge0 = String::new();
        let mut params_edge1 = String::new();

        // Generate parameters for edge0, edge1, and x (scalar)
        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params_edge0.push_str(&format!(
                " (param $edge0_{coord} {precision})",
                coord = coord,
                precision = full_precision
            ));
            params_edge1.push_str(&format!(
                " (param $edge1_{coord} {precision})",
                coord = coord,
                precision = full_precision
            ));

            locals.push_str(&format!(
                " (local $t_{coord} {precision})",
                coord = coord,
                precision = full_precision
            )); // Declare local for t
        }

        params.push_str(&format!(
            " (param $x {precision})",
            precision = full_precision
        ));

        // Scalar factor x is added after vector components
        let params_factor = format!(" (param $x {precision})", precision = full_precision);

        // Combine all parameters in correct order
        let params = format!("{}{}{}", params_edge0, params_edge1, params_factor);

        // Generate smooth step computation for each component
        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };
            body.push_str(&format!(
                "
        ;; Calculate normalized t for the component {coord}
        local.get $x
        local.get $edge0_{coord}
        {full_precision}.sub
        local.get $edge1_{coord}
        local.get $edge0_{coord}
        {full_precision}.sub
        {full_precision}.div
        local.tee $t_{coord}
        {full_precision}.const 0
        {full_precision}.max
        {full_precision}.const 1
        {full_precision}.min
        local.set $t_{coord}

        ;; Calculate smoothstep polynomial 3t^2 - 2t^3
        local.get $t_{coord}
        local.get $t_{coord}
        {full_precision}.mul
        {full_precision}.const 3
        {full_precision}.mul
        local.get $t_{coord}
        local.get $t_{coord}
        {full_precision}.mul
        {full_precision}.const 2
        {full_precision}.mul
        {full_precision}.sub",
                coord = coord,
                full_precision = full_precision
            ));
        }

        format!(
            "\n    ;; vec{} smoothstep\n    (func $_rpu_smoothstep_vec{}_f{}{} (result {})\n{}        {})\n\n",dim,
            dim, self.pr, params, full_precision, locals, body
        )
    }

    pub fn gen_vec_mix(&mut self, components: u32) -> String {
        let func_name = format!("_rpu_mix_vec{}_f{}", components, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_mix(components);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_mix(&self, dim: u32) -> String {
        let full_precision = format!("f{}", self.pr); // Prepends 'f' to the precision

        let mut body = String::new();
        let mut params_edge0 = String::new();
        let mut params_edge1 = String::new();

        // Generate parameters for edge0, edge1, and x (scalar)
        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params_edge0.push_str(&format!(
                " (param $edge0_{coord} {precision})",
                coord = coord,
                precision = full_precision
            ));
            params_edge1.push_str(&format!(
                " (param $edge1_{coord} {precision})",
                coord = coord,
                precision = full_precision
            ));
        }

        // Scalar factor is added after vector components
        let params_factor = format!(" (param $factor {precision})", precision = full_precision);

        // Combine all parameters in correct order
        let params = format!("{}{}{}", params_edge0, params_edge1, params_factor);

        // Generate smooth step computation for each component
        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            body.push_str(&format!(
                "
        ;; Calculate linear interpolation for component {coord}
        local.get $edge0_{coord}
        local.get $edge1_{coord}
        local.get $edge0_{coord}
        {full_precision}.sub
        local.get $factor
        {full_precision}.mul
        {full_precision}.add",
                coord = coord,
                full_precision = full_precision
            ));
        }

        // Return the result directly from the stack
        let result_type = format!(" (result{})", " ".repeat(dim as usize) + &full_precision);

        format!(
            "\n    ;; vec{} mix\n    (func $_rpu_mix_vec{}_f{}{} {}\n        {})\n",
            dim, dim, self.pr, params, result_type, body
        )
    }
}
