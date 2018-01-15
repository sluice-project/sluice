extern crate regex;
use self::regex::Regex;

lazy_static! {
  static ref TOKENS      : Regex = Regex::new(r"[0-9]+|[A-Za-z_][A-Za-z0-9_]*|&&|\|\||!|&|\||\^|>>|<<|==|!=|>=|<=|>|<|\+|-|/|\*|%|\{|\}|\(|\)|\[|\]|=|;|\.|,|\?|:|\S+").unwrap();
  static ref KEYWORDS    : Regex = Regex::new(r"^(static|snippet)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
}

enum Token {
  // Variants that take an argument
  Identifier(String),
  Values(String),

  // Keywords: static and snippet
  Static,
  Snippet,

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

  // Boolean operators
  And,
  Or,
  Not,
  Xor,

  // Comparison operators
  Equal,
  NotEqual,
  LessThan,
  GreaterThan,
  LTEQOp,
  GTEQOp,

  // Assignment
  Assign,

  // Bit-wise operators
  BitWiseOr,
  BitWiseAnd,
  BitWiseXor,
  LeftShift,
  RightShift
}

pub fn get_tokens(input_program : &str) {
  // Split string into tokens at whitespaces
  // TODO: Fix this to remove this assumption of tokens being separated by whitespaces.
  for cap in TOKENS.captures_iter(input_program) {
    let ref token = cap[0];
    if KEYWORDS.is_match(token) {
      println!("Found a keyword {}", token);
    } else if IDENTIFIERS.is_match(token) {
      println!("Found an identifier {}", token);
    } else if VALUES.is_match(token) {
      println!("Found a value {}", token);
    } else {
      println!("Found operator token {}", token);
    }
  }
}

#[test]
fn test_lexer_with_spaces() {
  let input_program = r"snippet fun ( a , b , c , x , y ) {
                          static x = 0 ;
                          if ( a >= b ) {
                            a = x ;
                            b = y ;
                          } elif ( c >= d ) {
                            m == 5 ;
                          }
                        }";
  get_tokens(input_program);
}

#[test]
fn test_lexer_wo_spaces() {
  let input_program = r"snippet fun(a, b, c, x, y) {
                          static x = 0;
                          if (a >= b) {
                            a = x;
                            b = y;
                          } elif (c >= d) {
                            m == 5;
                          }
                        }";
  get_tokens(input_program);
}
