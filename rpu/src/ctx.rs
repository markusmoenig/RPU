//use crate::prelude::*;

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

    pub fn gen_wat(&mut self) -> String {
        let mut header = "(module\n    (memory 1)\n".to_string();

        header += &self.math_funcs;

        let mut wat = header.to_string();
        wat.push_str(&self.wat);

        wat.push_str(")\n");

        println!("{}", wat);

        wat
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
}
