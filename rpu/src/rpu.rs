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

    pub fn compile_to_wat_from_path(&self, path: PathBuf) -> Result<String, String> {
        if let Ok(main) = std::fs::read_to_string(path.clone()) {
            let scanner = Scanner::new(main);
            let mut parser = Parser::new();

            parser.parse(scanner)
        } else {
            Err("Could not read file.".to_string())
        }
    }

    pub fn compile_to_wat(&self, rpu_source: String) -> Result<String, String> {
        let scanner = Scanner::new(rpu_source);
        let mut parser = Parser::new();

        parser.parse(scanner)
    }

    pub fn compile_and_run(
        &self,
        source: &str,
        func_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, String> {
        let rc = self.compile_to_wat(source.to_string());
        match rc {
            Ok(wat) => self.compile_wat_and_run(&wat, func_name, args),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn compile_wat_and_run(
        &self,
        wat: &str,
        func_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, String> {
        let mut store = Store::default();
        let module_rc = Module::new(&store, wat);
        match module_rc {
            Ok(module) => {
                let import_object = imports! {};
                if let Ok(instance) = Instance::new(&mut store, &module, &import_object) {
                    if let Ok(func) = instance.exports.get_function(func_name) {
                        match func.call(&mut store, &args) {
                            Ok(values) => return Ok(values.to_vec()),
                            Err(err) => return Err(err.to_string()),
                        }
                    }
                }
            }
            Err(err) => return Err(err.to_string()),
        }

        Err("Unknown error".to_string())
    }

    pub fn compile_wat_to_module(&mut self, wat: String, store: &Store) -> Result<Module, String> {
        let rc = Module::new(store, wat);
        match rc {
            Ok(module) => Ok(module),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn compile_wat_from_path(&mut self, mut path: PathBuf) -> Result<(), String> {
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

    /// Get the current time
    pub fn get_time(&self) -> u128 {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window().unwrap().performance().unwrap().now() as u128
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let stop = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            stop.as_millis()
        }
    }
}
