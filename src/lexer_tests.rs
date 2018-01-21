#[cfg(test)]
mod tests{
  use super::super::lexer::get_tokens;
  
  #[test]
  fn test_lexer_full_prog() {
    let input_program = r"snippet fun ( a , b , c , x , y, ) {
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
    println!("{:?}", get_tokens(input_program));
  }
}
