pub mod ast;
pub mod compile;
pub mod ctx;
pub mod environment;
//pub mod interpret;
pub mod parser;
pub mod rpu;
pub mod scanner;

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::compile::CompileVisitor;
    pub use crate::ctx::*;
    pub use crate::environment::Environment;
    //pub use crate::interpret::InterpretVisitor;
    pub use crate::parser::*;
    pub use crate::rpu::RPU;
    pub use crate::scanner::*;
    pub use maths_rs::prelude::*;
    pub use rustc_hash::FxHashMap;
}

pub use rpu::RPU;
pub use wasmer::Value as WasmValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let rpu = RPU::new();
        assert_eq!(
            rpu.compile_and_run(
                "export int main(int x) {
                    return x;
                }
                ",
                "main",
                vec![WasmValue::I64(2)],
            ),
            Ok(vec![WasmValue::I64(2)])
        );
    }

    #[test]
    fn scalar_mul() {
        let rpu = RPU::new();
        assert_eq!(
            rpu.compile_and_run(
                "export ivec2 main() {
                    return 2 * ivec2(1, 2) * 3;
                }
                ",
                "main",
                vec![],
            ),
            Ok(vec![WasmValue::I64(6), WasmValue::I64(12)])
        );
    }

    #[test]
    fn fib() {
        let rpu = RPU::new();
        let rc = rpu.compile_to_wat_from_path(std::path::PathBuf::from("../examples/fib.rpu"));
        match rc {
            Ok(wat) => {
                assert_eq!(
                    rpu.compile_wat_and_run(&wat, "main", vec![WasmValue::I64(10)]),
                    Ok(vec![WasmValue::I64(55)])
                );
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}
