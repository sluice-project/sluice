#![feature(test)]

extern crate test;
extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::def_use::DefUse;
use sculpt::def_use::SymbolTableCollector;
use sculpt::tree_fold::TreeFold;
use std::collections::HashMap;
use test::Bencher;

#[bench]
fn bench_lexer(b : &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(100);
  b.iter(|| { lexer::get_tokens(&input_program); });
}

#[bench]
fn bench_parser(b : &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(100);
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              let token_iter = & mut tokens.iter().peekable();
              parser::parse_prog(token_iter); } );
}

#[bench]
fn bench_lexer_large(b : &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(1000);
  b.iter(|| { lexer::get_tokens(&input_program); } );
}

#[bench]
fn bench_parser_large(b : &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              let token_iter = & mut tokens.iter().peekable();
              parser::parse_prog(token_iter); } );
}

#[bench]
fn bench_def_use(b : &mut Bencher) {
  let mut input_program = "".to_string();
  for i in 0..1000 {
    input_program += r"snippet foo";
    input_program += &i.to_string();
    input_program += r"(a, b, c, ) { d = 1; x = d; }";
  }
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              let token_iter = & mut tokens.iter().peekable();
              let parse_tree = parser::parse_prog(token_iter);
              let mut def_use_collector = SymbolTableCollector { current_snippet : "", symbol_table : HashMap::new() };
              DefUse::visit_prog(&parse_tree, &mut def_use_collector); } );
}
