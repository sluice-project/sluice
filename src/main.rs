extern crate sculpt;

use sculpt::lexer;
use sculpt::parser;
use sculpt::def_use::DefUse;
use sculpt::tree_fold::TreeFold;
use std::env;
use std::fs::File;
use std::io::prelude::*;
// Main compiler binary
// Takes an input sculpt program and produces a refined program
fn main() {
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];
  println!("In file {}", filename);
  let mut f = File::open(filename).expect("File not found");
  let mut contents = String::new();
  f.read_to_string(&mut contents).expect("Something went wrong reading the file");

  let tokens = & mut lexer::get_tokens(&contents);
  // parsing
  let token_iter = & mut tokens.iter().peekable();
  let parse_tree = parser::parse_prog(token_iter);
  assert!(token_iter.peek().is_none(), "Token iterator is not empty.");
  println!("Parse tree: {:?}\n", parse_tree);

  // Check that identifiers are defined before use
  let mut def_use = DefUse::new();
  def_use.visit_prog(&parse_tree);
}
