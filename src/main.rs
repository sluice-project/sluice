extern crate regex;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

lazy_static! {
  static ref KEYWORDS    : Regex = Regex::new(r"^(if|else|and|or|xor|pow|true|false)$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z$_][A-Za-z0-9$_]*$").unwrap();
  static ref REL_OPS     : Regex = Regex::new(r"^(==|!=|>=|<=|>|<)$").unwrap();
  static ref ARITH_OPS   : Regex = Regex::new(r"^(+|-|/|\*|%)$").unwrap();
  static ref OTHER_OPS   : Regex = Regex::new(r"^(=|{|}|;|\(|\)|\.|\[|\])$").unwrap();
}

fn main() {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    let input_string = "foobar is foo or bar or k1 or k2 or k3 or k4";
    let token_iter = input_string.split_whitespace();
    for token in token_iter {
      if KEYWORDS.is_match(token) {
        println!("Found a keyword");
      }
      println!("{}", token);
    }
    println!("Did our date match? {}", re.is_match("2014-01-01"));
}
