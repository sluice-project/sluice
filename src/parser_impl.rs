// Trait for parsing that will be implemented by each
// non-terminal in the grammar. Each implementation of
// this trait can be thought of as a parser combinator.

use super::parser::*;
use super::token::Token;
use super::token::is_ident;
use super::token::is_bin_op;
use super::token::is_static;
use super::token::is_snippet;

// Helper function to consume next token and match it against a specified token
// Throw an error if either:
// 1. token_vector is empty
// 2. the next token does not match
fn match_token(token_vector : & mut Vec<Token>, expected : Token, error_msg : &'static str) {
  if token_vector.is_empty() {
    panic!("token_vector is empty. Can't consume next token");
  } else {
    let next_token = token_vector.remove(0);
    if next_token == expected {
      return;
    } else {
      panic!("\nInvalid token: {:?}, expected {:?}.\nError message: {:?}", next_token, expected, error_msg);
    }
  }
}

pub trait Parsing where Self: Sized {
  fn parse(token_vector : & mut Vec<Token>) -> Self;
}

impl Parsing for Prog {
 fn parse(token_vector : & mut Vec<Token>) -> Prog {
   let snippets    : Snippets    = Parsing::parse(token_vector);
   let connections : Connections = Parsing::parse(token_vector);
   return Prog::Prog(snippets, connections);
 }
}

impl Parsing for Snippets {
  fn parse(token_vector : & mut Vec<Token>) -> Snippets {
    if token_vector.is_empty() || (!is_snippet(token_vector.first())) {
      return Snippets::Empty();
    } else {
      let snippet : Snippet = Parsing::parse(token_vector);
      let snippets: Snippets = Parsing::parse(token_vector);
      return Snippets::Snippets(snippet, Box::new(snippets));
    }
  }
}

impl Parsing for Snippet {
  fn parse(token_vector : & mut Vec<Token>) -> Snippet {
    match_token(token_vector, Token::Snippet, "Snippet definition must start with the keyword snippet.");
    let identifier : Identifier = Parsing::parse(token_vector);
    match_token(token_vector, Token::ParenLeft, "Snippet argument list must start with a left parenthesis.");
    let id_list    : IdList     = Parsing::parse(token_vector);
    match_token(token_vector, Token::ParenRight, "Snippet argument list must end with a right parenthesis.");
    match_token(token_vector, Token::BraceLeft, "Snippet body must begin with a left brace.");
    let inits      : Initializers = Parsing::parse(token_vector);
    let stmts      : Statements = Parsing::parse(token_vector);
    match_token(token_vector, Token::BraceRight, "Snippet body must end with a right brace.");
    return Snippet::Snippet(identifier, id_list, inits, stmts);
  }
}

impl Parsing for Connections {
  fn parse(token_vector : & mut Vec<Token>) -> Connections {
    if token_vector.is_empty() {
      return Connections::Empty();
    } else {
      match_token(token_vector, Token::ParenLeft, "Connection must start with a left parenthesis");
      let id1   : Identifier = Parsing::parse(token_vector);
      match_token(token_vector, Token::Comma, "Need a comma between snippets that are being connected.");
      let id2   : Identifier = Parsing::parse(token_vector);
      match_token(token_vector, Token::ParenRight, "Connection must end with a right parenthesis.");
      let conns : Connections= Parsing::parse(token_vector);
      return Connections::Connections(id1, id2, Box::new(conns));
    }
  }
}

impl Parsing for IdList {
  fn parse(token_vector : & mut Vec<Token>) -> IdList {
    if token_vector.is_empty() || (!is_ident(token_vector.first())) {
      return IdList::Empty();
    } else {
      let identifier : Identifier = Parsing::parse(token_vector);
      match_token(token_vector, Token::Comma, "Expected comma as separator between identifiers.");
      let idlist     : IdList     = Parsing::parse(token_vector);
      return IdList::IdList(identifier, Box::new(idlist));
    }
  }
}

impl Parsing for Initializers {
  fn parse(token_vector : & mut Vec<Token>) -> Initializers {
    if token_vector.is_empty() || (!is_static(token_vector.first())) {
      return Initializers::Empty();
    } else {
      let initializer : Initializer  = Parsing::parse(token_vector);
      let initializers: Initializers = Parsing::parse(token_vector);
      return Initializers::Initializers(initializer, Box::new(initializers));
    }
  }
}

impl Parsing for Initializer {
  fn parse(token_vector : & mut Vec<Token>) -> Initializer {
    match_token(token_vector, Token::Static, "First token in an initializer must be the keyword static");
    let identifier : Identifier = Parsing::parse(token_vector);
    match_token(token_vector, Token::Assign, "Must separate identifier and value by an assignment symbol");
    let value      : Value      = Parsing::parse(token_vector);
    match_token(token_vector, Token::SemiColon, "Last token in an initializer must be a semicolon");
    return Initializer::Initializer(identifier, value);
  }
}

impl Parsing for Statements {
  fn parse(token_vector : & mut Vec<Token>) -> Statements {
    if token_vector.is_empty() || (! is_ident(token_vector.first())) {
      return Statements::Empty();
    } else {
      let stmt : Statement  = Parsing::parse(token_vector);
      let stmts: Statements = Parsing::parse(token_vector);
      return Statements::Statements(stmt, Box::new(stmts));
    }
  }
}

impl Parsing for Statement {
  fn parse(token_vector : & mut Vec<Token>) -> Statement {
    let identifier : Identifier = Parsing::parse(token_vector);
    match_token(token_vector, Token::Assign, "Must separate identifier and expression by an assignment symbol");
    let expr      : Expr     = Parsing::parse(token_vector);
    match_token(token_vector, Token::SemiColon, "Last token in an initializer must be a semicolon");
    return Statement::Statement(identifier, expr);
  }
}

impl Parsing for Expr {
  fn parse(token_vector : & mut Vec<Token>) -> Expr {
    if token_vector.is_empty() {
      panic!("Insufficient tokens in call to parse_expr");
    }
    let operand    = token_vector.remove(0);
    let expr_right = token_vector;
    return Expr::Expr(Parsing::parse(&mut vec!(operand)),
                      Parsing::parse(expr_right));
  }
}

impl Parsing for ExprRight {
  fn parse(token_vector : & mut Vec<Token>) -> ExprRight {
    if token_vector.is_empty() || (! is_bin_op(token_vector.first())) {
      return ExprRight::Empty();
    }
    let op_type = token_vector.remove(0);
    return match op_type {
      e @ Token::Plus  |
      e @ Token::Minus |
      e @ Token::Mul   |
      e @ Token::Div   |
      e @ Token::Modulo => { let operand = token_vector.remove(0); // This has to be an operand or it's an error
                             ExprRight::BinOp(get_bin_op(e),
                                              Parsing::parse(&mut vec!(operand)),
                                              Box::new(Parsing::parse(token_vector))) },
      _          => { assert!(false, "Cannot get here!"); ExprRight::Empty()}
    }
  }
}

impl Parsing for Identifier {
  fn parse(token_vector : & mut Vec<Token>) -> Identifier {
    let identifier_token = token_vector.remove(0);
    match identifier_token {
      Token::Identifier(s) => Identifier::Identifier(s),
      _                    => panic!("Invalid token: {:?}, expected Token::Identifier", identifier_token)
    }
  }
}

impl Parsing for Operand {
  fn parse(token_vector : & mut Vec<Token>) -> Operand {
    let operand_token = token_vector.remove(0);
    match operand_token {
      Token::Identifier(i) => return Operand::Identifier(i),
      Token::Value(v)      => return Operand::Value(v),
      _                    => panic!("Invalid token: {:?}, expected Token::Identifier or Token::Value", operand_token)
    }
  }
}

impl Parsing for Value {
  fn parse(token_vector : & mut Vec<Token>) -> Value {
    let value_token = token_vector.remove(0);
    match value_token {
     Token::Value(v)  => return Value::Value(v),
     _                => panic!("Invalid token: {:?}, expected Token::Value", value_token)
   }
  }
}
