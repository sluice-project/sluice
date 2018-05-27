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
  let is_snippet = |token| { match token { &Token::Snippet => true, _ => false, } };

  let mut snippet_vector = Vec::<Snippet>::new();
  loop {
    if !token_iter.peek().is_some() || !is_snippet(*token_iter.peek().unwrap()) {
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
  let persistent_decls    = parse_persistent_decls(token_iter);
  let transient_decls     = parse_transient_decls(token_iter);
  let statements      = parse_statements(token_iter);
  match_token(token_iter, Token::BraceRight, "Snippet body must end with a right brace.");
  return Snippet::Snippet(identifier, id_list, persistent_decls, transient_decls, statements);
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
  // Helper function to detect identifiers and colons
  let is_ident = |token| { match token { &Token::Identifier(_) => true, _ => false, } };
  let is_colon = |token| { match token { &Token::Colon => true, _ => false, } };

  match_token(token_iter, Token::ParenLeft, "Connection must start with a left parenthesis.");
  let id1   = parse_identifier(token_iter);
  match_token(token_iter, Token::Comma, "Need a comma between snippets that are being connected.");
  let id2   = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenRight, "Connection must end with a right parenthesis.");
  let mut variable_pairs = Vec::<VariablePair>::new();
  if token_iter.peek().is_some() && is_colon(*token_iter.peek().unwrap()) {
    match_token(token_iter, Token::Colon, "Need a colon before variable pairings");
    loop {
      if !token_iter.peek().is_some() || !is_ident(*token_iter.peek().unwrap()) { break; }
      let from_variable = parse_identifier(token_iter);
      match_token(token_iter, Token::Arrow, "Need an arrow between variables.");
      let to_variable   = parse_identifier(token_iter);
      match_token(token_iter, Token::Comma, "Need a comma separating variable connections.");
      variable_pairs.push(VariablePair { from_var : from_variable, to_var : to_variable });
    }
  }
  return Connection { from_snippet : id1, to_snippet : id2, variable_pairs : variable_pairs };
}

fn parse_idlist<'a>(token_iter : &mut TokenIterator<'a>) -> IdList<'a> {
  // Helper function to detect identifiers
  let is_ident = |token| { match token { &Token::Identifier(_) => true, _ => false, } };

  let mut id_vector = Vec::<Identifier>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_ident(*token_iter.peek().unwrap())) {
      return IdList::IdList(id_vector);
    } else {
      let identifier = parse_identifier(token_iter);
      match_token(token_iter, Token::Comma, "Expected comma as separator between identifiers.");
      id_vector.push(identifier);
    }
  }
}

fn parse_persistent_decls<'a>(token_iter : &mut TokenIterator<'a>) -> PersistentDecls<'a> {
  // Helper function to determine if it's an persistent_decl
  let is_persistent = |token| { match token { &Token::Persistent => true, _ => false, } };

  let mut init_vector = Vec::<PersistentDecl>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_persistent(*token_iter.peek().unwrap())) {
      return PersistentDecls::PersistentDecls(init_vector);
    } else {
      let persistent_decl = parse_persistent_decl(token_iter);
      init_vector.push(persistent_decl);
    }
  }
}

fn parse_persistent_decl<'a>(token_iter : &mut TokenIterator<'a>) -> PersistentDecl<'a> {
  match_token(token_iter, Token::Persistent, "First token in a persistent_decl must be the keyword persistent.");
  let identifier = parse_identifier(token_iter);
  let bit_width  = parse_type_annotation(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and value by an assignment symbol.");
  let value      = parse_initial_value(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in a persistent_decl must be a semicolon.");

  // Check that the initial values are representable using bit vector of bit_width
  match &value {
    &InitialValue::Value(Value::Value(ref init_value_u32)) => {
      if *init_value_u32 > 2_u32.pow(bit_width) - 1 {
        panic!("Initial value {} is outside the range [0, {}] of {}-bit vector.",
               init_value_u32,
               2_u32.pow(bit_width) - 1,
               bit_width);
      }
    },
    &InitialValue::ValueList(ValueList::ValueList(ref initial_values)) => {
      for value in initial_values {
        let Value::Value(init_value_u32) = value;
        if *init_value_u32 > 2_u32.pow(bit_width) - 1 {
          panic!("Initial value {} is outside the range [0, {}] of {}-bit vector.",
                 init_value_u32,
                 2_u32.pow(bit_width) - 1,
                 bit_width);
        }
      }
    }
  }
  return PersistentDecl { identifier : identifier, initial_value : value, bit_width : bit_width};
}

fn parse_transient_decls<'a>(token_iter : &mut TokenIterator<'a>) -> TransientDecls<'a> {
  // Helper function to determine if it's an transient_decl
  let is_transient = |token| { match token { &Token::Transient => true, _ => false, } };

  let mut init_vector = Vec::<TransientDecl>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_transient(*token_iter.peek().unwrap())) {
      return TransientDecls::TransientDecls(init_vector);
    } else {
      let transient_decl = parse_transient_decl(token_iter);
      init_vector.push(transient_decl);
    }
  }
}

fn parse_transient_decl<'a>(token_iter : &mut TokenIterator<'a>) -> TransientDecl<'a> {
  match_token(token_iter, Token::Transient, "First token in a transient_decl must be the keyword transient.");
  let identifier = parse_identifier(token_iter);
  let bit_width  = parse_type_annotation(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in a transient_decl must be a semicolon.");
  return TransientDecl { identifier : identifier, bit_width : bit_width };
}

// Retrieve bit width of bit vector. That's the only type for now.
fn parse_type_annotation<'a>(token_iter : &mut TokenIterator<'a>) -> u32 {
  match_token(token_iter, Token::Colon, "Type annotation must start with a colon.");
  match_token(token_iter, Token::Bit, "Invalid type, bit vectors are the only supported type.");
  match_token(token_iter, Token::LessThan, "Need angular brackets to specify width of bit vector.");
  let Value::Value(bit_width) = parse_value(token_iter);
  if bit_width > 30 {
    panic!("Bit width can be at most 30.");
  } else if bit_width < 1 {
    panic!("Bit width must be at least 1.");
  }
  match_token(token_iter, Token::GreaterThan, "Need angular brackets to specify width of bit vector.");
  return bit_width;
}

fn parse_statements<'a>(token_iter : &mut TokenIterator<'a>) -> Statements<'a> {
  // Helper function to identify beginning of statements
  let is_ident = |token| { match token { &Token::Identifier(_) => true, _ => false } };

  let mut statement_vector = Vec::<Statement>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_ident(*token_iter.peek().unwrap())) {
      return Statements::Statements(statement_vector);
    } else {
      let statement = parse_statement(token_iter);
      statement_vector.push(statement);
    }
  }
}

fn parse_statement<'a>(token_iter : &mut TokenIterator<'a>) -> Statement<'a> {
  let lvalue = parse_lvalue(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and expression by an assignment symbol.");
  let expr       = parse_expr(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in a statement must be a semicolon.");
  return Statement::Statement(lvalue, expr);
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
    fn parse_expr_right<'a>(token_iter : &mut TokenIterator<'a>) -> ExprRight<'a> {
      // generate is_operator helper function
      let is_operator = |token| { match token { $(&Token::$x|)* &Token::Cond => true, _ => false, } };

      // generate get_bin_op helper function
      fn get_bin_op(t : & Token) -> BinOpType {
        match *t {
          $(Token::$x=>BinOpType::$x,)*
          _ => panic!("Invalid BinOpType")
        }
      }

      // use it in parse implementation
      if !token_iter.peek().is_some() || (!is_operator(*token_iter.peek().unwrap())) {
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

fn parse_lvalue<'a>(token_iter : &mut TokenIterator<'a>) -> LValue<'a> {
  let lvalue_token = token_iter.next().unwrap();
  let is_square_left = |token| { match token { &Token::SquareLeft => true, _ => false, } };
  match lvalue_token {
    & Token::Identifier(i) => {
      if token_iter.peek().is_none() || !is_square_left(token_iter.peek().unwrap()) {
        return LValue::Identifier(Identifier::Identifier(i));
      } else {
        match_token(token_iter, Token::SquareLeft, "Expected [ here.");
        let array_address = parse_operand(token_iter);
        match_token(token_iter, Token::SquareRight, "Expected ] here.");
        return LValue::Array(Identifier::Identifier(i), Box::new(array_address));
      }
    }
    _                      => panic!("Invalid token: {:?}, expected Token::Identifier", lvalue_token)
  }
}

fn parse_operand<'a>(token_iter : &mut TokenIterator<'a>) -> Operand<'a> {
  match token_iter.peek().unwrap() { // && is required because we are using Peekable iterators
    && Token::Identifier(_) => return Operand::LValue(parse_lvalue(token_iter)),
    && Token::Value(_)      => return Operand::Value(parse_value(token_iter)),
    _                       => panic!("Invalid token: {:?}, expected Token::LValue or Token::Value", token_iter.peek().unwrap())
  }
}

fn parse_initial_value<'a>(token_iter : &mut TokenIterator<'a>) -> InitialValue {
  match token_iter.peek().unwrap() {
    && Token::Value(_)  => return InitialValue::Value(parse_value(token_iter)),
    && Token::BraceLeft => {
      match_token(token_iter, Token::BraceLeft, "PersistentDecl list must start with a left brace.");
      let value_list = parse_valuelist(token_iter);
      match_token(token_iter, Token::BraceRight, "PersistentDecl list must end with a right brace.");
      return InitialValue::ValueList(value_list);
    },
    _                   => panic!("Invalid token: {:?}, expected Token::Value or Token::BraceLeft", token_iter.peek().unwrap())
  }
}

fn parse_valuelist<'a>(token_iter : &mut TokenIterator<'a>) -> ValueList {
  // Helper function to detect values
  let is_value = |token| { match token { &Token::Value(_) => true, _ => false, } };

  let mut value_vector = Vec::<Value>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_value(*token_iter.peek().unwrap())) {
      return ValueList::ValueList(value_vector);
    } else {
      let value = parse_value(token_iter);
      match_token(token_iter, Token::Comma, "Expected comma as separator between values.");
      value_vector.push(value);
    }
  }
}

fn parse_value<'a>(token_iter : &mut TokenIterator<'a>) -> Value {
  let value_token = token_iter.next().unwrap();
  match value_token {
    & Token::Value(v)  => return Value::Value(v),
    _                  => panic!("Invalid token: {:?}, expected Token::Value", value_token)
 }
}

#[cfg(test)]
mod tests {
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
  fn test_parse_operand_id() {
    let input =  r"a";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_operand(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_operand_array() {
    let input =  r"a[5]";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_operand(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  #[should_panic(expected="Invalid token: Value(5), expected Token::Identifier")]
  fn test_parse_identifier_fail() {
    let input =  r"5";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_identifier(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_identifier_pass() {
    let input =  r"a";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_identifier(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_lvalue1() {
    let input =  r"a[5]";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_lvalue(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_lvalue2() {
    let input =  r"a";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_lvalue(token_iter));
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
  fn test_parse_transient_decls() {
    let input = r"transient x : bit<8>;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_transient_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_persistent_decls() {
    let input = r"persistent x : bit<3> = 6;persistent y : bit<3> =7;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_persistent_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_persistent_decls2() {
    let input = r"persistent x : bit<3> ={4, 5, 6, 7,};persistent y : bit<3> =7;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_persistent_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  #[should_panic(expected="Invalid token: BraceRight, expected Comma.\nError message: \"Expected comma as separator between values.\"")]
  fn test_parse_persistent_decls2_fail() {
    let input = r"persistent x : bit<3> ={4, 5, 6, 7};persistent y : bit<3> =7;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_persistent_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  #[should_panic(expected="Initial value 4 is outside the range [0, 3] of 2-bit vector.")]
  fn test_parse_persistent_decls_outside_range() {
    let input = r"persistent x : bit<2> = 4;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_persistent_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  #[should_panic(expected="Bit width must be at least 1.")]
  fn test_parse_persistent_decls_bitwidth0() {
    let input = r"persistent x : bit<0> = 4;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_persistent_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  #[should_panic(expected="Bit width can be at most 30.")]
  fn test_parse_persistent_decls_bitwidth31() {
    let input = r"persistent x : bit<31> = 4;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_persistent_decls(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_snippet1() {
    let input = r"snippet fun(a, b, c,) { persistent x : bit<3> =6;persistent y : bit<3> =7;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_snippet(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippet2() {
    let input = r"snippet fun(a, b, c,) { persistent x : bit<3> =6;x=y+5;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", parse_snippet(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippets() {
    let input = r"snippet fun(a, b, c,) { persistent x : bit<3> =6;x=y+5;} snippet fun(a, b, c,) { persistent x : bit<3> =6;x=y+5;}";
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
                            persistent x : bit<3> = 0;
                            a = x;
                            b = y;
                            m = 5;
                          }
                          snippet foo(a, b, c, ) {
                            persistent x : bit<3> = 1;
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
                            persistent x : bit<3> = 0 ;
                            transient k : bit<5>;
                            t1 = a >= b;
                            a = t1 ? x : a;
                            b = t1 ? y : b;
                            t2 = c >= d;
                            t3 = t2 and t1;
                            e = t2 ? m : 5;
                          }
                          snippet foo(a, b, c,) {
                            persistent x : bit<3> = 1;
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
