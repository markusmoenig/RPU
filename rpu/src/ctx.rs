//use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Context {
    /// Verbose / debug mode for the interpreter
    pub verbose: bool,
    ///
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
            verbose: true,
            wat: String::new(),
        }
    }

    pub fn gen_wat(&mut self) -> String {
        let header = r#"
            (module
            (type $t0 (func (param i64) (result i64)))
            (func $add_one (export "main") (type $t0) (param $p0 i64) (result i64)
            "#;
        //     get_local $p0
        //     i64.const 1
        //     i64.add))
        // "#;

        let mut wat = header.to_string();
        wat.push_str(&self.wat);

        //wat.push_str("\ni64.const 1\n");
        wat.push_str("))\n");

        wat
    }
}
