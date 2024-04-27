pub use rpu::prelude::*;

fn main() {
    let mut path = std::path::PathBuf::new();
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
            _ = std::fs::write(path, wat.clone());

            let rc = rpu.compile_wat_and_run(&wat, "main", vec![]);
            match rc {
                Ok(values) => {
                    println!("{:?}", values);
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
