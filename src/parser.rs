// Trait for parsing that will be implemented by each
// non-terminal in the grammar. Each implementation of
// this trait can be thought of as a parser combinator.

use std;
use super::grammar::*;
use super::token::Token;
use std::iter::Peekable;

type TokenIterator<'a> = Peekable<std::slice::Iter<'a, Token<'a>>>;

// Helper function to consume next token and match it against a specified token
// Throw an error if either:
// 1. token_iter is empty
// 2. the next token does not match
fn match_token<'a>(token_iter : & mut TokenIterator<'a>, expected : Token<'a>, error_msg : &'static str) {
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

pub fn parse_prog<'a>(token_iter : &mut TokenIterator<'a>) -> Prog<'a> {
  let snippets    = parse_snippets(token_iter);
  let connections = parse_connections(token_iter);
  return Prog::Prog(snippets, connections);
}

fn parse_snippets<'a>(token_iter : &mut TokenIterator<'a>) -> Snippets<'a> {
  // Internal helper function to check if it's a snippet or not
  fn is_snippet(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Snippet)=> true,
      _                     => false,
    }
  }

  let mut snippet_vector = Vec::<Snippet>::new();
  loop {
    if !token_iter.peek().is_some() || !is_snippet(token_iter.peek()) {
      return Snippets::Snippets(snippet_vector);
    } else {
      let snippet = parse_snippet(token_iter);
      snippet_vector.push(snippet);
    }
  }
}
  
fn parse_snippet<'a>(token_iter : &mut TokenIterator<'a>) -> Snippet<'a> {
  match_token(token_iter, Token::Snippet, "Snippet definition must start with the keyword snippet.");
  let identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenLeft, "Snippet argument list must start with a left parenthesis.");
  let id_list    = parse_idlist(token_iter);
  match_token(token_iter, Token::ParenRight, "Snippet argument list must end with a right parenthesis.");
  match_token(token_iter, Token::BraceLeft, "Snippet body must begin with a left brace.");
  let initializers      = parse_initializers(token_iter);
  let statements      = parse_statements(token_iter);
  match_token(token_iter, Token::BraceRight, "Snippet body must end with a right brace.");
  return Snippet::Snippet(identifier, id_list, initializers, statements);
}

fn parse_connections<'a>(token_iter : &mut TokenIterator<'a>) -> Connections<'a> {
  let mut connection_vector = Vec::<Connection<'a>>::new();
  loop {
    if !token_iter.peek().is_some() {
      return Connections::Connections(connection_vector);
    } else {
      connection_vector.push(parse_connection(token_iter));
    }
  }
}

fn parse_connection<'a>(token_iter : &mut TokenIterator<'a>) -> Connection<'a> {
  // Helper function to detect identifiers
  fn is_ident(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Identifier(_)) => true,
      _                            => false,
    }
  }

  // Internal helper function to check if it's a colon or not
  fn is_colon(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Colon)=> true,
      _                     => false,
    }
  }


  match_token(token_iter, Token::ParenLeft, "Connection must start with a left parenthesis.");
  let id1   = parse_identifier(token_iter);
  match_token(token_iter, Token::Comma, "Need a comma between snippets that are being connected.");
  let id2   = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenRight, "Connection must end with a right parenthesis.");
  let mut variable_pairs = Vec::<VariablePair>::new();
  if token_iter.peek().is_some() && is_colon(token_iter.peek()) {
    match_token(token_iter, Token::Colon, "Need a colon before variable pairings");
    loop {
      if !token_iter.peek().is_some() || !is_ident(token_iter.peek()) { break; }
      let from_variable = parse_identifier(token_iter);
      match_token(token_iter, Token::Arrow, "Need an arrow between variables.");
      let to_variable   = parse_identifier(token_iter);
      match_token(token_iter, Token::Comma, "Need a comma separating variable connections.");
      variable_pairs.push(VariablePair { from_var : from_variable, to_var : to_variable });
    }
  }
  return Connection { from_function : id1, to_function : id2, variable_pairs : variable_pairs };
}

fn parse_idlist<'a>(token_iter : &mut TokenIterator<'a>) -> IdList<'a> {
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
      let identifier = parse_identifier(token_iter);
      match_token(token_iter, Token::Comma, "Expected comma as separator between identifiers.");
      id_vector.push(identifier);
    }
  }
}

fn parse_initializers<'a>(token_iter : &mut TokenIterator<'a>) -> Initializers<'a> {
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
      let initializer = parse_initializer(token_iter);
      init_vector.push(initializer);
    }
  }
}

fn parse_initializer<'a>(token_iter : &mut TokenIterator<'a>) -> Initializer<'a> {
  match_token(token_iter, Token::Static, "First token in an initializer must be the keyword static.");
  let identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and value by an assignment symbol.");
  let value      = parse_value(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in an initializer must be a semicolon.");
  return Initializer::Initializer(identifier, value);
}

fn parse_statements<'a>(token_iter : &mut TokenIterator<'a>) -> Statements<'a> {
  // Helper function to identify beginning of statements
  fn is_ident(token : Option<&& Token>) -> bool {
    match token {
      Some(&& Token::Identifier(_)) => true,
      _                            => false,
    }
  }

  let mut statement_vector = Vec::<Statement>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_ident(token_iter.peek())) {
      return Statements::Statements(statement_vector);
    } else {
      let statement = parse_statement(token_iter);
      statement_vector.push(statement);
    }
  }
}

fn parse_statement<'a>(token_iter : &mut TokenIterator<'a>) -> Statement<'a> {
  let identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and expression by an assignment symbol.");
  let expr       = parse_expr(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in an initializer must be a semicolon.");
  return Statement::Statement(identifier, expr);
}

fn parse_expr<'a>(token_iter : &mut TokenIterator<'a>) -> Expr<'a> {
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

    fn parse_expr_right<'a>(token_iter : &mut TokenIterator<'a>) -> ExprRight<'a> {
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

fn parse_identifier<'a>(token_iter : &mut TokenIterator<'a>) -> Identifier<'a> {
  let identifier_token = token_iter.next().unwrap();
  match identifier_token {
    & Token::Identifier(i) => Identifier::Identifier(i),
    _                      => panic!("Invalid token: {:?}, expected Token::Identifier", identifier_token)
  }
}

fn parse_operand<'a>(token_iter : &mut TokenIterator<'a>) -> Operand<'a> {
  let operand_token = token_iter.next().unwrap();
  match operand_token {
    & Token::Identifier(i) => return Operand::Identifier(Identifier::Identifier(i)),
    & Token::Value(v)      => return Operand::Value(Value::Value(v)),
    _                      => panic!("Invalid token: {:?}, expected Token::Identifier or Token::Value", operand_token)
  }
}

fn parse_value<'a>(token_iter : & mut TokenIterator<'a>) -> Value {
  let value_token = token_iter.next().unwrap();
  match value_token {
    & Token::Value(ref v)  => return Value::Value(v.clone()),
    _                      => panic!("Invalid token: {:?}, expected Token::Value", value_token)
 }
}

#[cfg(test)]
mod tests{
  use super::*;
  use super::super::lexer::get_tokens;
  
  #[test]
  fn test_parse_operand() {
    let input =  r"5";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_operand(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_expr() {
    let input = r"7%5";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_expr(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_statement() {
    let input = r"x=6+5;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_statement(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_statements() {
    let input = r"x=6+5;y=7*8;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_statements(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_initializers() {
    let input = r"static x=6;static y=7;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_initializers(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippet1() {
    let input = r"snippet fun(a, b, c,) { static x=6;static y=7;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_snippet(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippet2() {
    let input = r"snippet fun(a, b, c,) { static x=6;x=y+5;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_snippet(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippets() {
    let input = r"snippet fun(a, b, c,) { static x=6;x=y+5;} snippet fun(a, b, c,) { static x=6;x=y+5;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_snippets(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_connections() {
    let input = r"(foo, fun) (bar, foobar)";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_connections(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_connections2() {
    let input = r"(foo, fun): a->b, c->x, (bar, foobar)";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_connections(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_prog() {
    let input        = r"snippet fun(a, b, c, x, y, ) {
                            static x = 0;
                            a = x;
                            b = y;
                            m = 5;
                          }
                          snippet foo(a, b, c, ) {
                            static x = 1;
                            x = 5;
                          }
                          (foo, fun)
                          ";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_prog(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_prog2() {
    let input          = r"snippet fun ( a , b , c , x , y, ) {
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
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_prog(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
}
