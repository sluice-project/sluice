extern crate regex;
use self::regex::Regex;
use std::str::FromStr;

lazy_static! {
  static ref TOKENS      : Regex = Regex::new(r"[0-9]+|[A-Za-z_][A-Za-z0-9_]*|==|!=|>=|<=|>|<|\+|-|/|\*|%|\{|\}|\(|\)|\[|\]|=|;|\.|,|\?|:|\S+").unwrap();
  static ref KEYWORDS    : Regex = Regex::new(r"^(static|snippet|and|or|not)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
}

use token::Token;
pub fn get_single_token(token : &str) -> Token {
  if KEYWORDS.is_match(token) {
    return match token {
     "static" => Token::Static,
     "snippet"=> Token::Snippet,
     "and"    => Token::BooleanAnd,
     "or"     => Token::BooleanOr,
     "not"    => Token::BooleanNot,
     _        => panic!("Unrecognized token: {}", token)
    }
  } else if IDENTIFIERS.is_match(token) {
    return Token::Identifier(String::from_str(token).unwrap());
  } else if VALUES.is_match(token) {
    return Token::Values(String::from_str(token).unwrap());
  } else {
    return match token {
      ":" => Token::Colon,
      ";" => Token::SemiColon,
      "." => Token::Period,
      "," => Token::Comma,

      "[" => Token::SqBktLeft,
      "]" => Token::SqBktRight,
      "(" => Token::ParenLeft,
      ")" => Token::ParenRight,
      "{" => Token::BraceLeft,
      "}" => Token::BraceRight,

      "+" => Token::Plus,
      "-" => Token::Minus,
      "*" => Token::Mul,
      "/" => Token::Div,
      "?" => Token::Cond,
      "%" => Token::Modulo,

      "=="=> Token::Equal,
      "!="=> Token::NotEqual,
      "<="=> Token::LTEQOp,
      ">="=> Token::GTEQOp,
      "<" => Token::LessThan,
      ">" => Token::GreaterThan,

      "=" => Token::Assign, 
      _   => panic!("Unrecognized token: {}", token)
    }
  }
}

pub fn get_tokens(input_program : &str) -> Vec<Token> {
  let mut token_array = Vec::new();
  for cap in TOKENS.captures_iter(input_program) {
    let ref token = cap[0];
    token_array.push(get_single_token(token));
  }
  return token_array;
}
