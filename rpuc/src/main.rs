use clap::{Arg, ArgAction, Command};
pub use rpu::prelude::*;
use std::sync::{Arc, Mutex};

#[allow(clippy::assigning_clones)]
fn main() {
    let matches = Command::new("RPU Compiler")
        .version("0.1.0")
        .author("Markus Moenig <markus@moenig.io>")
        .about("Compiles and executes RPU source files.")
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .value_name("FILE")
                .help("Sets the source file to compile and execute")
                .required(true)
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("function")
                .short('f')
                .long("function")
                .action(ArgAction::Set)
                .value_name("STRING")
                .help("The function name to execute. Defaults to 'main'"),
        )
        .arg(
            Arg::new("precision")
                .short('p')
                .long("precision")
                .help("The numerical precision. Defaults to '64'")
                .action(ArgAction::Set)
                .value_name("STRING")
                .value_parser(clap::builder::ValueParser::string()),
        )
        .arg(
            Arg::new("arg")
                .short('a')
                .long("argument")
                .help("The argument for the function. Could be an integer or a float")
                .action(ArgAction::Set)
                .value_name("NUMBER")
                .value_parser(clap::builder::ValueParser::string()),
        )
        .arg(
            Arg::new("image_size")
                .short('z')
                .long("size")
                .help("The size of the image to be rendered. Defaults to '800x600'")
                .action(ArgAction::Set)
                .value_name("STRING")
                .value_parser(clap::builder::ValueParser::string()),
        )
        .arg(
            Arg::new("tiled")
                .short('t')
                .long("tiled")
                .help("The size of the tiles for the shader. Defaults to '80x80'")
                .action(ArgAction::Set)
                .value_name("STRING")
                .value_parser(clap::builder::ValueParser::string()),
        )
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .help("The number of rendering iterations. Defaults to 1")
                .action(ArgAction::Set)
                .value_name("STRING")
                .value_parser(clap::builder::ValueParser::string()),
        )
        .arg(
            Arg::new("write")
                .short('w')
                .long("write")
                .help("Writes the shader image after each completed tile")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let mut path = std::path::PathBuf::new();
    if let Some(c) = matches.get_one::<String>("source") {
        path.push(c);
    }

    let mut function_name = "main";
    if let Some(f) = matches.get_one::<String>("function") {
        function_name = f;
    }

    let mut high_precision = true;
    let mut precision_str = "64".to_string();
    if let Some(precision) = matches.get_one::<String>("precision") {
        if precision == "32" {
            high_precision = false;
            precision_str = "32".to_string();
        }
    }

    let mut arguments: Vec<WasmValue> = vec![];
    let mut arg_string = String::new();
    if let Some(number_str) = matches.get_one::<String>("arg") {
        arg_string = number_str.clone();
        match number_str.parse::<i64>() {
            Ok(num) => {
                if high_precision {
                    arguments.push(WasmValue::I64(num))
                } else {
                    arguments.push(WasmValue::I32(num as i32))
                }
            }
            Err(_) => match number_str.parse::<f64>() {
                Ok(num) => {
                    if high_precision {
                        arguments.push(WasmValue::F64(num))
                    } else {
                        arguments.push(WasmValue::F32(num as f32))
                    }
                }
                Err(_) => println!("Invalid number format"),
            },
        }
    }

    let mut width = 800;
    let mut height = 600;
    if let Some(f) = matches.get_one::<String>("image_size") {
        let parts: Vec<&str> = f.split('x').collect();
        if parts.len() == 2 {
            width = parts[0].parse::<usize>().unwrap();
            height = parts[1].parse::<usize>().unwrap();
        }
    }

    let mut tiled: Option<(usize, usize)> = Some((80, 80));
    if let Some(f) = matches.get_one::<String>("tiled") {
        let parts: Vec<&str> = f.split('x').collect();
        if parts.len() == 2 {
            let x = parts[0].parse::<usize>().unwrap();
            let y = parts[1].parse::<usize>().unwrap();
            tiled = Some((x, y));
        }
    }

    let mut iterations: usize = 1;
    if let Some(str) = matches.get_one::<String>("iterations") {
        if let Ok(i) = (*str).parse::<usize>() {
            iterations = i;
        }
    }

    let mut write: bool = false;
    if *matches.get_one::<bool>("write").unwrap_or(&false) {
        write = true;
    }

    println!(
        "Input '{}'. Function '{}'. Precision: '{}'. Argument '{}'.",
        path.to_str().unwrap(),
        function_name,
        precision_str,
        if arguments.is_empty() {
            "None"
        } else {
            &arg_string
        }
    );

    let as_shader = function_name.starts_with("shader");

    let rpu = RPU::new();
    let rc = rpu.compile_to_wat_from_path(path.clone(), high_precision);

    match rc {
        Ok(wat) => {
            // Write the WAT to file
            path.set_extension("wat");
            _ = std::fs::write(path.clone(), wat.clone());

            if !as_shader {
                let rc = rpu.compile_wat_and_run(&wat, "main", arguments, high_precision);
                match rc {
                    Ok(values) => {
                        println!("Returns {:?}", values);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            } else {
                let mut buffer = Arc::new(Mutex::new(ColorBuffer::new(width, height)));
                if write {
                    if let Ok(mut buffer) = buffer.lock() {
                        path.set_extension("png");
                        buffer.file_path = Some(path.clone());
                    }
                }

                println!("Computing {} iteration(s) ...", iterations);

                if let Some(tiled) = tiled {
                    let rc = rpu.compile_wat_and_run_as_tiled_shader(
                        &wat,
                        "shader",
                        &mut buffer,
                        tiled,
                        iterations,
                        high_precision,
                    );
                    match rc {
                        Ok(_) => {
                            path.set_extension("png");
                            println!("Saved image as {:?}.", path);
                            buffer.lock().unwrap().save(path);
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                } else {
                    let mut buffer = ColorBuffer::new(width, height);

                    let rc = rpu.compile_wat_and_run_as_shader(
                        &wat,
                        "shader",
                        &mut buffer,
                        high_precision,
                    );
                    match rc {
                        Ok(_) => {
                            path.set_extension("png");
                            println!("Saved image as {:?}.", path);
                            buffer.save(path);
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
