// Trait for parsing that will be implemented by each
// non-terminal in the grammar. Each implementation of
// this trait can be thought of as a parser combinator.

use super::parser::*;
use super::token::Token;

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
    // Internal helper function to check if it's a snippet or not
    fn is_snippet(token : Option<& Token>) -> bool {
        match token {
          Some(& Token::Snippet)=> true,
          _                     => false,
        }
    }

    let mut snippet_vector = Vec::<Snippet>::new();
    loop {
      if token_vector.is_empty() || (!is_snippet(token_vector.first())) {
        return Snippets::Snippets(snippet_vector);
      } else {
        let snippet : Snippet = Parsing::parse(token_vector);
        snippet_vector.push(snippet);
      }
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
      match_token(token_vector, Token::ParenLeft, "Connection must start with a left parenthesis.");
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
    // Helper function to detect identifiers
    fn is_ident(token : Option<& Token>) -> bool {
      match token {
        Some(& Token::Identifier(_)) => true,
        _                            => false,
      }
    }

    let mut id_vector = Vec::<Identifier>::new();
    loop {
      if token_vector.is_empty() || (!is_ident(token_vector.first())) {
        return IdList::IdList(id_vector);
      } else {
        let identifier : Identifier = Parsing::parse(token_vector);
        match_token(token_vector, Token::Comma, "Expected comma as separator between identifiers.");
        id_vector.push(identifier);
      }
    }
  }
}

impl Parsing for Initializers {
  fn parse(token_vector : & mut Vec<Token>) -> Initializers {
    // Helper function to determine if it's an initializer
    fn is_static(token : Option<& Token>) -> bool {
      match token {
        Some(& Token::Static) => true,
        _                     => false,
      }
    }

    let mut init_vector = Vec::<Initializer>::new();
    loop {
      if token_vector.is_empty() || (!is_static(token_vector.first())) {
        return Initializers::Initializers(init_vector);
      } else {
        let initializer : Initializer  = Parsing::parse(token_vector);
        init_vector.push(initializer);
      }
    }
  }
}

impl Parsing for Initializer {
  fn parse(token_vector : & mut Vec<Token>) -> Initializer {
    match_token(token_vector, Token::Static, "First token in an initializer must be the keyword static.");
    let identifier : Identifier = Parsing::parse(token_vector);
    match_token(token_vector, Token::Assign, "Must separate identifier and value by an assignment symbol.");
    let value      : Value      = Parsing::parse(token_vector);
    match_token(token_vector, Token::SemiColon, "Last token in an initializer must be a semicolon.");
    return Initializer::Initializer(identifier, value);
  }
}

impl Parsing for Statements {
  fn parse(token_vector : & mut Vec<Token>) -> Statements {
    // Helper function to identify beginning of statements
    fn is_ident(token : Option<& Token>) -> bool {
      match token {
        Some(& Token::Identifier(_)) => true,
        _                            => false,
      }
    }

    let mut stmt_vector = Vec::<Statement>::new();
    loop {
      if token_vector.is_empty() || (!is_ident(token_vector.first())) {
        return Statements::Statements(stmt_vector);
      } else {
        let stmt : Statement  = Parsing::parse(token_vector);
        stmt_vector.push(stmt);
      }
    }
  }
}

impl Parsing for Statement {
  fn parse(token_vector : & mut Vec<Token>) -> Statement {
    let identifier : Identifier = Parsing::parse(token_vector);
    match_token(token_vector, Token::Assign, "Must separate identifier and expression by an assignment symbol.");
    let expr      : Expr     = Parsing::parse(token_vector);
    match_token(token_vector, Token::SemiColon, "Last token in an initializer must be a semicolon.");
    return Statement::Statement(identifier, expr);
  }
}

impl Parsing for Expr {
  fn parse(token_vector : & mut Vec<Token>) -> Expr {
    if token_vector.is_empty() {
      panic!("Insufficient tokens in call to parse_expr.");
    }
    let operand    = Parsing::parse(token_vector);
    let expr_right = Parsing::parse(token_vector);
    return Expr::Expr(operand, expr_right);
  }
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

    impl Parsing for ExprRight {
      fn parse(token_vector : & mut Vec<Token>) -> ExprRight {
        // generate is_operator helper function
        fn is_operator(token : Option<& Token>) -> bool {
          match token {
            $(Some(& Token::$x)|)*
            Some(& Token::Cond) => true,
            _                   => false,
          }
        }

        // generate get_bin_op helper function
        fn get_bin_op(t : Token) -> BinOpType {
          match t {
            $(Token::$x=>BinOpType::$x,)*
            _ => panic!("Invalid BinOpType")
          }
        }

        // use it in parse implementation
        if token_vector.is_empty() || (!is_operator(token_vector.first())) {
          return ExprRight::Empty();
        }
        let op_type = token_vector.remove(0);
        return match op_type {
          $(e @ Token::$x     => { let operand   = Parsing::parse(token_vector); // Must be an operand
                                   ExprRight::BinOp(get_bin_op(e), operand)},)*
          Token::Cond         => { let operand_true = Parsing::parse(token_vector); // Must be an operand
                                   match_token(token_vector, Token::Colon, "Colon must separate conditional halves.");
                                   let operand_false = Parsing::parse(token_vector);
                                   ExprRight::Cond(operand_true, operand_false)},
          _                   => { assert!(false, "Cannot get here!"); ExprRight::Empty()}
        }
      }
    }
  };
}

// generate parser using macro
expr_right_parser!(BooleanAnd, BooleanOr, Plus, Minus, Mul, Div, Modulo, Equal, NotEqual, LTEQOp, GTEQOp, LessThan, GreaterThan);

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
      Token::Identifier(i) => return Operand::Identifier(Identifier::Identifier(i)),
      Token::Value(v)      => return Operand::Value(Value::Value(v)),
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
