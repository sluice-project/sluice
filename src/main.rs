extern crate regex;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

lazy_static! {
  static ref KEYWORDS : Regex = Regex::new("^(k1|k2|k3)$").unwrap();
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
