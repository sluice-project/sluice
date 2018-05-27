extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::def_use::DefUse;
use sculpt::def_use::VariableCollector;
use sculpt::tree_fold::TreeFold;
use std::collections::HashMap;
use std::collections::HashSet;

// Main compiler binary
// Takes an input sculpt program and produces a refined program
fn main() {
  let input_program = r"snippet fun(a, b, c, x, y, ) {
                          z = a + b;
                          q = x;
                          r = y;
                          m = 5;
                        }
                        snippet foo(a, b, c, ) {
                          persistent p = 1;
                          persistent m = {1, 2, 3, };
                          q = 5;
                          z[5] = 6;
                          h = z[7];
                          m = 5;
                        }
                        (foo, fun)
                        ";
  // Lexing
  let tokens = & mut lexer::get_tokens(input_program);

  // parsing
  let token_iter = & mut tokens.iter().peekable();
  let parse_tree = parser::parse_prog(token_iter);
  assert!(token_iter.peek().is_none(), "Token iterator is not empty.");
  println!("Parse tree: {:?}\n", parse_tree);

  // Check that identifiers are defined before use
  let mut def_use_collector = VariableCollector { current_snippet : "",
                                                  transient_vars : HashMap::new(),
                                                  persistent_vars : HashMap::new(),
                                                  snippet_set : HashSet::new() };
  DefUse::visit_prog(&parse_tree, &mut def_use_collector);
}
