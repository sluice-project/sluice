extern crate regex;
use self::regex::Regex;
use std::str::FromStr;

lazy_static! {
  static ref TOKENS      : Regex = Regex::new(r"[0-9]+|[A-Za-z_][A-Za-z0-9_]*|==|!=|>=|<=|>|<|\+|-|/|\*|%|\{|\}|\(|\)|\[|\]|=|;|\.|,|\?|:|\S+").unwrap();
  static ref KEYWORDS    : Regex = Regex::new(r"^(static|snippet|and|or|not)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
}

use tokens::Tokens;
pub fn get_single_token(token : &str) -> Tokens {
  use tokens::Tokens::*;
  if KEYWORDS.is_match(token) {
    return match token {
     "static" => Static,
     "snippet"=> Snippet,
     "and"    => BooleanAnd,
     "or"     => BooleanOr,
     "not"    => BooleanNot,
     _        => panic!("Unrecognized token: {}", token)
    }
  } else if IDENTIFIERS.is_match(token) {
    return Identifier(String::from_str(token).unwrap());
  } else if VALUES.is_match(token) {
    return Values(String::from_str(token).unwrap());
  } else {
    return match token {
      ":" => Colon,
      ";" => SemiColon,
      "." => Period,
      "," => Comma,

      "[" => SqBktLeft,
      "]" => SqBktRight,
      "(" => ParenLeft,
      ")" => ParenRight,
      "{" => BraceLeft,
      "}" => BraceRight,

      "+" => Plus,
      "-" => Minus,
      "*" => Mul,
      "/" => Div,
      "?" => Cond,
      "%" => Modulo,

      "=="=> Equal,
      "!="=> NotEqual,
      "<="=> LTEQOp,
      ">="=> GTEQOp,
      "<" => LessThan,
      ">" => GreaterThan,

      "=" => Assign, 
      _   => panic!("Unrecognized token: {}", token)
    }
  }
}

pub fn get_tokens(input_program : &str) -> Vec<Tokens> {
  let mut token_array = Vec::new();
  for cap in TOKENS.captures_iter(input_program) {
    let ref token = cap[0];
    token_array.push(get_single_token(token));
  }
  return token_array;
}

#[test]
fn test_lexer_with_spaces() {
  let input_program = r"snippet fun ( a , b , c , x , y ) {
                          static x = 0 ;
                          if ( a >= b ) {
                            a = x ;
                            b = y ;
                          } elif ( c >= d ) {
                            m == 5 ;
                          }
                        }";
  println!("{:?}", get_tokens(input_program));
}

#[test]
fn test_lexer_wo_spaces() {
  let input_program = r"snippet fun(a, b, c, x, y) {
                          static x = 0;
                          if (a >= b) {
                            a = x;
                            b = y;
                          } elif (c >= d) {
                            m == 5;
                          }
                        }";
  println!("{:?}", get_tokens(input_program));
}
