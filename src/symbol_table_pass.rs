use super::grammar::*;
use std::collections::HashSet;
use tree_fold::TreeFold;
use std::str::FromStr;

// Compiler pass to get list of symbols in the program
pub struct SymbolTablePass;

// Override portion that handles identifiers alone
impl TreeFold<HashSet<String>> for SymbolTablePass {
  fn visit_identifier(tree : &Identifier, collector : &mut HashSet<String>) {
    match tree {
      &Identifier::Identifier(ref s) => {collector.insert(String::from_str(s).unwrap());}
    }
  }
}
