// Trait for parsing that will be implemented by each
// non-terminal in the grammar. Each implementation of
// this trait can be thought of as a parser combinator.

use std;
use super::parser::*;
use super::token::Token;
use std::iter::Peekable;
use std::str::FromStr;

// Helper function to consume next token and match it against a specified token
// Throw an error if either:
// 1. token_iter is empty
// 2. the next token does not match
fn match_token(token_iter : & mut Peekable<std::slice::Iter<Token>>, expected : Token, error_msg : &'static str) {
  if !token_iter.peek().is_some() {
    panic!("token_iter is empty. Can't consume next token");
  } else {
    let next_token = token_iter.next().unwrap();
    if *next_token == expected {
      return;
    } else {
      panic!("\nInvalid token: {:?}, expected {:?}.\nError message: {:?}", next_token, expected, error_msg);
    }
  }
}

pub fn parse_prog<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Prog<'a> {
  let snippets    : Snippets    = parse_snippets(token_iter);
  let connections : Connections = parse_connections(token_iter);
  return Prog::Prog(snippets, connections);
}

pub fn parse_snippets<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Snippets<'a> {
  // Internal helper function to check if it's a snippet or not
  fn is_snippet(token : Option<&& Token>) -> bool {
      match token {
        Some(&& Token::Snippet)=> true,
        _                     => false,
      }
  }

  let mut snippet_vector = Vec::<Snippet>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_snippet(token_iter.peek())) {
      return Snippets::Snippets(snippet_vector);
    } else {
      let snippet : Snippet = parse_snippet(token_iter);
      snippet_vector.push(snippet);
    }
  }
}
  
pub fn parse_snippet<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Snippet<'a> {
  match_token(token_iter, Token::Snippet, "Snippet definition must start with the keyword snippet.");
  let identifier : Identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenLeft, "Snippet argument list must start with a left parenthesis.");
  let id_list    : IdList     = parse_idlist(token_iter);
  match_token(token_iter, Token::ParenRight, "Snippet argument list must end with a right parenthesis.");
  match_token(token_iter, Token::BraceLeft, "Snippet body must begin with a left brace.");
  let inits      : Initializers = parse_initializers(token_iter);
  let stmts      : Statements = parse_statements(token_iter);
  match_token(token_iter, Token::BraceRight, "Snippet body must end with a right brace.");
  return Snippet::Snippet(identifier, id_list, inits, stmts);
}

pub fn parse_connections<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Connections<'a> {
  if !token_iter.peek().is_some() {
    return Connections::Empty();
  } else {
    match_token(token_iter, Token::ParenLeft, "Connection must start with a left parenthesis.");
    let id1   : Identifier = parse_identifier(token_iter);
    match_token(token_iter, Token::Comma, "Need a comma between snippets that are being connected.");
    let id2   : Identifier = parse_identifier(token_iter);
    match_token(token_iter, Token::ParenRight, "Connection must end with a right parenthesis.");
    let conns : Connections= parse_connections(token_iter);
    return Connections::Connections(id1, id2, Box::new(conns));
  }
}

pub fn parse_idlist<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> IdList<'a> {
  // Helper function to detect identifiers
  fn is_ident(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Identifier(_)) => true,
      _                            => false,
    }
  }

  let mut id_vector = Vec::<Identifier>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_ident(token_iter.peek())) {
      return IdList::IdList(id_vector);
    } else {
      let identifier : Identifier = parse_identifier(token_iter);
      match_token(token_iter, Token::Comma, "Expected comma as separator between identifiers.");
      id_vector.push(identifier);
    }
  }
}

pub fn parse_initializers<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Initializers<'a> {
  // Helper function to determine if it's an initializer
  fn is_static(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Static) => true,
      _                     => false,
    }
  }

  let mut init_vector = Vec::<Initializer>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_static(token_iter.peek())) {
      return Initializers::Initializers(init_vector);
    } else {
      let initializer : Initializer  = parse_initializer(token_iter);
      init_vector.push(initializer);
    }
  }
}

pub fn parse_initializer<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Initializer<'a> {
  match_token(token_iter, Token::Static, "First token in an initializer must be the keyword static.");
  let identifier : Identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and value by an assignment symbol.");
  let value      : Value      = parse_value(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in an initializer must be a semicolon.");
  return Initializer::Initializer(identifier, value);
}

pub fn parse_statements<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Statements<'a> {
  // Helper function to identify beginning of statements
  fn is_ident(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Identifier(_)) => true,
      _                            => false,
    }
  }

  let mut stmt_vector = Vec::<Statement>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_ident(token_iter.peek())) {
      return Statements::Statements(stmt_vector);
    } else {
      let stmt : Statement  = parse_statement(token_iter);
      stmt_vector.push(stmt);
    }
  }
}

pub fn parse_statement<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Statement<'a> {
  let identifier : Identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and expression by an assignment symbol.");
  let expr      : Expr     = parse_expr(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in an initializer must be a semicolon.");
  return Statement::Statement(identifier, expr);
}

pub fn parse_expr<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Expr<'a> {
  if !token_iter.peek().is_some() {
    panic!("Insufficient tokens in call to parse_expr.");
  }
  let operand    = parse_operand(token_iter);
  let expr_right = parse_expr_right(token_iter);
  return Expr::Expr(operand, expr_right);
}

// Macro to generate parser for ExprRight given a list of binary operations
macro_rules! expr_right_parser {
  ($($x:ident),*) => {
    // generate enum of binary operation types
    // this must be public (I think)
    #[derive(Debug)]
    pub enum BinOpType {
      $($x,)*
    }

    pub fn parse_expr_right<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> ExprRight<'a> {
      // generate is_operator helper function
      fn is_operator(token : Option<&& Token>) -> bool {
        match token {
          $(Some(&& Token::$x)|)*
          Some(&& Token::Cond) => true,
          _                   => false,
        }
      }

      // generate get_bin_op helper function
      fn get_bin_op(t : & Token) -> BinOpType {
        match *t {
          $(Token::$x=>BinOpType::$x,)*
          _ => panic!("Invalid BinOpType")
        }
      }

      // use it in parse implementation
      if !token_iter.peek().is_some() || (!is_operator(token_iter.peek())) {
        return ExprRight::Empty();
      }
      let op_type = token_iter.next().unwrap();
      return match op_type {
        $(e @ & Token::$x       => { let operand   = parse_operand(token_iter); // Must be an operand
                                 ExprRight::BinOp(get_bin_op(e), operand)},)*
        & Token::Cond         => { let operand_true = parse_operand(token_iter); // Must be an operand
                                 match_token(token_iter, Token::Colon, "Colon must separate conditional halves.");
                                 let operand_false = parse_operand(token_iter);
                                 ExprRight::Cond(operand_true, operand_false)},
        _                   => { assert!(false, "Cannot get here!"); ExprRight::Empty()}
      }
    }
  };
}

// generate parser using macro
expr_right_parser!(BooleanAnd, BooleanOr, Plus, Minus, Mul, Div, Modulo, Equal, NotEqual, LTEQOp, GTEQOp, LessThan, GreaterThan);

pub fn parse_identifier<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Identifier<'a> {
  let identifier_token = token_iter.next().unwrap();
  match identifier_token {
    & Token::Identifier(i) => Identifier::Identifier(i),
    _                      => panic!("Invalid token: {:?}, expected Token::Identifier", identifier_token)
  }
}

pub fn parse_operand<'a>(token_iter : &mut Peekable<std::slice::Iter<'a, Token>>) -> Operand<'a> {
  let operand_token = token_iter.next().unwrap();
  match operand_token {
    & Token::Identifier(i) => return Operand::Identifier(Identifier::Identifier(i)),
    & Token::Value(v)      => return Operand::Value(Value::Value(v)),
    _                      => panic!("Invalid token: {:?}, expected Token::Identifier or Token::Value", operand_token)
  }
}

pub fn parse_value(token_iter : & mut Peekable<std::slice::Iter<Token>>) -> Value {
  let value_token = token_iter.next().unwrap();
  match value_token {
    & Token::Value(ref v)  => return Value::Value(v.clone()),
    _                      => panic!("Invalid token: {:?}, expected Token::Value", value_token)
 }
}
