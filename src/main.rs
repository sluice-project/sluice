extern crate sluice;
#[macro_use]
extern crate serde_json;
use sluice::lexer;
use sluice::parser;
use sluice::def_use::DefUse;
use sluice::tree_fold::TreeFold;
use sluice::trans_snippet::*;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use std::process;
const INCLUDE_DIR : &str = "net-progs/include/";

// Main compiler binary
// Takes an input sluice program and produces a P4 program for each network device
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
  let mut def_use = DefUse::new();
  // need to fix def_use. Conditional statements in if/else that modify the same lvalue cause error
  // example error from first.np:  'Redefining variable l that is already defined in fun.'

  // def_use.visit_prog(&parse_tree);
  println!("Parse tree: {:?}\n", parse_tree);

  let packet_file = format!("{}packet.np", INCLUDE_DIR);
  let mut f = File::open(packet_file).expect("File not found");
  let mut contents = String::new();
  f.read_to_string( &mut contents).expect("Something went wrong reading the file");
  let tokens = & mut lexer::get_tokens(&contents);
  let token_iter = & mut tokens.iter().peekable();
  let pkt_tree = parser::parse_import_packets(token_iter);
  println!("Packet tree: {:?}\n", pkt_tree);
  // process::exit(1);

  trans_snippets(&parse_tree.imports,&parse_tree.globals, &parse_tree.packets, &parse_tree.snippets, &pkt_tree);//, &mut my_dag);
  // Check that identifiers are defined before use
}

    // let mut field_decls : HashMap<String, VarType> = HashMap::new();
