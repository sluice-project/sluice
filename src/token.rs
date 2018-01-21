#[derive(Debug)]
#[derive(PartialEq)]
pub enum Token {
  // Variants that take an argument
  Identifier(String),
  Value(String),

  // Keywords: static, snippet, and, or, not
  Static,
  Snippet,
  BooleanAnd,
  BooleanOr,
  BooleanNot, // XXX: Not used in language yet

  // Separators
  Colon,
  SemiColon,
  Period,     // XXX: Not used in language yet
  Comma,

  // Grouping operators
  SqBktLeft,  // XXX: Not used in language yet
  SqBktRight, // XXX: Not used in language yet
  ParenLeft,
  ParenRight,
  BraceLeft,
  BraceRight,

  // Binary arithmetic operators + conditional operator
  Plus,
  Minus,
  Mul,
  Div,
  Modulo,
  Cond,

  // Comparison operators
  Equal,
  NotEqual,
  LTEQOp,
  GTEQOp,
  LessThan,
  GreaterThan,

  // Assignment
  Assign,
}

pub fn is_ident(token : Option<& Token>) -> bool {
  match token {
    Some(& Token::Identifier(_)) => true,
    _                            => false,
  }
}

pub fn is_operator(token : Option<& Token>) -> bool {
  match token {
    Some(& Token::BooleanAnd) |
    Some(& Token::BooleanOr)  |
    Some(& Token::BooleanNot) |
    Some(& Token::Plus)       |
    Some(& Token::Minus)      |
    Some(& Token::Mul)        |
    Some(& Token::Div)        |
    Some(& Token::Modulo)     |
    Some(& Token::Equal)      |
    Some(& Token::NotEqual)   |
    Some(& Token::LTEQOp)     |
    Some(& Token::GTEQOp)     |
    Some(& Token::LessThan)   |
    Some(& Token::GreaterThan)|
    Some(& Token::Cond)          => true,
    _                            => false,
  }
}

pub fn is_static(token : Option<& Token>) -> bool {
  match token {
    Some(& Token::Static) => true,
    _                     => false,
  }
}

pub fn is_snippet(token : Option<& Token>) -> bool {
  match token {
    Some(& Token::Snippet)=> true,
    _                     => false,
  }
}
