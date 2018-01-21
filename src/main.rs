#[macro_use]
extern crate lazy_static;

mod token;
mod lexer;
mod lexer_tests;
mod parser;
mod parser_impl;
mod parser_tests;
use parser_impl::Parsing;

fn main() {
  let input_program = r"snippet fun(a, b, c, x, y, ) {
                          static x = 0;
                          a = x;
                          b = y;
                          m = 5;
                        }
                        snippet foo(a, b, c, ) {
                          static x = 1;
                          x = 5;
                        }
                        (foo, fun)
                        ";
  let tokens = & mut lexer::get_tokens(input_program);
  parser::Prog::parse(tokens);
  assert!(tokens.is_empty(), "tokens is not empty");
}
