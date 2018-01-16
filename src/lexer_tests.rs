use super::lexer::get_tokens;

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
  println!("{:?}", get_tokens(input_program));
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
                        }
                        snippet foo(a, b, c) {
                          static x = 1;
                          x = 5;
                        }
                        (foo, fun) 
                        ";
  println!("{:?}", get_tokens(input_program));
}
