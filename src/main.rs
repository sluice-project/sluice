extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::def_use::DefUse;
use sculpt::def_use::SymTableCollector;
use sculpt::tree_fold::TreeFold;
use std::collections::HashMap;
use std::collections::HashSet;

// Main compiler binary
// Takes an input sculpt program and produces a refined program
fn main() {
  let input_program = r"snippet fun(){
                          input a : bit<2>;
                          input b : bit<2>;
                          input c : bit<2>;
                          input x : bit<2>;
                          input y : bit<2>;
                          transient z : bit<2>;
                          transient r : bit<2>;
                          transient q : bit<2>;
                          transient m : bit<2>;
                          z = a + b;
                          q = x;
                          r = y;
                          m = 5;
                        }
                        snippet foo() {
                          input a : bit<2>;
                          input b : bit<2>;
                          input c : bit<2>;
                          persistent p : bit<2> = 1;
                          persistent m : bit<2>[3] = {1, 2, 3, };
                          transient z : bit<2>;
                          transient h : bit<2>;
                          transient q : bit<2>;
                          q = 5;
                          z[5] = 6;
                          h = z[7];
                          m = 5;
                        }
                        (foo, fun)
                        ";  // Lexing
  let tokens = & mut lexer::get_tokens(input_program);

  // parsing
  let token_iter = & mut tokens.iter().peekable();
  let parse_tree = parser::parse_prog(token_iter);
  assert!(token_iter.peek().is_none(), "Token iterator is not empty.");
  println!("Parse tree: {:?}\n", parse_tree);

  // Check that identifiers are defined before use
  let mut def_use_collector = SymTableCollector { current_snippet : "",
                                                  symbol_table : HashMap::new(),
                                                  snippet_set : HashSet::new() };
  DefUse::visit_prog(&parse_tree, &mut def_use_collector);
}
