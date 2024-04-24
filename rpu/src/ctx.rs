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
            precision: Precision::P64,
            verbose: true,
            wat: String::new(),
        }
    }

    pub fn gen_wat(&mut self) -> String {
        let header = "(module\n(memory 1)\n";

        let mut wat = header.to_string();
        wat.push_str(&self.wat);

        wat.push_str(")\n");

        //println!("--");
        //println!("{}", wat);

        wat
    }
}
