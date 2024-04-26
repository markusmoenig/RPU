use crate::prelude::*;
use std::path::PathBuf;
use wasmer::{imports, Instance, Module, Store, Value};

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

    pub fn compile_from_path(&mut self, mut path: PathBuf) -> Result<(), String> {
        if let Ok(main) = std::fs::read_to_string(path.clone()) {
            //println!("{}", main);
            let scanner = Scanner::new(main);
            let mut parser = Parser::new();

            let wat = parser.parse(scanner);

            match &wat {
                Ok(wat) => {
                    path.set_extension("wat");
                    _ = std::fs::write(path, wat);

                    let mut store = Store::default();
                    let module_rc = Module::new(&store, wat);
                    match module_rc {
                        Ok(module) => {
                            let import_object = imports! {};
                            if let Ok(instance) = Instance::new(&mut store, &module, &import_object)
                            {
                                if let Ok(main) = instance.exports.get_function("main") {
                                    let rc = main.call(&mut store, &[Value::I64(42)]);
                                    println!("rc {:?}", rc);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }

        Ok(())
    }
}
