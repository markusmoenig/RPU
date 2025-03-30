use crate::prelude::*;
use rand::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use wasmer::{imports, Function, Instance, Module, Store, Value};

#[derive(Debug, Clone, Copy)]
struct Tile {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

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
            let mut pre = Preprocessor::default();
            let main = pre.process_module(&main);
            let scanner = Scanner::new(main);
            let mut parser = Parser::new();
            parser.set_high_precision(high_precision);

            parser.parse(scanner)
        } else {
            Err("Could not read file.".to_string())
        }
    }

    /// Compile the RPU source code to WAT source code.
    pub fn compile_to_wat(
        &self,
        rpu_source: String,
        high_precision: bool,
    ) -> Result<String, String> {
        let mut pre = Preprocessor::default();
        let rpu_source = pre.process_module(&rpu_source);
        let scanner = Scanner::new(rpu_source);
        let mut parser = Parser::new();
        parser.set_high_precision(high_precision);

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
        let rc = self.compile_to_wat(source.to_string(), high_precision);
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
                let import_object = RPU::create_imports(&mut store, high_precision);
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
                let import_object = RPU::create_imports(&mut store, high_precision);
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

    /// Compile the WAT source code and run the shader with the given arguments. The shader will be executed on the given buffer.
    pub fn compile_wat_and_run_as_tiled_shader(
        &self,
        wat: &str,
        func_name: &str,
        buffer: &mut Arc<Mutex<ColorBuffer>>,
        tile_size: (usize, usize),
        iterations: usize,
        high_precision: bool,
    ) -> Result<Vec<Value>, String> {
        let width = buffer.lock().unwrap().width;
        let height = buffer.lock().unwrap().height;

        let tiles = self.create_tiles(width, height, tile_size.0, tile_size.1);

        let tiles_mutex = Arc::new(Mutex::new(tiles));

        let num_cpus = num_cpus::get();
        let _start = self.get_time();

        // Create threads
        let mut handles = vec![];
        for _ in 0..num_cpus {
            let tiles_mutex = Arc::clone(&tiles_mutex);
            let buffer_mutex = Arc::clone(buffer);
            let fname = func_name.to_string().clone();
            let wat = wat.to_string().clone();

            let handle = thread::spawn(move || {
                let mut store = Store::default();
                let module_rc = Module::new(&store, wat);
                match module_rc {
                    Ok(module) => {
                        let import_object = RPU::create_imports(&mut store, high_precision);
                        if let Ok(instance) = Instance::new(&mut store, &module, &import_object) {
                            if let Ok(func) = instance.exports.get_function(&fname) {
                                let mut tile_buffer = ColorBuffer::new(tile_size.0, tile_size.1);
                                loop {
                                    // Lock mutex to access tiles
                                    let mut tiles = tiles_mutex.lock().unwrap();

                                    // Check if there are remaining tiles
                                    if let Some(tile) = tiles.pop() {
                                        // Release mutex before processing tile
                                        drop(tiles);
                                        // Process tile
                                        for h in 0..tile.height {
                                            for w in 0..tile.width {
                                                let x = tile.x + w;
                                                let y = tile.y + h;

                                                if x >= width || y >= height {
                                                    continue;
                                                }

                                                let args = if high_precision {
                                                    vec![
                                                        Value::F64(x as f64),
                                                        Value::F64(height as f64 - y as f64),
                                                        Value::F64(width as f64),
                                                        Value::F64(height as f64),
                                                    ]
                                                } else {
                                                    vec![
                                                        Value::F32(x as f32),
                                                        Value::F32(height as f32 - y as f32),
                                                        Value::F32(width as f32),
                                                        Value::F32(height as f32),
                                                    ]
                                                };

                                                let mut fc = [0.0, 0.0, 0.0, 0.0];
                                                for i in 0..iterations {
                                                    if let Ok(gl) =
                                                        instance.exports.get_global("mem_ptr")
                                                    {
                                                        _ = gl.set(&mut store, Value::I32(32));
                                                    }
                                                    match func.call(&mut store, &args) {
                                                        Ok(values) => {
                                                            let rgba = if high_precision {
                                                                [
                                                                    values[0].f64().unwrap(),
                                                                    values[1].f64().unwrap(),
                                                                    values[2].f64().unwrap(),
                                                                    values[3].f64().unwrap(),
                                                                ]
                                                            } else {
                                                                [
                                                                    values[0].f32().unwrap() as f64,
                                                                    values[1].f32().unwrap() as f64,
                                                                    values[2].f32().unwrap() as f64,
                                                                    values[3].f32().unwrap() as f64,
                                                                ]
                                                            };
                                                            let f = 1.0 / (i as f64 + 1.0);
                                                            fc[0] = fc[0] * (1.0 - f) + rgba[0] * f;
                                                            fc[1] = fc[1] * (1.0 - f) + rgba[1] * f;
                                                            fc[2] = fc[2] * (1.0 - f) + rgba[2] * f;
                                                            fc[3] = fc[3] * (1.0 - f) + rgba[3] * f;
                                                        }
                                                        Err(err) => println!("{}", err),
                                                    }

                                                    // Set the final color into the local buffer
                                                    tile_buffer.set(w, h, fc);
                                                }
                                            }
                                        }
                                        // Save the tile buffer to the main buffer
                                        buffer_mutex.lock().unwrap().copy_from(
                                            tile.x,
                                            tile.y,
                                            &tile_buffer,
                                        );

                                        // Save thebuffer optionally to disk after each completed block.
                                        if let Ok(buffer) = buffer_mutex.lock() {
                                            if let Some(path) = &buffer.file_path {
                                                buffer.save(path.clone());
                                            }
                                        }
                                    } else {
                                        // No remaining tiles, exit loop
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        let _stop = self.get_time();
        println!("Shader execution time: {:?} ms.", _stop - _start);

        Ok(vec![])
    }

    /// Create the tiles as a spiral pattern starting from the center.
    fn create_tiles(
        &self,
        image_width: usize,
        image_height: usize,
        tile_width: usize,
        tile_height: usize,
    ) -> Vec<Tile> {
        // TODO: Generate the tiles in a nice spiral pattern

        let mut tiles = Vec::new();
        let mut x = 0;
        let mut y = 0;
        while x < image_width && y < image_height {
            let tile = Tile {
                x,
                y,
                width: tile_width,
                height: tile_height,
            };
            tiles.push(tile);
            x += tile_width;
            if x >= image_width {
                x = 0;
                y += tile_height;
            }
        }

        tiles
    }

    fn create_imports(store: &mut Store, high_precision: bool) -> wasmer::Imports {
        if high_precision {
            imports! {
                "env" => {
                    "_rpu_sin" => Function::new_typed(store, |x: f64| -> f64 { x.sin() }),
                    "_rpu_cos" => Function::new_typed(store, |x: f64| -> f64 { x.cos() }),
                    "_rpu_tan" => Function::new_typed(store, |x: f64| -> f64 { x.tan() }),
                    "_rpu_atan" => Function::new_typed(store, |x: f64| -> f64 { x.atan() }),
                    "_rpu_exp" => Function::new_typed(store, |x: f64| -> f64 { x.exp() }),
                    "_rpu_degrees" => Function::new_typed(store, |x: f64| -> f64 { x.to_degrees() }),
                    "_rpu_radians" => Function::new_typed(store, |x: f64| -> f64 { x.to_radians() }),
                    "_rpu_min" => Function::new_typed(store, |x: f64, y: f64| -> f64 { x.min(y) }),
                    "_rpu_max" => Function::new_typed(store, |x: f64, y: f64| -> f64 { x.max(y) }),
                    "_rpu_pow" => Function::new_typed(store, |x: f64, y: f64| -> f64 { x.powf(y) }),
                    "_rpu_mod" => Function::new_typed(store, |x: f64, y: f64| -> f64 { x - y * (x / y).floor() }),
                    "_rpu_step" => Function::new_typed(store, |edge: f64, x: f64| -> f64 { if x < edge {
                        0.0
                    } else {
                        1.0
                    }}),
                    "_rpu_rand" => Function::new_typed(store, || -> f64 {
                        let mut rng = rand::thread_rng();
                        rng.gen()
                    }),
                    "_rpu_sign" => Function::new_typed(store, |x: f64| -> f64 { x.signum() }),
                    "_rpu_clamp" => Function::new_typed(store, |x: f64, y: f64, z: f64| -> f64 { x.clamp(y, z) }),
                },
            }
        } else {
            imports! {
                "env" => {
                    "_rpu_sin" => Function::new_typed(store, |x: f32| -> f32 { x.sin() }),
                    "_rpu_cos" => Function::new_typed(store, |x: f32| -> f32 { x.cos() }),
                    "_rpu_tan" => Function::new_typed(store, |x: f32| -> f32 { x.tan() }),
                    "_rpu_atan" => Function::new_typed(store, |x: f32| -> f32 { x.atan() }),
                    "_rpu_exp" => Function::new_typed(store, |x: f32| -> f32 { x.exp() }),
                    "_rpu_degrees" => Function::new_typed(store, |x: f32| -> f32 { x.to_degrees() }),
                    "_rpu_radians" => Function::new_typed(store, |x: f32| -> f32 { x.to_radians() }),
                    "_rpu_min" => Function::new_typed(store, |x: f32, y: f32| -> f32 { x.min(y) }),
                    "_rpu_max" => Function::new_typed(store, |x: f32, y: f32| -> f32 { x.max(y) }),
                    "_rpu_pow" => Function::new_typed(store, |x: f32, y: f32| -> f32 { x.powf(y) }),
                    "_rpu_mod" => Function::new_typed(store, |x: f32, y: f32| -> f32 { x - y * (x / y).floor() }),
                    "_rpu_step" => Function::new_typed(store, |edge: f32, x: f32| -> f32 { if x < edge {
                        0.0
                    } else {
                        1.0
                    }}),
                    "_rpu_rand" => Function::new_typed(store, || -> f32 {
                        let mut rng = rand::thread_rng();
                        rng.gen()
                    }),
                    "_rpu_sign" => Function::new_typed(store, |x: f32| -> f32 { x.signum() }),
                    "_rpu_clamp" => Function::new_typed(store, |x: f32, y: f32, z: f32| -> f32 { x.clamp(y, z) }),
                },
            }
        }
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
