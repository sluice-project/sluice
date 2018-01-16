// TODO: LL(1) parser for sculpt grammar

use token::Token;

#[derive(Debug)]
pub enum Identifier {
  Identifier(String),
}

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

pub fn get_bin_op(t : Token) -> BinOpType {
  match t {
    Token::Plus  => BinOpType::Plus,
    Token::Minus => BinOpType::Minus,
    Token::Mul   => BinOpType::Mul,
    Token::Div   => BinOpType::Div,
    Token::Modulo=> BinOpType::Modulo,
    _             => panic!("Invalid BinOpType")
  }
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

pub enum Statement {
  Statement(Identifier, Expr)
}

pub enum Statements {
  Statements(Statement, Box<Statements>)
}

pub fn parse_prog(token_vector : Vec<Token>)  {
  println!("Within parser, doing nothing");
}
