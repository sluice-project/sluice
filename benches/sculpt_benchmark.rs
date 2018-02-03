#![feature(test)]

extern crate test;
extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::symbol_table_pass::SymbolTablePass;
use sculpt::define_before_use_pass::DefineBeforeUsePass;
use sculpt::tree_fold::TreeFold;
use std::collections::HashSet;
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
fn bench_symbol_table(b : &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              let token_iter = & mut tokens.iter().peekable();
              let parse_tree = parser::parse_prog(token_iter);
              let mut symbol_table = HashSet::new();
              SymbolTablePass::visit_prog(&parse_tree, &mut symbol_table); });
}

#[bench]
fn bench_def_use(b : &mut Bencher) {
  let input_program = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(1000);
  b.iter(|| { let tokens = & mut lexer::get_tokens(&input_program);
              let token_iter = & mut tokens.iter().peekable();
              let parse_tree = parser::parse_prog(token_iter);
              let mut symbol_table = HashSet::new();
              DefineBeforeUsePass::visit_prog(&parse_tree, &mut symbol_table); });
}
