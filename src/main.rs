extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::semantic_checker::SymbolTablePass;
use sculpt::semantic_checker::DefineBeforeUsePass;
use sculpt::parser_impl::Parsing;
use sculpt::tree_fold::TreeFold;
use std::collections::HashSet;

// Main compiler binary
// Takes an input sculpt program and produces a refined program
fn main() {
  let input_program = r"snippet fun(a, b, c, x, y, ) {
                          static x = 0;
                          y = 5;
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
  // Lexing
  let tokens = & mut lexer::get_tokens(input_program);

  // parsing
  let parse_tree = parser::Prog::parse(tokens);
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
