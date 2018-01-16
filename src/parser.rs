// TODO: LL(1) parser for sculpt grammar

use tokens::Tokens;

#[derive(Debug)]
pub enum Operand {
  Identifier(String),
  Values(String),
}

#[derive(Debug)]
pub enum BinOpType {
  Plus,
  Minus,
  Mul,
  Div,
  Modulo
}

#[derive(Debug)]
pub enum ExprRight {
  BinOp(BinOpType, Operand, Box<ExprRight>),
  Empty()
  // TODO: Ignoring conditionals for now
}

#[derive(Debug)]
pub enum Expr {
  Expr(Operand, ExprRight)
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

pub fn parse_expr_right(mut token_vector : Vec<Tokens>) -> Result<ExprRight, &'static str> {
  use tokens::Tokens::*;
  if token_vector.len() == 0 {
    return Ok(ExprRight::Empty());
  }

  if token_vector.len() < 2 {
    return Err("invalid input to parse_expr_right"); // TODO: print token_vector as part of err message
  }

  let op_type = token_vector.remove(0);
  let operand = token_vector.remove(0);
  return match op_type {
    Plus   => Ok(ExprRight::BinOp(BinOpType::Plus,  parse_operand(vec!(operand)).unwrap(), Box::new(parse_expr_right(token_vector).unwrap()) )),
    Minus  => Ok(ExprRight::BinOp(BinOpType::Minus, parse_operand(vec!(operand)).unwrap(), Box::new(parse_expr_right(token_vector).unwrap()) )),
    Mul    => Ok(ExprRight::BinOp(BinOpType::Mul,   parse_operand(vec!(operand)).unwrap(), Box::new(parse_expr_right(token_vector).unwrap()) )),
    Div    => Ok(ExprRight::BinOp(BinOpType::Div,   parse_operand(vec!(operand)).unwrap(), Box::new(parse_expr_right(token_vector).unwrap()) )),
    Modulo => Ok(ExprRight::BinOp(BinOpType::Modulo,parse_operand(vec!(operand)).unwrap(), Box::new(parse_expr_right(token_vector).unwrap()) )),
    _      => Err("invalid expr_right") 
  }
}

pub fn parse_expr(mut token_vector : Vec<Tokens>) -> Result<Expr, &'static str> {
  if token_vector.len() == 0 {
    return Err("insufficient tokens in parse_expr");
  }
  let operand_tokens = vec!(token_vector.remove(0));
  let expr_right_tokens = token_vector;
  return Ok(Expr::Expr(parse_operand(operand_tokens).unwrap(), parse_expr_right(expr_right_tokens).unwrap()));
}

pub fn parse_prog(token_vector : Vec<Tokens>)  {
  println!("Within parser, doing nothing");
}

#[test]
fn test_parse_operand() {
  let input =  r"5";
  println!("{:?}", parse_operand(super::lexer::get_tokens(input)));
}

#[test]
fn test_parse_expr() {
  let input = r"7%5-5+6";
  println!("{:?}", parse_expr(super::lexer::get_tokens(input)));
}
