#[macro_use]
extern crate lazy_static;

mod token;
pub mod lexer;
pub mod grammar;
pub mod parser;
pub mod tree_fold;
pub mod define_before_use_pass;
pub mod symbol_table_pass;
