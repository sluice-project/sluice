//0. SystemVerilog directives.
//1. Clocking logic.
//2. Constraints file.
//3. sequential and combinational logic in always_ff and always_comb blocks respectively.
//4. Top SystemVerilog file for running specific snippets on an FPGA/simulation quickly.
//5. Specifying which module to run on FPGA or in simulation.
//6. Specification for simplified version of constraints file.
//7. Reset logic for all variables.

// Generate everything (top file, clk, xdc file, tcl script) that is required to test a single snippet.
use super::grammar::*;
use tree_fold::TreeFold;

pub struct CodeGen<'a> {
  snippet_name : &'a str,
  generated_string : String
}

impl<'a> CodeGen<'a> {
  pub fn new(t_snippet_name : &'a str) -> CodeGen<'a> {
    CodeGen{snippet_name : t_snippet_name, generated_string : "".to_string()}
  }
}

impl<'a> TreeFold<'a> for  CodeGen<'a> {
  fn visit_snippet(&mut self, tree : &'a Snippet) {
    if tree.snippet_id.get_str() == self.snippet_name {
      println!("Visit found snippet of interest.");
      self.generated_string.push_str("module ");
      self.generated_string.push_str(tree.snippet_id.get_str());
      self.generated_string.push_str("()\n");
      self.generated_string.push_str("\nendmodule");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::super::lexer;
  use super::super::parser;
  use super::CodeGen;
  use super::super::tree_fold::TreeFold;
  use super::super::def_use::DefUse;
 
  fn run_code_gen(input_program : &str) {
    // Lexing
    let tokens = & mut lexer::get_tokens(input_program);

    // parsing
    let token_iter = & mut tokens.iter().peekable();
    let parse_tree = parser::parse_prog(token_iter);
    assert!(token_iter.peek().is_none(), "token_iter is not empty.");
    println!("Parse tree: {:?}\n", parse_tree);

    // Check that identifiers are defined before use
    let mut def_use = DefUse::new();
    def_use.visit_prog(&parse_tree);

    // Run code generator
    let mut code_gen = CodeGen::new("fun");
    code_gen.visit_prog(&parse_tree);
  }

  #[test]
  fn test_code_gen() {
    let input_program = r"snippet fun() {
                            input x : bit<2>;
                            transient b : bit<2>;
                            persistent m : bit<3> = 5;
                            b = x;
                          }
                          ";
    run_code_gen(input_program);
  }
}
