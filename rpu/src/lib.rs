pub mod ast;
pub mod ast_value;
pub mod buffer;
pub mod compile;
pub mod ctx;
pub mod environment;
pub mod error;
pub mod parser;
pub mod preprocessor;
pub mod rpu;
pub mod scanner;
pub mod tests;
pub mod varverifier;

// Re-exports
pub use crate::rpu::Tile;

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::ast_value::*;
    pub use crate::buffer::ColorBuffer;
    pub use crate::compile::CompileVisitor;
    pub use crate::ctx::*;
    pub use crate::environment::Environment;
    pub use crate::error::RPUError;
    pub use crate::parser::*;
    pub use crate::preprocessor::*;
    pub use crate::rpu::RPU;
    pub use crate::scanner::*;
    pub use crate::varverifier::*;
    pub use rustc_hash::FxHashMap;
    pub use wasmer::Value as WasmValue;
}

pub use rpu::RPU;
pub use wasmer::Value as WasmValue;
