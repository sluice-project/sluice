extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::symbol_table_pass::SymbolTablePass;
use sculpt::define_before_use_pass::DefineBeforeUsePass;
use sculpt::parser_impl::Parsing;
use sculpt::tree_fold::TreeFold;
use std::collections::HashSet;

fn run_def_use(input_program : &str) {
  // Lexing
  let tokens = & mut lexer::get_tokens(input_program);

  // parsing
  let token_iter = & mut tokens.iter().peekable();
  let parse_tree = parser::Prog::parse(token_iter);
  assert!(token_iter.peek().is_none(), "token_iter is not empty.");
  println!("Parse tree: {:?}\n", parse_tree);

  // symbol table generation
  let mut symbol_table = HashSet::new();
  SymbolTablePass::visit_prog(&parse_tree, &mut symbol_table);
  println!("Symbol table: {:?}",symbol_table);

  // Check that identifiers are defined before use
  let mut definitions = HashSet::new();
  DefineBeforeUsePass::visit_prog(&parse_tree, &mut definitions);
}

#[test]
#[should_panic(expected="y used before definition")]
fn test_du_undefined(){
  let input_program = r"snippet fun(x, ) {
                          b = y;
                          m = 5;
                        }
                        ";
  run_def_use(input_program);
}

#[test]
fn test_du_defined_in_arg_list(){
  let input_program = r"snippet foo(a, b, c, ) {
                          x = a;
                        }
                        ";
  run_def_use(input_program);
}

#[test]
fn test_du_defined_in_prog(){
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ";
  run_def_use(input_program);
}

#[test]
#[should_panic(expected="Can't initialize x that is already defined")]
fn test_du_redefined(){
  let input_program = r"snippet fun(a, b, c, x, y, ) {
                          static x = 0;
                        }
                        ";
  run_def_use(input_program);
}

#[test]
#[should_panic(expected="foo connected, but not defined")]
fn test_du_undefined_snippet() {
  let input_program = r"(foo, fun)";
  run_def_use(input_program);
}
