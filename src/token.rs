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
