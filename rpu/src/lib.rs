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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
