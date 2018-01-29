#[cfg(test)]
mod tests{
  use super::super::parser_impl::Parsing;
  use super::super::lexer::get_tokens;
  use super::super::parser::Operand;
  use super::super::parser::Expr;
  use super::super::parser::Statement;
  use super::super::parser::Statements;
  use super::super::parser::Initializers;
  use super::super::parser::Snippet;
  use super::super::parser::Snippets;
  use super::super::parser::Connections;
  use super::super::parser::Prog;
  
  #[test]
  fn test_parse_operand() {
    let input =  r"5";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Operand::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_expr() {
    let input = r"7%5";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Expr::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_stmt() {
    let input = r"x=6+5;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Statement::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_stmts() {
    let input = r"x=6+5;y=7*8;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Statements::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_inits() {
    let input = r"static x=6;static y=7;";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Initializers::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippet1() {
    let input = r"snippet fun(a, b, c,) { static x=6;static y=7;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Snippet::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippet2() {
    let input = r"snippet fun(a, b, c,) { static x=6;x=y+5;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Snippet::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_snippets() {
    let input = r"snippet fun(a, b, c,) { static x=6;x=y+5;} snippet fun(a, b, c,) { static x=6;x=y+5;}";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Snippets::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_connections() {
    let input = r"(foo, fun) (bar, foobar)";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Connections::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
  
  #[test]
  fn test_parse_prog() {
    let input        = r"snippet fun(a, b, c, x, y, ) {
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
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Prog::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_prog2() {
    let input          = r"snippet fun ( a , b , c , x , y, ) {
                            static x = 0 ;
                            t1 = a >= b;
                            a = t1 ? x : a;
                            b = t1 ? y : b;
                            t2 = c >= d;
                            t3 = t2 and t1;
                            e = t2 ? m : 5;
                          }
                          snippet foo(a, b, c,) {
                            static x = 1;
                            x = 5;
                          }
                          (foo, fun)
                          ";
    let tokens = & mut get_tokens(input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Prog::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }

  #[test]
  fn test_parse_huge_prog() {
    let input       = r"snippet foo(a, b, c, ) {
                          d = 1;
                          x = d;
                        }
                        ".repeat(10000);
    let tokens = & mut get_tokens(&input);
    let token_iter = & mut tokens.iter().peekable();
    println!("{:?}", Prog::parse(token_iter));
    assert!(token_iter.peek().is_none(), "token iterator is not empty");
  }
}
