pub use rpu::prelude::*;
//use std::sync::{Arc, Mutex};
use clap::{Arg, ArgAction, Command};

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
                .help("Sets the source file to use")
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
            Arg::new("arg")
                .short('a')
                .long("argument")
                .help("The argument for the function. Could be an integer or a float")
                .action(ArgAction::Set)
                .value_name("NUMBER")
                .value_parser(clap::builder::ValueParser::string()),
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

    let mut arguments: Vec<WasmValue> = vec![];
    let mut arg_string = String::new();
    if let Some(number_str) = matches.get_one::<String>("arg") {
        arg_string = number_str.clone();
        match number_str.parse::<i64>() {
            Ok(num) => arguments.push(WasmValue::I64(num)),
            Err(_) => match number_str.parse::<f64>() {
                Ok(num) => arguments.push(WasmValue::F64(num)),
                Err(_) => println!("Invalid number format"),
            },
        }
    }

    println!(
        "Input '{}'. Function '{}'. Argument '{}'.",
        path.to_str().unwrap(),
        function_name,
        if arguments.is_empty() {
            "None"
        } else {
            &arg_string
        }
    );

    let as_shader = function_name.starts_with("shader");

    let rpu = RPU::new();
    let rc = rpu.compile_to_wat_from_path(path.clone());

    match rc {
        Ok(wat) => {
            // Write the WAT to file
            path.set_extension("wat");
            _ = std::fs::write(path.clone(), wat.clone());

            if !as_shader {
                let rc = rpu.compile_wat_and_run(&wat, "main", arguments);
                match rc {
                    Ok(values) => {
                        println!("Returns {:?}", values);
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            } else {
                let mut buffer = ColorBuffer::new(800, 600);
                let rc = rpu.compile_wat_and_run_as_shader_sync(&wat, "shader", &mut buffer);
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
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
