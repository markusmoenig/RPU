use crate::prelude::*;
use std::path::PathBuf;

pub struct RPU {}

impl Default for RPU {
    fn default() -> Self {
        Self::new()
    }
}

impl RPU {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compile_from_path(&mut self, path: PathBuf) -> Result<(), String> {
        if let Ok(main) = std::fs::read_to_string(path) {
            //println!("{}", main);
            let mut scanner = Scanner::new(main);
            let mut parser = Parser::new();
            parser.parse(scanner);
        }

        /*
        let mut compiler = TheCompiler::default();

        if let Ok(main) = std::fs::read_to_string(path) {
            compiler.compile(main)
        } else {
            Err(InterpretError::CompileError(
                "Could not read file.".to_string(),
                0,
            ))
            }*/
        Ok(())
    }
}
