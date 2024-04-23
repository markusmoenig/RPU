pub use rpu::prelude::*;

fn main() {
    let mut path_to_main = std::path::PathBuf::new();
    path_to_main.push("examples/main.rpu");

    let mut rpu = RPU::new();
    let _rc = rpu.compile_from_path(path_to_main);
}
