use super::parser_impl::Parsing;
use super::lexer::get_tokens;
use super::parser::Operand;
use super::parser::Expr;
use super::parser::Statement;
use super::parser::Statements;
use super::parser::Initializers;
use super::parser::Snippet;
use super::parser::Snippets;
use super::parser::Connections;
use super::parser::Prog;

#[test]
fn test_parse_operand() {
  let input =  r"5";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Operand::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_expr() {
  let input = r"7%5-5+6";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Expr::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_stmt() {
  let input = r"x=6+5+4;";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Statement::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_stmts() {
  let input = r"x=6+5+4;y=7*8%5;";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Statements::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_inits() {
  let input = r"static x=6;static y=7;";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Initializers::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_snippet1() {
  let input = r"snippet fun(a, b, c,) { static x=6;static y=7;}";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Snippet::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_snippet2() {
  let input = r"snippet fun(a, b, c,) { static x=6;x=y+5;}";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Snippet::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_snippets() {
  let input = r"snippet fun(a, b, c,) { static x=6;x=y+5;} snippet fun(a, b, c,) { static x=6;x=y+5;}";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Snippets::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_connections() {
  let input = r"(foo, fun) (bar, foobar)";
  let tokens = & mut get_tokens(input);
  println!("{:?}", Connections::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}

#[test]
fn test_parse_prog() {
  let input_program = r"snippet fun(a, b, c, x, y, ) {
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
  let tokens = & mut get_tokens(input_program);
  println!("{:?}", Prog::parse(tokens));
  assert!(tokens.is_empty(), "tokens is not empty");
}
