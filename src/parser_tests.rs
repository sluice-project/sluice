use super::parser_impl::Parsing;
use super::lexer::get_tokens;
use super::parser::Operand;
use super::parser::Expr;

#[test]
fn test_parse_operand() {
  let input =  r"5";
  println!("{:?}", Operand::parse(get_tokens(input)));
}

#[test]
fn test_parse_expr() {
  let input = r"7%5-5+6";
  println!("{:?}", Expr::parse(get_tokens(input)));
}
