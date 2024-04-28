pub use rpu::prelude::*;
//use std::sync::{Arc, Mutex};

fn main() {
    let mut path = std::path::PathBuf::new();
    let as_shader = true;
    path.push("examples/main.rpu");

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        path = std::path::PathBuf::from(&args[1]);
    }

    let rpu = RPU::new();
    let rc = rpu.compile_to_wat_from_path(path.clone());

    match rc {
        Ok(wat) => {
            // Write the WAT to file
            path.set_extension("wat");
            _ = std::fs::write(path.clone(), wat.clone());

            if !as_shader {
                let rc = rpu.compile_wat_and_run(&wat, "main", vec![WasmValue::F64(10.0)]);
                match rc {
                    Ok(values) => {
                        println!("{:?}", values);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            } else {
                // let mut buffer = Arc::new(Mutex::new(ColorBuffer::new(800, 600)));
                let mut buffer = ColorBuffer::new(800, 600);
                let rc = rpu.compile_wat_and_run_as_shader_sync(&wat, "shader", &mut buffer);
                match rc {
                    Ok(_) => {
                        path.set_extension("png");
                        buffer.save(path);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
