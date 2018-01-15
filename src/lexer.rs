extern crate regex;
use self::regex::Regex;

lazy_static! {
  static ref TOKENS      : Regex = Regex::new(r"[0-9]+|[A-Za-z_][A-Za-z0-9_]*|&&|\|\||!|&|\||\^|>>|<<|==|!=|>=|<=|>|<|\+|-|/|\*|%|\{|\}|\(|\)|\[|\]|=|;|\.|,|\?|:|\S+").unwrap();
  static ref KEYWORDS    : Regex = Regex::new(r"^(static|snippet)$").unwrap();
  static ref IDENTIFIERS : Regex = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
  static ref VALUES      : Regex = Regex::new(r"^([0-9]+)$").unwrap();
}

pub fn run_lexer(input_program : &str) {
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
  run_lexer(input_program);
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
  run_lexer(input_program);
}
