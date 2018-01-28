#![feature(test)]

extern crate test;
extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::symbol_table_pass::SymbolTablePass;
use sculpt::define_before_use_pass::DefineBeforeUsePass;
use sculpt::parser_impl::Parsing;
use sculpt::tree_fold::TreeFold;
use std::collections::HashSet;
use test::Bencher;

#[bench]
fn bench_lexer(b: &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(100);
  b.iter(|| { lexer::get_tokens(&input_program); });
}

#[bench]
fn bench_parser(b: &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(100);
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              parser::Prog::parse(tokens); } );
}

#[bench]
fn bench_lexer_large(b: &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(1000);
  b.iter(|| { lexer::get_tokens(&input_program); } );
}

#[bench]
fn bench_parser_large(b: &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              parser::Prog::parse(tokens); } );
}
