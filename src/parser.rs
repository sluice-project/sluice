// TODO: LL(1) parser for sculpt grammar

use token::Token;

// Trait for parsing that will be implemented by each
// non-terminal in the grammar. Each implementation of
// this trait can be thought of as a parser combinator.
trait Parsing where Self: Sized {
  fn parse(token_vector : Vec<Token>) -> Result<Self, &'static str> ;
}

#[derive(Debug)]
pub enum Identifier {
  Identifier(String),
}

impl Parsing for Identifier {
  fn parse(mut token_vector : Vec<Token>) -> Result<Identifier, &'static str> {
    assert_eq!(token_vector.len(), 1, "token_vector too long for Identifier");
    let identifier_token = token_vector.remove(0);
    match identifier_token {
      Token::Identifier(s) => Ok(Identifier::Identifier(s)),
      _                    => Err("Invalid Identifier ")
    }
  }
}

#[derive(Debug)]
pub enum Operand {
  Identifier(String),
  Values(String),
}

impl Parsing for Operand {
  fn parse(mut token_vector : Vec<Token>) -> Result<Operand, &'static str> {
    assert_eq!(token_vector.len(), 1, "token_vector too long for parse_operand");
    let operand_token = token_vector.remove(0);
    match operand_token {
      Token::Identifier(i) => return Ok(Operand::Identifier(i)),
      Token::Values(v)     => return Ok(Operand::Values(v)),
      _                    => return Err("invalid operand")
    }
  }
}

#[derive(Debug)]
pub enum BinOpType {
  Plus,
  Minus,
  Mul,
  Div,
  Modulo
}

fn get_bin_op(t : Token) -> BinOpType {
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

impl Parsing for ExprRight {
  fn parse(mut token_vector : Vec<Token>) -> Result<ExprRight, &'static str> {
    if token_vector.len() == 0 {
      return Ok(ExprRight::Empty());
    }
  
    if token_vector.len() < 2 {
      return Err("invalid input to parse_expr_right"); // TODO: print token_vector as part of err message
    }
  
    let op_type = token_vector.remove(0);
    let operand = token_vector.remove(0);
    return match op_type {
      e @ Token::Plus  |
      e @ Token::Minus |
      e @ Token::Mul   |
      e @ Token::Div   |
      e @ Token::Modulo => Ok(ExprRight::BinOp(get_bin_op(e),
                                               Parsing::parse(vec!(operand)).unwrap(),
                                               Box::new(Parsing::parse(token_vector).unwrap()))),
      _          => Err("invalid operation type in expr_right") 
    }
  }
}


#[derive(Debug)]
pub enum Expr {
  Expr(Operand, ExprRight)
}

impl Parsing for Expr {
  fn parse(mut token_vector : Vec<Token>) -> Result<Expr, &'static str> {
    if token_vector.len() == 0 {
      return Err("insufficient tokens in parse_expr");
    }
    let operand_token = vec!(token_vector.remove(0));
    let expr_right_tokens = token_vector;
    return Ok(Expr::Expr(Parsing::parse(operand_token).unwrap(),
                         Parsing::parse(expr_right_tokens).unwrap()));
  }
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

#[test]
fn test_parse_operand() {
  let input =  r"5";
  println!("{:?}", Operand::parse(super::lexer::get_tokens(input)));
}

#[test]
fn test_parse_expr() {
  let input = r"7%5-5+6";
  println!("{:?}", Expr::parse(super::lexer::get_tokens(input)));
}
