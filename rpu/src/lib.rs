pub mod ast;
pub mod parser;
pub mod print;
pub mod rpu;
pub mod scanner;

pub mod prelude {
    pub use crate::ast::*;
    pub use crate::parser::*;
    pub use crate::print::*;
    pub use crate::rpu::RPU;
    pub use crate::scanner::*;
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
