use token::Token;

#[derive(Debug)]
pub enum Identifier {
  Identifier(String),
}

#[derive(Debug)]
pub enum Value {
  Value(String)
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

#[derive(Debug)]
pub enum Statement {
  Statement(Identifier, Expr)
}

#[derive(Debug)]
pub enum Statements {
  Statements(Statement, Box<Statements>),
  Empty()
}

#[derive(Debug)]
pub enum Initializer {
  Initializer(Identifier, Value)
}

#[derive(Debug)]
pub enum Initializers {
  Initializers(Initializer, Box<Initializers>),
  Empty()
}

#[derive(Debug)]
pub enum IdList {
  IdList(Identifier, Box<IdList>),
  Empty()
}

#[derive(Debug)]
pub enum Connections {
  Connections(Identifier, Identifier, Box<Connections>),
  Empty()
}

#[derive(Debug)]
pub enum Snippet {
  Snippet(Identifier, IdList, Initializers, Statements) 
}

#[derive(Debug)]
pub enum Snippets {
  Snippets(Snippet, Box<Snippets>),
  Empty()
}

#[derive(Debug)]
pub enum Prog {
  Prog(Snippets, Connections)
}

pub fn parse_prog(token_vector : Vec<Token>)  {
  println!("Within parser, doing nothing");
}
