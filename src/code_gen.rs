//0. SystemVerilog directives.
//1. Clocking logic.
//2. Constraints file.
//3. sequential and combinational logic in always_ff and always_comb blocks respectively.
//4. Top SystemVerilog file for running specific snippets on an FPGA/simulation quickly.
//5. Specifying which module to run on FPGA or in simulation.
//6. Specification for simplified version of constraints file.

use super::grammar::*;
use tree_fold::TreeFold;
pub struct CodeGen;

impl<'a> TreeFold<'a, String> for  CodeGen {
  fn visit_prog(tree : &'a Prog, collector : &mut String) {
    collector.push_str("program"); 
  }
}

#[cfg(test)]
mod tests {
  use super::super::lexer;
  use super::super::parser;
  use super::CodeGen;
  use super::super::tree_fold::TreeFold;
 
  fn run_code_gen(input_program : &str) {
    // Lexing
    let tokens = & mut lexer::get_tokens(input_program);

    // parsing
    let token_iter = & mut tokens.iter().peekable();
    let parse_tree = parser::parse_prog(token_iter);
    assert!(token_iter.peek().is_none(), "token_iter is not empty.");
    println!("Parse tree: {:?}\n", parse_tree);

    // Run code generator
    let mut generated_code = String::new();
    CodeGen::visit_prog(&parse_tree, &mut generated_code);
  }

  #[test]
  fn test_code_gen(){
    let input_program = r"snippet fun() {
                            input x : bit<2>;
                            b = y;
                            m = 5;
                          }
                          ";
    run_code_gen(input_program);
  }
}
