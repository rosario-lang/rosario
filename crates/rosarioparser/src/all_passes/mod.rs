pub mod generic_solver_pass;
pub mod implicit_trait_pass;
pub mod static_number_solver_pass;

pub use generic_solver_pass::*;
pub use implicit_trait_pass::*;
pub use static_number_solver_pass::*;

use crate::parser::Parser;

pub fn standard_pass(parser: &mut Parser) {
    generic_solver_pass(parser);
    implicit_trait_pass(parser);
    static_number_solver_pass(parser);
}
