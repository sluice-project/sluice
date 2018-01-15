#[macro_use]
extern crate lazy_static;

mod lexer;
fn main() {
  let input_program = r"snippet fun(a, b, c, x, y) {
                          static x = 0;
                          if (a >= b) {
                            a = x;
                            b = y;
                          } elif (c >= d) {
                            m == 5;
                          }
                        }";
  lexer::run_lexer(input_program);
}
