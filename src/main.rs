extern crate regex;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

lazy_static! {
  static ref KEYWORDS    : Regex = Regex::new(r"^(if|else|and|or|xor|pow|true|false)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z$_][A-Za-z0-9$_]*$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
  static ref REL_OPS     : Regex = Regex::new(r"^(==|!=|>=|<=|>|<)$").unwrap();
  static ref ARITH_OPS   : Regex = Regex::new(r"^(\+|-|/|\*|%)$").unwrap();
  static ref GROUPING_OPS: Regex = Regex::new(r"^(\{|\}|\(|\)|\[|\])$").unwrap();
  static ref OTHER_OPS   : Regex = Regex::new(r"^(=|;|\.)$").unwrap();
}

fn main() {
    let input_string = "if ( a >= b ) { a = x ; b = y ; } else if ( c >= d ) { m == 5 ; } ";
    // Split string into tokens at whitespaces
    let token_iter = input_string.split_whitespace();
    for token in token_iter {
      if KEYWORDS.is_match(token) {
        print!("Found a keyword");
      } else if IDENTIFIERS.is_match(token) {
        print!("Found an identifier");
      } else if VALUES.is_match(token) {
        print!("Found a value");
      } else if REL_OPS.is_match(token) {
        print!("Found a relational operator");
      } else if ARITH_OPS.is_match(token) {
        print!("Found an arithmetic operator");
      } else if GROUPING_OPS.is_match(token) {
        print!("Found a grouping operator");
      } else if OTHER_OPS.is_match(token) {
        print!("Found other operator");
      } else {
        print!("Found nothing here!!!");
      }
      println!(" {}", token);
    }
}
