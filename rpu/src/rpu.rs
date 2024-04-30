use crate::prelude::*;
//use rayon::prelude::*;
//use std::collections::VecDeque;
use std::path::PathBuf;
//use std::sync::{Arc, Mutex};
use wasmer::{imports, Function, Instance, Module, Store, Value};

/*
struct Tile {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    }*/

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

    /// Compile the RPU source code (given by its path) to WAT source code.
    pub fn compile_to_wat_from_path(
        &self,
        path: PathBuf,
        high_precision: bool,
    ) -> Result<String, String> {
        if let Ok(main) = std::fs::read_to_string(path.clone()) {
            let scanner = Scanner::new(main);
            let mut parser = Parser::new();
            parser.set_high_precision(high_precision);

            parser.parse(scanner)
        } else {
            Err("Could not read file.".to_string())
        }
    }

    /// Compile the RPU source code to WAT source code.
    pub fn compile_to_wat(&self, rpu_source: String) -> Result<String, String> {
        let scanner = Scanner::new(rpu_source);
        let mut parser = Parser::new();

        parser.parse(scanner)
    }

    /// Compile the RPU source code and run the function with the given arguments.
    pub fn compile_and_run(
        &self,
        source: &str,
        func_name: &str,
        args: Vec<Value>,
        high_precision: bool,
    ) -> Result<Vec<Value>, String> {
        let rc = self.compile_to_wat(source.to_string());
        match rc {
            Ok(wat) => self.compile_wat_and_run(&wat, func_name, args, high_precision),
            Err(err) => Err(err.to_string()),
        }
    }

    /// Compile the WAT source code and run the function with the given arguments.
    pub fn compile_wat_and_run(
        &self,
        wat: &str,
        func_name: &str,
        args: Vec<Value>,
        high_precision: bool,
    ) -> Result<Vec<Value>, String> {
        let mut store = Store::default();
        let module_rc = Module::new(&store, wat);
        match module_rc {
            Ok(module) => {
                let import_object = self.create_imports(&mut store, high_precision);
                if let Ok(instance) = Instance::new(&mut store, &module, &import_object) {
                    if let Ok(func) = instance.exports.get_function(func_name) {
                        let _start = self.get_time();
                        match func.call(&mut store, &args) {
                            Ok(values) => {
                                let _stop = self.get_time();
                                println!("Execution time: {:?} ms.", _stop - _start);
                                return Ok(values.to_vec());
                            }
                            Err(err) => return Err(err.to_string()),
                        }
                    }
                }
            }
            Err(err) => return Err(err.to_string()),
        }

        Err("Unknown error".to_string())
    }

    /// Compile the WAT source code and run the shader with the given arguments. The shader will be executed on the given buffer.
    pub fn compile_wat_and_run_as_shader(
        &self,
        wat: &str,
        func_name: &str,
        buffer: &mut ColorBuffer,
        high_precision: bool,
    ) -> Result<Vec<Value>, String> {
        let mut store = Store::default();
        let module_rc = Module::new(&store, wat);
        match module_rc {
            Ok(module) => {
                let import_object = self.create_imports(&mut store, high_precision);
                if let Ok(instance) = Instance::new(&mut store, &module, &import_object) {
                    if let Ok(func) = instance.exports.get_function(func_name) {
                        let _start = self.get_time();
                        for y in 0..buffer.height {
                            for x in 0..buffer.width {
                                let args = if high_precision {
                                    vec![
                                        Value::F64(x as f64),
                                        Value::F64(buffer.height as f64 - y as f64),
                                        Value::F64(buffer.width as f64),
                                        Value::F64(buffer.height as f64),
                                    ]
                                } else {
                                    vec![
                                        Value::F32(x as f32),
                                        Value::F32(buffer.height as f32 - y as f32),
                                        Value::F32(buffer.width as f32),
                                        Value::F32(buffer.height as f32),
                                    ]
                                };

                                match func.call(&mut store, &args) {
                                    Ok(values) => {
                                        if high_precision {
                                            let r = values[0].f64().unwrap();
                                            let g = values[1].f64().unwrap();
                                            let b = values[2].f64().unwrap();
                                            let a = values[3].f64().unwrap();
                                            buffer.set(x, y, [r, g, b, a]);
                                        } else {
                                            let r = values[0].f32().unwrap();
                                            let g = values[1].f32().unwrap();
                                            let b = values[2].f32().unwrap();
                                            let a = values[3].f32().unwrap();
                                            buffer.set(
                                                x,
                                                y,
                                                [r as f64, g as f64, b as f64, a as f64],
                                            );
                                        }
                                    }
                                    Err(err) => return Err(err.to_string()),
                                }
                            }
                        }
                        let _stop = self.get_time();
                        println!("Shader execution time: {:?} ms.", _stop - _start);
                    }
                }
            }
            Err(err) => return Err(err.to_string()),
        }

        Ok(vec![])
    }

    fn create_imports(&self, store: &mut Store, high_precision: bool) -> wasmer::Imports {
        if high_precision {
            imports! {
                "env" => {
                    "_rpu_sin" => Function::new_typed(store, |x: f64| -> f64 { x.sin() }),
                    "_rpu_cos" => Function::new_typed(store, |x: f64| -> f64 { x.cos() }),
                },
            }
        } else {
            imports! {
                "env" => {
                    "_rpu_sin" => Function::new_typed(store, |x: f32| -> f32 { x.sin() }),
                    "_rpu_cos" => Function::new_typed(store, |x: f32| -> f32 { x.cos() }),
                },
            }
        }
    }

    /*
    pub fn compile_wat_and_run_as_shader(
        &self,
        wat: &str,
        func_name: &str,
        buffer: &mut Arc<Mutex<ColorBuffer>>,
    ) -> Result<(), String> {
        let tile_size = 80;

        let width = buffer.lock().unwrap().width;
        let height = buffer.lock().unwrap().width;

        let tiles_x = (width + tile_size - 1) / tile_size;
        let tiles_y = (height + tile_size - 1) / tile_size;
        let tile_queue: Arc<Mutex<VecDeque<Tile>>> = Arc::new(Mutex::new(VecDeque::new()));

        for y in 0..tiles_y {
            for x in 0..tiles_x {
                let tile = Tile {
                    x: x * tile_size,
                    y: y * tile_size,
                    width: tile_size, //((x + 1) * tile_size).min(width) - x * tile_size,
                    height: tile_size, //((y + 1) * tile_size).min(height) - y * tile_size,
                };
                tile_queue.lock().unwrap().push_back(tile);
            }
        }

        let _start = self.get_time();

        (0../*num_cpus::get()*/5).into_par_iter().for_each(|_| {
            let mut store = Store::default();
            let module_rc = Module::new(&store, wat);
            let module = module_rc.map_err(|e| e.to_string()).ok().unwrap();
            let import_object = imports! {};
            let instance = Instance::new(&mut store, &module, &import_object)
                .map_err(|e| e.to_string())
                .ok()
                .unwrap();
            let func = instance
                .exports
                .get_function(func_name)
                .map_err(|e| e.to_string())
                .ok()
                .unwrap();
            let func = Arc::new(func);
            while let Some(tile) = tile_queue.lock().unwrap().pop_front() {
                let mut local_pixels = vec![0.0; tile.width * tile.height * 4];

                for y in 0..tile.height {
                    for x in 0..tile.width {
                        let global_x = tile.x + x;
                        let global_y = tile.y + y;
                        if global_x >= width || global_y >= height {
                            continue;
                        }
                        let args = vec![
                            Value::F64(global_x as f64 / width as f64),
                            Value::F64(1.0 - global_y as f64 / height as f64),
                            Value::F64(width as f64),
                            Value::F64(height as f64),
                        ];
                        if let Ok(values) = func.call(&mut store, &args) {
                            let r = values[0].f64().unwrap();
                            let g = values[1].f64().unwrap();
                            let b = values[2].f64().unwrap();
                            let a = values[3].f64().unwrap();
                            let index = (y * tile.width + x) * 4;
                            local_pixels[index] = r;
                            local_pixels[index + 1] = g;
                            local_pixels[index + 2] = b;
                            local_pixels[index + 3] = a;
                        }
                    }
                }
                let mut buffer_guard = buffer.lock().unwrap();
                buffer_guard.set_pixels(tile.x, tile.y, tile.width, tile.height, &local_pixels);
            }
        });

        let _stop = self.get_time();
        println!("shader time {:?}ms", _stop - _start);

        Ok(())
    }

    pub fn compile_wat_and_run_as_shader_(
        &self,
        wat: &str,
        func_name: &str,
        buffer: &mut ColorBuffer,
    ) -> Result<(), String> {
        let module_rc = Module::new(&Store::default(), wat);
        let module = module_rc.map_err(|e| e.to_string())?;
        let module = Arc::new(module);

        let tile_size = 80;

        let tiles_x = (buffer.width + tile_size - 1) / tile_size;
        let tiles_y = (buffer.height + tile_size - 1) / tile_size;

        let start_time = self.get_time();

        (0..tiles_x).into_par_iter().for_each(|tile_x| {
            let start_x = tile_x * tile_size;
            let end_x = (start_x + tile_size).min(buffer.width);

            (0..tiles_y).into_par_iter().for_each(|tile_y| {
                let start_y = tile_y * tile_size;
                let end_y = (start_y + tile_size).min(buffer.height);

                let mut local_buffer = ColorBuffer::new(tile_size, tile_size);

                let mut store = Store::default();
                let import_object = imports! {};
                let instance = Instance::new(&mut store, &module, &import_object)
                    .expect("Failed to create instance");
                let func = instance
                    .exports
                    .get_function(func_name)
                    .expect("Failed to get function");

                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let args = vec![
                            Value::F64(x as f64 / buffer.width as f64),
                            Value::F64(1.0 - y as f64 / buffer.height as f64),
                            Value::F64(buffer.width as f64),
                            Value::F64(buffer.height as f64),
                        ];
                        let result = func.call(&mut store, &args);
                        if let Ok(values) = result {
                            let r = values[0].f64().unwrap();
                            let g = values[1].f64().unwrap();
                            let b = values[2].f64().unwrap();
                            let a = values[3].f64().unwrap();
                            local_buffer.set(x, y, [r, g, b, a]);
                        }
                    }
                }
            });
        });

        let stop_time = self.get_time();
        println!("Shader time: {}ms", stop_time - start_time);
        Ok(())
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
        }*/

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
