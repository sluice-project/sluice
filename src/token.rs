#[derive(Debug)]
#[derive(PartialEq)]
pub enum Token<'a> {
  // Variants that take an argument
  Identifier(&'a str),
  Value(u32),

  // Keywords: static, snippet, and, or, not
  Static,
  Snippet,
  BooleanAnd,
  BooleanOr,
  BooleanNot, // XXX: Not used in language yet

  // Separators
  Colon,
  SemiColon,
  Comma,
  Arrow,

  // Grouping operators
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
