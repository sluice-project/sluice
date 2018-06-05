#![feature(test)]

extern crate test;
extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::def_use::DefUse;
use sculpt::pretty_printer::PrettyPrinter;
use sculpt::tree_fold::TreeFold;
use std::collections::HashMap;
use std::collections::HashSet;
use test::Bencher;

fn create_test_input(size : u32) -> String {
  let mut input_program = "".to_string();
  for i in 0..size {
    input_program += r"snippet foo";
    input_program += &i.to_string();
    input_program += r"() { input a : bit<2>; input b : bit<2>; input c : bit<2>; transient d : bit<2>; transient x : bit<2>; d = 1; x = d; }";
  }
  return input_program;
}

#[bench]
fn bench_lexer(b : &mut Bencher) {
  b.iter(|| { lexer::get_tokens(&create_test_input(100)); });
}

#[bench]
fn bench_parser(b : &mut Bencher) {
  let input_program = &create_test_input(100);
  b.iter(|| { let tokens = & mut lexer::get_tokens(input_program);
              let token_iter = & mut tokens.iter().peekable();
              parser::parse_prog(token_iter); } );
}

#[bench]
fn bench_lexer_large(b : &mut Bencher) {
  b.iter(|| { lexer::get_tokens(&create_test_input(1000)); } );
}

#[bench]
fn bench_parser_large(b : &mut Bencher) {
  let input_program = &create_test_input(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(input_program);
              let token_iter = & mut tokens.iter().peekable();
              parser::parse_prog(token_iter); } );
}

#[bench]
fn bench_def_use(b : &mut Bencher) {
  let input_program = &create_test_input(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(input_program);
              let token_iter = & mut tokens.iter().peekable();
              let parse_tree = parser::parse_prog(token_iter);
              let mut def_use = DefUse { current_snippet : "",
                                         symbol_table : HashMap::new(),
                                         snippet_set : HashSet::new() };
              def_use.visit_prog(&parse_tree);} );
}

#[bench]
fn bench_pretty_printer(b : &mut Bencher) {
  let input_program = &create_test_input(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(input_program);
              let token_iter = & mut tokens.iter().peekable();
              let parse_tree = parser::parse_prog(token_iter);
              let mut pretty_printer = PrettyPrinter{ pretty_print_str : "".to_string() };
              pretty_printer.visit_prog(&parse_tree);} );
}
