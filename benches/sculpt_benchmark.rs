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
                        ".repeat(1000);
  b.iter(|| { lexer::get_tokens(&input_program); } );
}
