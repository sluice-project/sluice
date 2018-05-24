extern crate regex;
use self::regex::Regex;

lazy_static! {
  static ref TOKENS      : Regex = Regex::new(r"[0-9]+|[A-Za-z_][A-Za-z0-9_]*|->|==|!=|>=|<=|>|<|\+|-|/|\*|%|\{|\}|\(|\)|\[|\]|=|;|,|\?|:|\S+").unwrap();
  static ref KEYWORDS    : Regex = Regex::new(r"^(static|snippet|and|or|not)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
}

use token::Token;
fn get_single_token(tok_str : &str) -> Token {
  if KEYWORDS.is_match(tok_str) {
    return match tok_str {
     "static" => Token::Static,
     "snippet"=> Token::Snippet,
     "and"    => Token::BooleanAnd,
     "or"     => Token::BooleanOr,
     "not"    => Token::BooleanNot,
     _        => panic!("Unrecognized token string: {}", tok_str)
    }
  } else if IDENTIFIERS.is_match(tok_str) {
    return Token::Identifier(tok_str);
  } else if VALUES.is_match(tok_str) {
    return Token::Value(tok_str.parse::<u32>().unwrap());
  } else {
    return match tok_str {
      ":" => Token::Colon,
      ";" => Token::SemiColon,
      "," => Token::Comma,
      "->"=> Token::Arrow,

      "(" => Token::ParenLeft,
      ")" => Token::ParenRight,
      "{" => Token::BraceLeft,
      "}" => Token::BraceRight,
      "[" => Token::SquareLeft,
      "]" => Token::SquareRight,

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
      _   => panic!("Unrecognized token string: {}", tok_str)
    }
  }
}

pub fn get_tokens(input_program : &str) -> Vec<Token> {
  let mut token_array = Vec::new();
  for cap in TOKENS.captures_iter(input_program) {
    let ref tok_str = cap.get(0).unwrap().as_str();
    token_array.push(get_single_token(tok_str));
  }
  return token_array;
}

#[cfg(test)]
mod tests {
  use super::get_tokens;
  
  #[test]
  fn test_lexer_full_prog() {
    let input_program = r"snippet fun ( a , b , c , x , y, ) {
                            static x = 0 ;
                            t1 = a >= b;
                            a = t1 ? x : a;
                            b = t1 ? y : b;
                            t2 = c >= d;
                            t3 = t2 and t1;
                            e = t2 ? m : 5;
                          }
                          snippet foo(a, b, c,) {
                            static x = 1;
                            x = 5;
                          }
                          (foo, fun) 
                          ";
    println!("{:?}", get_tokens(input_program));
  }
}
