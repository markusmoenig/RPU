pub mod ast;
pub mod buffer;
pub mod compile;
pub mod ctx;
pub mod environment;
pub mod parser;
pub mod rpu;
pub mod scanner;

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::buffer::ColorBuffer;
    pub use crate::compile::CompileVisitor;
    pub use crate::ctx::*;
    pub use crate::environment::Environment;
    pub use crate::parser::*;
    pub use crate::rpu::RPU;
    pub use crate::scanner::*;
    pub use maths_rs::prelude::*;
    pub use rustc_hash::FxHashMap;
    pub use wasmer::Value as WasmValue;
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
    fn vec_length() {
        let rpu = RPU::new();
        assert_eq!(
            rpu.compile_and_run(
                "export float main() {
                    return length(vec3(1.0, 3.0, 5.0));
                }
                ",
                "main",
                vec![],
            ),
            Ok(vec![WasmValue::F64(5.916079783099616)])
        );
    }

    #[test]
    fn vec_normalize() {
        let rpu = RPU::new();
        assert_eq!(
            rpu.compile_and_run(
                "export vec3 main() {
                    vec3 result = normalize(vec3(1.0, 3.0, 5.0));
                    return result;
                }
                ",
                "main",
                vec![],
            ),
            Ok(vec![
                WasmValue::F64(0.1690308509457033),
                WasmValue::F64(0.50709255283711),
                WasmValue::F64(0.8451542547285166)
            ])
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
