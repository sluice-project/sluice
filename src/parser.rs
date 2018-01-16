// TODO: LL(1) parser for sculpt grammar

use tokens::Tokens;

#[derive(Debug)]
pub enum Operand {
  Identifier(String),
  Values(String),
}

pub fn parse_operand(mut token_vector : Vec<Tokens>) -> Result<Operand, &'static str> {
  use tokens::Tokens::*;
  assert_eq!(token_vector.len(), 1, "token_vector too long for parse_operand");
  let operand_token = token_vector.remove(0);
  match operand_token {
    Identifier(i) => return Ok(Operand::Identifier(i)),
    Values(v)     => return Ok(Operand::Values(v)),
    _             => return Err("invalid operand")
  }
}

// pub fn parse_expr(token_vector : Vec<Tokens>) {
//   return Expr(parse_operand(token_vector[..1]), parse_expr_right(token_vector[1..]));
// }

pub fn parse_prog(token_vector : Vec<Tokens>)  {
  println!("Within parser, doing nothing");
}

#[test]
fn test_parse_operand() {
  let input =  r"5";
  println!("{:?}", parse_operand(super::lexer::get_tokens(input)));
}
