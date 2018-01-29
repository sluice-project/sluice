extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::symbol_table_pass::SymbolTablePass;
use sculpt::define_before_use_pass::DefineBeforeUsePass;
use sculpt::parser_impl::Parsing;
use sculpt::tree_fold::TreeFold;
use std::collections::HashSet;

// Main compiler binary
// Takes an input sculpt program and produces a refined program
fn main() {
  let input_program = r"snippet fun(a, b, c, x, y, ) {
                          y = 5;
                          a = x;
                          b = y;
                          m = 5;
                        }
                        snippet foo(a, b, c, ) {
                          static p = 1;
                          q = 5;
                        }
                        (foo, fun)
                        ";
  // Lexing
  let tokens = & mut lexer::get_tokens(input_program);

  // parsing
  let parse_tree = parser::Prog::parse(&mut tokens.iter().peekable());
  assert!(tokens.is_empty(), "Tokens is not empty.");
  println!("Parse tree: {:?}\n", parse_tree);

  // symbol table generation
  let mut symbol_table = HashSet::new();
  SymbolTablePass::visit_prog(&parse_tree, &mut symbol_table);
  println!("Symbol table: {:?}",symbol_table);

  // Check that identifiers are defined before use
  let mut definitions = HashSet::new();
  DefineBeforeUsePass::visit_prog(&parse_tree, &mut definitions);
}
