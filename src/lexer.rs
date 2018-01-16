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
  if KEYWORDS.is_match(token) {
    return match token {
     "static" => Tokens::Static,
     "snippet"=> Tokens::Snippet,
     "and"    => Tokens::BooleanAnd,
     "or"     => Tokens::BooleanOr,
     "not"    => Tokens::BooleanNot,
     _        => panic!("Unrecognized token: {}", token)
    }
  } else if IDENTIFIERS.is_match(token) {
    return Tokens::Identifier(String::from_str(token).unwrap());
  } else if VALUES.is_match(token) {
    return Tokens::Values(String::from_str(token).unwrap());
  } else {
    return match token {
      ":" => Tokens::Colon,
      ";" => Tokens::SemiColon,
      "." => Tokens::Period,
      "," => Tokens::Comma,

      "[" => Tokens::SqBktLeft,
      "]" => Tokens::SqBktRight,
      "(" => Tokens::ParenLeft,
      ")" => Tokens::ParenRight,
      "{" => Tokens::BraceLeft,
      "}" => Tokens::BraceRight,

      "+" => Tokens::Plus,
      "-" => Tokens::Minus,
      "*" => Tokens::Mul,
      "/" => Tokens::Div,
      "?" => Tokens::Cond,
      "%" => Tokens::Modulo,

      "=="=> Tokens::Equal,
      "!="=> Tokens::NotEqual,
      "<="=> Tokens::LTEQOp,
      ">="=> Tokens::GTEQOp,
      "<" => Tokens::LessThan,
      ">" => Tokens::GreaterThan,

      "=" => Tokens::Assign, 
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
                        }
                        snippet foo(a, b, c) {
                          static x = 1;
                          x = 5;
                        }
                        (foo, fun) 
                        ";
  println!("{:?}", get_tokens(input_program));
}
