use super::grammar::*;
use std::collections::HashSet;
use tree_fold::TreeFold;

// Compiler pass to get list of symbols in the program
pub struct SymbolTablePass;

// Override portion that handles identifiers alone
impl<'a> TreeFold<'a, HashSet<&'a str>> for SymbolTablePass {
  fn visit_identifier(tree : &'a Identifier, collector : &mut HashSet<&'a str>) {
    match tree {
      &Identifier::Identifier(s) => {collector.insert(s);}
    }
  }
}
