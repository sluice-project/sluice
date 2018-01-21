use token::Token;

#[derive(Debug)]
pub enum Prog {
  Prog(Snippets, Connections)
}

#[derive(Debug)]
pub enum Snippets {
  Snippets(Snippet, Box<Snippets>),
  Empty()
}

#[derive(Debug)]
pub enum Snippet {
  Snippet(Identifier, IdList, Initializers, Statements) 
}

#[derive(Debug)]
pub enum Connections {
  Connections(Identifier, Identifier, Box<Connections>),
  Empty()
}

#[derive(Debug)]
pub enum IdList {
  IdList(Identifier, Box<IdList>),
  Empty()
}

#[derive(Debug)]
pub enum Initializers {
  Initializers(Initializer, Box<Initializers>),
  Empty()
}

#[derive(Debug)]
pub enum Initializer {
  Initializer(Identifier, Value)
}

#[derive(Debug)]
pub enum Statements {
  Statements(Statement, Box<Statements>),
  Empty()
}

#[derive(Debug)]
pub enum Statement {
  Statement(Identifier, Expr)
}

#[derive(Debug)]
pub enum Expr {
  Expr(Operand, ExprRight)
}

#[derive(Debug)]
pub enum ExprRight {
  BinOp(BinOpType, Operand),
  Cond(Operand, Operand),
  Empty()
}

#[derive(Debug)]
pub enum BinOpType {
  BooleanAnd,
  BooleanOr,
  Plus,
  Minus,
  Mul,
  Div,
  Modulo,
  Equal,
  NotEqual,
  LTEQOp,
  GTEQOp,
  LessThan,
  GreaterThan
}

pub fn get_bin_op(t : Token) -> BinOpType {
  match t {
    Token::BooleanAnd  => BinOpType::BooleanAnd,
    Token::BooleanOr   => BinOpType::BooleanOr,
    Token::Plus        => BinOpType::Plus,
    Token::Minus       => BinOpType::Minus,
    Token::Mul         => BinOpType::Mul,
    Token::Div         => BinOpType::Div,
    Token::Modulo      => BinOpType::Modulo,
    Token::Equal       => BinOpType::Equal,
    Token::NotEqual    => BinOpType::NotEqual,
    Token::LTEQOp      => BinOpType::LTEQOp,
    Token::GTEQOp      => BinOpType::GTEQOp,
    Token::LessThan    => BinOpType::LessThan,
    Token::GreaterThan => BinOpType::GreaterThan,
    _                  => panic!("Invalid BinOpType")
  }
}

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
  Value(String),
}
