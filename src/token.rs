#[derive(Debug)]
pub enum Token {
  // Variants that take an argument
  Identifier(String),
  Values(String),

  // Keywords: static, snippet, and, or, not
  Static,
  Snippet,
  BooleanAnd,
  BooleanOr,
  BooleanNot,

  // Separators
  Colon,
  SemiColon,
  Period,
  Comma,

  // Grouping operators
  SqBktLeft,
  SqBktRight,
  ParenLeft,
  ParenRight,
  BraceLeft,
  BraceRight,

  // Binary arithmetic operators + conditional operator
  Plus,
  Minus,
  Mul,
  Div,
  Cond,
  Modulo,

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
