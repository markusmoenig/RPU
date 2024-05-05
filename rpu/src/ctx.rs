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

    /// Adjusts the precision.
    pub fn set_high_precision(&mut self, high: bool) {
        if high {
            self.precision = Precision::P64;
            self.pr = "64".to_string();
        } else {
            self.precision = Precision::P32;
            self.pr = "32".to_string();
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

    /// Create the return value from a swizzle.
    pub fn create_value_from_swizzle(&self, base: &ASTValue, components: usize) -> ASTValue {
        if base.is_float_based() {
            match components {
                1 => ASTValue::Float(None, 0.0),
                2 => ASTValue::Float2(None, empty_expr!(), empty_expr!()),
                3 => ASTValue::Float3(None, empty_expr!(), empty_expr!(), empty_expr!()),
                4 => ASTValue::Float4(
                    None,
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                    empty_expr!(),
                ),
                _ => panic!("Invalid swizzle components"),
            }
        } else {
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
    }

    /// Generate the final wat code
    pub fn gen_wat(&mut self) -> String {
        let mut output = "(module\n".to_string();

        output += &format!(
            "    (import \"env\" \"_rpu_sin\" (func $_rpu_sin (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );
        output += &format!(
            "    (import \"env\" \"_rpu_cos\" (func $_rpu_cos (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );
        output += &format!(
            "    (import \"env\" \"_rpu_tan\" (func $_rpu_tan (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );
        output += &format!(
            "    (import \"env\" \"_rpu_degrees\" (func $_rpu_degrees (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );
        output += &format!(
            "    (import \"env\" \"_rpu_radians\" (func $_rpu_radians (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );

        output += &format!(
            "    (import \"env\" \"_rpu_min\" (func $_rpu_min (param f{pr}) (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );
        output += &format!(
            "    (import \"env\" \"_rpu_max\" (func $_rpu_max (param f{pr}) (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );
        output += &format!(
            "    (import \"env\" \"_rpu_pow\" (func $_rpu_pow (param f{pr}) (param f{pr}) (result f{pr})))\n",
            pr = self.pr
        );

        output += "\n    (memory 1)\n";

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

    /// vec2 op vec2
    pub fn gen_vec2_vec2(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_vec2_{}_vec2_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; vec2 {op} vec2 ({data_type})
    (func ${func_name}
        (param $vec2l_x {data_type})
        (param $vec2l_y {data_type})
        (param $vec2r_x {data_type})
        (param $vec2r_y {data_type})
        (result {data_type} {data_type})

        ({data_type}.{op}
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        ({data_type}.{op}
            (local.get $vec2l_y)
            (local.get $vec2r_y)
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

    /// vec3 op vec3
    pub fn gen_vec3_vec3(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_vec3_{}_vec3_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; vec3 {op} vec3 ({data_type})
    (func ${func_name}
        (param $vec2l_x {data_type})
        (param $vec2l_y {data_type})
        (param $vec2l_z {data_type})
        (param $vec2r_x {data_type})
        (param $vec2r_y {data_type})
        (param $vec2r_z {data_type})
        (result {data_type} {data_type} {data_type})

        ({data_type}.{op}
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        ({data_type}.{op}
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        ({data_type}.{op}
            (local.get $vec2l_z)
            (local.get $vec2r_z)
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

    /// vec3 op vec3
    pub fn gen_vec4_vec4(&mut self, data_type: &str, op: &str) {
        let func_name = format!("_rpu_vec4_{}_vec4_{}", op, data_type);

        if self.math_funcs_included.contains(&func_name) {
            return;
        }

        let str = format!(
            r#"
    ;; vec4 {op} vec4 ({data_type})
    (func ${func_name}
        (param $vec2l_x {data_type})
        (param $vec2l_y {data_type})
        (param $vec2l_z {data_type})
        (param $vec2l_w {data_type})
        (param $vec2r_x {data_type})
        (param $vec2r_y {data_type})
        (param $vec2r_z {data_type})
        (param $vec2r_w {data_type})
        (result {data_type} {data_type} {data_type} {data_type})

        ({data_type}.{op}
            (local.get $vec2l_x)
            (local.get $vec2r_x)
        )

        ({data_type}.{op}
            (local.get $vec2l_y)
            (local.get $vec2r_y)
        )

        ({data_type}.{op}
            (local.get $vec2l_z)
            (local.get $vec2r_z)
        )

        ({data_type}.{op}
            (local.get $vec2l_w)
            (local.get $vec2r_w)
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

        format!(
            "\n    ;; vec{} length\n    (func $_rpu_vec{}_length_{}{} (result {})        {})\n",
            dim, dim, full_precision, params, full_precision, body
        )
    }

    // Smoothstep

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
        let full_precision = format!("f{}", self.pr);

        let mut params = String::new();
        let mut body = String::new();
        let mut locals = String::new();
        let mut params_edge0 = String::new();
        let mut params_edge1 = String::new();

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
            ));
        }

        params.push_str(&format!(
            " (param $x {precision})",
            precision = full_precision
        ));

        let params_factor = format!(" (param $x {precision})", precision = full_precision);
        let params = format!("{}{}{}", params_edge0, params_edge1, params_factor);

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

        let mut result_type = " (result".to_string();
        for _ in 0..dim {
            result_type.push_str(&format!(" {} ", full_precision));
        }
        result_type.push(')');

        format!(
            "\n    ;; vec{} smoothstep\n    (func $_rpu_smoothstep_vec{}_f{}{} {}\n       {}        {})\n\n",dim,
            dim, self.pr, params, result_type, locals, body
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

    // mix

    fn generate_wat_mix(&self, dim: u32) -> String {
        let full_precision = format!("f{}", self.pr); // Prepends 'f' to the precision

        let mut body = String::new();
        let mut params_edge0 = String::new();
        let mut params_edge1 = String::new();

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

        let params_factor = format!(" (param $factor {precision})", precision = full_precision);

        let params = format!("{}{}{}", params_edge0, params_edge1, params_factor);

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

        let mut result_type = " (result".to_string();
        for _ in 0..dim {
            result_type.push_str(&format!(" {} ", full_precision));
        }
        result_type.push(')');

        format!(
            "\n    ;; vec{} mix\n    (func $_rpu_mix_vec{}_f{}{} {}\n        {})\n",
            dim, dim, self.pr, params, result_type, body
        )
    }

    // normalize

    pub fn gen_vec_normalize(&mut self, components: u32) -> String {
        let func_name = format!("_rpu_normalize_vec{}_f{}", components, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_normalize(components);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_normalize(&self, dim: u32) -> String {
        let full_precision = format!("f{}", self.pr);

        let mut params = String::new();
        let mut body = String::new();
        let mut length_calculation = String::new();

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params.push_str(&format!(" (param ${} {})", coord, full_precision));
            length_calculation.push_str(&format!(
                "\n        local.get ${}\n        local.get ${}\n        {full_precision}.mul",
                coord,
                coord,
                full_precision = full_precision
            ));
            if i > 0 {
                length_calculation.push_str(&format!(
                    "\n        {full_precision}.add",
                    full_precision = full_precision
                ));
            }
        }

        length_calculation.push_str(&format!(
            "\n        {full_precision}.sqrt\n        (local.set $magn)",
            full_precision = full_precision
        ));

        body.push_str(&length_calculation);

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            body.push_str(&format!(
                "\n        local.get ${}\n        (local.get $magn)\n        {full_precision}.div",
                coord,
                full_precision = full_precision
            ));
        }

        let mut result_type = " (result".to_string();
        for _ in 0..dim {
            result_type.push_str(&format!(" {} ", full_precision));
        }
        result_type.push(')');

        format!(
            "\n    ;; vec{} normalize\n    (func $_rpu_normalize_vec{}_{}{} {}\n        (local $magn {})\n         {})\n",
            dim, dim, full_precision, params, result_type, full_precision, body
        )
    }

    // Dot product

    pub fn gen_vec_dot_product(&mut self, components: u32) -> String {
        let func_name = format!("_rpu_dot_product_vec{}_f{}", components, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_dot_product(components);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_dot_product(&self, dim: u32) -> String {
        let full_precision = format!("f{}", self.pr);

        let mut params_a = String::new();
        let mut params_b = String::new();
        let mut body = String::new();

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params_a.push_str(&format!(
                " (param $a_{coord} {precision}) ",
                coord = coord,
                precision = full_precision
            ));
            params_b.push_str(&format!(
                " (param $b_{coord} {precision}) ",
                coord = coord,
                precision = full_precision
            ));
        }

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            body.push_str(&format!(
                "        local.get $a_{coord}\n        local.get $b_{coord}\n        {full_precision}.mul\n",
                coord=coord, full_precision=full_precision
            ));

            if i == 0 {
                body.push_str("        local.set $dot_product\n");
            } else if i == dim - 1 {
                body.push_str(&format!(
                    "        local.get $dot_product\n        {full_precision}.add",
                    full_precision = full_precision
                ));
            } else {
                body.push_str(&format!("        local.get $dot_product\n        {full_precision}.add\n        local.set $dot_product\n", full_precision=full_precision));
            }
        }

        let params = format!("{}{}", params_a, params_b);
        format!(
            "\n    ;; vec{dim} dot product\n    (func $_rpu_dot_product_vec{dim}_{precision} {params} (result {precision}) (local $dot_product {precision})\n{body})\n",
            dim=dim, precision=full_precision, params=params, body=body
        )
    }

    // Cross product

    pub fn gen_vec_cross_product(&mut self) -> String {
        let func_name = format!("_rpu_cross_product_f{}", self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_cross_product();

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_cross_product(&self) -> String {
        let full_precision = format!("f{}", self.pr);

        let params = format!(
            "(param $a_x {precision}) (param $a_y {precision}) (param $a_z {precision}) \
             (param $b_x {precision}) (param $b_y {precision}) (param $b_z {precision})",
            precision = full_precision
        );

        let body = format!(
            "    local.get $a_y\n        local.get $b_z\n        {full_precision}.mul\n        \
        local.get $a_z\n        local.get $b_y\n        {full_precision}.mul\n        \
             {full_precision}.sub\n        local.set $c_x\n        \
        local.get $a_z\n        local.get $b_x\n        {full_precision}.mul\n        \
        local.get $a_x\n        local.get $b_z\n        {full_precision}.mul\n        \
             {full_precision}.sub\n        local.set $c_y\n        \
        local.get $a_x\n        local.get $b_y\n        {full_precision}.mul\n        \
        local.get $a_y\n        local.get $b_x\n        {full_precision}.mul\n        \
             {full_precision}.sub\n        local.set $c_z",
            full_precision = full_precision
        );

        let result_type = format!(
            "(result {precision} {precision} {precision})",
            precision = full_precision
        );

        format!(
            "\n    ;; cross product\n    (func $_rpu_cross_product_f{precision} {params} {result_type}\n        \
             (local $c_x f{precision}) (local $c_y f{precision}) (local $c_z f{precision})\n    \
             {body}\n        local.get $c_x\n        local.get $c_y\n        local.get $c_z)\n",
            precision = self.pr,
            params = params,
            result_type = result_type,
            body = body
        )
    }

    // Vec operations (sin, cos, sqrt, etc.)

    pub fn gen_vec_operation(&mut self, dim: u32, op: &str) -> String {
        let func_name = format!("_rpu_vec{}_{}_f{}", dim, op, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_vec_op(dim, op);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    // Operation on a vector (sin, cos, tan, )

    pub fn gen_vec_operation_scalar(&mut self, dim: u32, op: &str) -> String {
        let func_name = format!("_rpu_vec{}_{}_f{}", dim, op, self.pr);

        if self.math_funcs_included.contains(&func_name) {
            return func_name.clone();
        }

        let func = self.generate_wat_vec_op_scalar(dim, op);

        self.math_funcs_included.insert(func_name.clone());
        self.math_funcs.push_str(&func);

        func_name
    }

    fn generate_wat_vec_op(&self, dim: u32, op: &str) -> String {
        let full_precision = format!("f{}", self.pr);

        let mut params = String::new();
        let mut body = String::new();
        let mut result_type = String::new();

        let rpu_call = match op {
            "sin" => Some("$_rpu_sin"),
            "cos" => Some("$_rpu_cos"),
            "tan" => Some("$_rpu_tan"),
            "degrees" => Some("$_rpu_degrees"),
            "radians" => Some("$_rpu_radians"),
            _ => None,
        };

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params.push_str(&format!(
                " (param ${coord} {precision}) ",
                coord = coord,
                precision = full_precision
            ));
            result_type.push_str(&format!(" {precision}", precision = full_precision));
        }

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            if let Some(rpu_call) = rpu_call {
                body.push_str(&format!(
                    "        local.get ${coord}\n        (call {rpu_call})\n",
                    coord = coord,
                    rpu_call = rpu_call
                ));
            } else {
                body.push_str(&format!(
                    "        local.get ${coord}\n        {full_precision}.{op}\n",
                    coord = coord,
                    full_precision = full_precision,
                    op = op
                ));
            }
        }

        body.pop();

        format!(
            "\n    ;; vec{dim} {op}\n    (func $_rpu_vec{dim}_{op}_{precision} {params} (result{result_type})\n{body})\n",
            dim = dim,
            op = op,
            precision = full_precision,
            result_type = result_type,
            params = params,
            body = body
        )
    }

    // Operation on a vector and as scalar (max, min, powf etc.)

    pub fn generate_wat_vec_op_scalar(&self, dim: u32, op: &str) -> String {
        let full_precision = format!("f{}", self.pr);

        let mut params = String::new();
        let mut body = String::new();
        let mut result_type = String::new();

        let rpu_call = match op {
            "max" => Some("$_rpu_max"),
            "min" => Some("$_rpu_min"),
            "pow" => Some("$_rpu_pow"),
            _ => None,
        };

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            params.push_str(&format!(
                " (param ${coord} {precision}) ",
                coord = coord,
                precision = full_precision
            ));
            result_type.push_str(&format!(" {precision}", precision = full_precision));
        }

        params.push_str(&format!(
            " (param $scalar {precision}) ",
            precision = full_precision
        ));

        for i in 0..dim {
            let coord = match i {
                0 => "x",
                1 => "y",
                2 => "z",
                3 => "w",
                _ => unreachable!(),
            };

            if let Some(rpu_call) = rpu_call {
                body.push_str(&format!(
                    "        local.get ${coord}\n        local.get $scalar\n        (call {rpu_call})\n",
                    coord = coord,
                    rpu_call = rpu_call
                ));
            } else {
                body.push_str(&format!(
                    "        local.get ${coord}\n        local.get $scalar\n        {full_precision}.{op}\n",
                    coord = coord,
                    full_precision = full_precision,
                    op = op
                ));
            }
        }

        body.pop();

        format!(
            "\n    ;; vec{dim} {op}\n    (func $_rpu_vec{dim}_{op}_{precision} {params} (result{result_type})\n{body})\n",
            dim = dim,
            op = op,
            precision = full_precision,
            result_type = result_type,
            params = params,
            body = body
        )
    }
}
