use super::grammar::*;
use std::collections::HashSet;
use std::collections::HashMap;
use tree_fold::TreeFold;

// Compiler pass to check that identifiers are defined before being used in a snippet
pub struct DefUse;

// SymbolTableCollector type to wrap a:
// 1. string context that represents the current snippet
// 2. and a global symbol table, which is a dictionary
// from strings (snippet names) to
// sets of strings (set of variables within a snippet)
// TODO: This is a bit ugly because our members are public.
// We should be using sensible method calls instead, but
// I don't know how to do that while using lifetimes.
pub struct SymbolTableCollector<'a> { 
  pub current_snippet : &'a str,
  pub symbol_table    : HashMap<&'a str, HashSet<&'a str>>,
}

// Add definitions from initializers, idlist, snippet names, and statements
// Check use of these definitions in visit_expr and visit_connections
impl<'a> TreeFold<'a, SymbolTableCollector<'a>> for DefUse {
  fn visit_initializer(tree : &'a Initializer, collector : &mut SymbolTableCollector<'a>) {
    let &Initializer::Initializer(ref identifier, _) = tree;
    let &Identifier::Identifier(id_string) = identifier;
    if collector.symbol_table.get_mut(collector.current_snippet).unwrap().get(id_string) != None {
      panic!("Static variable {} has same name as {}'s argument variable {}",
             id_string,
             collector.current_snippet,
             id_string);
    }
    collector.symbol_table.get_mut(collector.current_snippet).unwrap().insert(id_string);
  }

  fn visit_idlist(tree : &'a IdList, collector : &mut SymbolTableCollector<'a>) {
    let &IdList::IdList(ref id_vector) = tree;
    for id in id_vector { collector.symbol_table.get_mut(collector.current_snippet).unwrap().insert(id.get_string()); }
  }

  fn visit_snippet(tree : &'a Snippet, collector: &mut SymbolTableCollector<'a>) {
    let &Snippet::Snippet(ref identifier, ref id_list, ref initializers, ref statements) = tree;
    // Initialize symbol table for this snippet
    collector.current_snippet = identifier.get_string();
    if collector.symbol_table.get(collector.current_snippet) != None {
      panic!("Can't have two snippets named {}.", collector.current_snippet);
    } else {
      collector.symbol_table.insert(collector.current_snippet, HashSet::new());
    }
    Self::visit_idlist(id_list, collector);
    Self::visit_initializers(initializers, collector);
    Self::visit_statements(statements, collector);
  }

  fn visit_statement(tree : &'a Statement, collector : &mut SymbolTableCollector<'a>) {
    let &Statement::Statement(ref identifier, ref expr) = tree;
    let &Identifier::Identifier(ref id_string) = identifier;
    collector.symbol_table.get_mut(collector.current_snippet).unwrap().insert(id_string);
    Self::visit_expr(expr, collector);
  }

  fn visit_expr(tree : &'a Expr, collector : &mut SymbolTableCollector<'a>) {
    // Check def-before-use for first operand
    let &Expr::Expr(ref op1, ref expr_right) = tree;
    if op1.is_id() && collector.symbol_table.get_mut(collector.current_snippet).unwrap().get(op1.get_id()) == None { panic!("{} used before definition", op1.get_id()); }

    // Check for the remaining operands
    match expr_right {
      &ExprRight::BinOp(_, ref op2) => {
        if op2.is_id() && collector.symbol_table.get_mut(collector.current_snippet).unwrap().get(op2.get_id()) == None { panic!("{} used before definition", op2.get_id()); }
      }
      &ExprRight::Cond(ref true_op, ref false_op) => {
        if true_op.is_id()  && collector.symbol_table.get_mut(collector.current_snippet).unwrap().get(true_op.get_id())  == None { panic!("{} used before definition", true_op.get_id());}
        if false_op.is_id() && collector.symbol_table.get_mut(collector.current_snippet).unwrap().get(false_op.get_id()) == None { panic!("{} used before definition", false_op.get_id());}
      }
      &ExprRight::Empty() => ()
    }
  }

  // 1. Make sure snippets that are connected are defined.
  // 2. Make sure that variables within a connection are defined in their respective snippets.
  fn visit_connections(tree : &'a Connections, collector: &mut SymbolTableCollector<'a>) {
    let &Connections::Connections(ref connection_vector) = tree;
    for connection in connection_vector {
      let from_snippet = connection.from_snippet.get_string();
      let to_snippet   = connection.to_snippet.get_string();
      if collector.symbol_table.get(from_snippet) == None {
        panic!("{} connected, but undefined", from_snippet);
      }
      if collector.symbol_table.get(to_snippet) == None {
        panic!("{} connected, but undefined", to_snippet);
      }
      for variable_pair in &connection.variable_pairs {
        let from_var = variable_pair.from_var.get_string();
        let to_var   = variable_pair.to_var.get_string();
        if collector.symbol_table.get(from_snippet).unwrap().get(from_var) == None {
          panic!("Trying to connect non-existent variable {} from snippet {}", from_var, from_snippet);
        }
        if collector.symbol_table.get(to_snippet).unwrap().get(to_var) == None {
          panic!("Trying to connect non-existent variable {} from snippet {}", to_var, to_snippet);
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::super::lexer;
  use super::super::parser;
  use super::DefUse;
  use super::SymbolTableCollector;
  use super::super::tree_fold::TreeFold;
  use std::collections::HashMap;
  
  fn run_def_use(input_program : &str) {
    // Lexing
    let tokens = & mut lexer::get_tokens(input_program);
  
    // parsing
    let token_iter = & mut tokens.iter().peekable();
    let parse_tree = parser::parse_prog(token_iter);
    assert!(token_iter.peek().is_none(), "token_iter is not empty.");
    println!("Parse tree: {:?}\n", parse_tree);
  
    // Check that identifiers are defined before use
    let mut def_use_collector = SymbolTableCollector { current_snippet : "", symbol_table : HashMap::new() };
    DefUse::visit_prog(&parse_tree, &mut def_use_collector);
  }
  
  #[test]
  #[should_panic(expected="y used before definition")]
  fn test_def_use_undefined(){
    let input_program = r"snippet fun(x, ) {
                            b = y;
                            m = 5;
                          }
                          ";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="Can't have two snippets named foo.")]
  fn test_connection_du_duplicate() {
    let input_program = r"snippet foo() {} snippet foo() {}";
    run_def_use(input_program);
  }
 
  #[test]
  fn test_def_use_defined_in_arg_list(){
    let input_program = r"snippet foo(a, b, c, ) {
                            x = a;
                          }
                          ";
    run_def_use(input_program);
  }
  
  #[test]
  fn test_def_use_defined_in_prog(){
    let input_program = r"snippet foo(a, b, c, ) {
                            d = 1;
                            x = d;
                          }
                          ";
    run_def_use(input_program);
  }
  
  #[test]
  #[should_panic(expected="Static variable x has same name as fun's argument variable x")]
  fn test_def_use_redefined(){
    let input_program = r"snippet fun(a, b, c, x, y, ) {
                            static x = 0;
                          }
                          ";
    run_def_use(input_program);
  }

  #[test]
  fn test_connection_def_use_ok(){
    let input_program = r"snippet foo() {} snippet bar() {}";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="foo connected, but undefined")]
  fn test_connection_def_use_undefined_snippet() {
    let input_program = r"(foo, fun)";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="Trying to connect non-existent variable c from snippet foo")]
  fn test_connection_def_use_undefined_variable() {
    let input_program = r"snippet foo(a, b,) {} snippet fun(c, d,) {} (foo, fun):c->d,";
    run_def_use(input_program);
  }
}
