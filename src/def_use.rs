use super::grammar::*;
use std::collections::HashSet;
use std::collections::HashMap;
use tree_fold::TreeFold;

// Compiler pass to check that identifiers are defined before being used in a snippet
pub struct DefUse;

// VariableCollector type to wrap a:
// 1. string context that represents the current snippet
// 2. a global transient_vars table, which is a dictionary
// from strings (snippet names) to
// sets of strings (set of transient variables within a snippet)
// 3. a global persistent_vars table, which similarly stores persistent variables
// 4. a global snippet_set, which stores the names of all snippets in a set.
// Note: persistent_vars and transient_vars can overlap because persistent variables
// are typically updated as part of every snippet invocation---unless they are read-only.
// TODO: This is a bit ugly because our members are public.
// We should be using sensible method calls instead, but
// I don't know how to do that while using lifetimes (compiler's errors are confusing)
// TODO: Need to check types (can't assign integers to arrays, etc.)
pub struct VariableCollector<'a> {
  pub current_snippet : &'a str,
  pub transient_vars  : HashMap<&'a str, HashSet<&'a str>>,
  pub persistent_vars : HashMap<&'a str, HashSet<&'a str>>,
  pub snippet_set     : HashSet<&'a str>
}

// Add definitions from persistent_decls, idlist, snippet names, and statements
// Check use of these definitions in visit_expr and visit_connections
impl<'a> TreeFold<'a, VariableCollector<'a>> for DefUse {
  fn visit_persistent_decl(tree : &'a PersistentDecl, collector : &mut VariableCollector<'a>) {
    let &Identifier::Identifier(id_string) = &tree.identifier;
    if collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(id_string) != None {
      panic!("Persistent variable {} has same name as {}'s argument variable {}",
             id_string,
             collector.current_snippet,
             id_string);
    } else {
      collector.persistent_vars.get_mut(collector.current_snippet).unwrap().insert(id_string);
    }
  }

  fn visit_idlist(tree : &'a IdList, collector : &mut VariableCollector<'a>) {
    let &IdList::IdList(ref id_vector) = tree;
    for id in id_vector {
      if collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(id.get_string()) != None {
        panic!("Variable {} repeated twice in {}'s argument list", id.get_string(), collector.current_snippet);
      } else {
        collector.transient_vars.get_mut(collector.current_snippet).unwrap().insert(id.get_string());
      }
    }
  }

  fn visit_snippet(tree : &'a Snippet, collector: &mut VariableCollector<'a>) {
    let &Snippet::Snippet(ref identifier, ref id_list, ref persistent_decls, ref transient_decls, ref statements) = tree;
    // Initialize symbol table for this snippet
    collector.current_snippet = identifier.get_string();
    if collector.transient_vars.get(collector.current_snippet) != None {
      panic!("Can't have two snippets named {}.", collector.current_snippet);
    } else {
      collector.transient_vars.insert(collector.current_snippet, HashSet::new());
      collector.persistent_vars.insert(collector.current_snippet, HashSet::new());
      collector.snippet_set.insert(collector.current_snippet);
    }
    Self::visit_idlist(id_list, collector);
    Self::visit_persistent_decls(persistent_decls, collector);
    Self::visit_statements(statements, collector);
  }

  fn visit_statement(tree : &'a Statement, collector : &mut VariableCollector<'a>) {
    let &Statement::Statement(ref lvalue, ref expr) = tree;
    let id_string =
      match lvalue {
        &LValue::Identifier(Identifier::Identifier(x)) => { x },
        &LValue::Array(Identifier::Identifier(x), _) => { x }
      };

    // First visit expression because that is conceptually processed first
    Self::visit_expr(expr, collector);

    // Then process id_string;
    if collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(id_string) != None {
      panic!("Can't redefine transient var {} in {}. Transients are immutable for now.", id_string, collector.current_snippet);
    } else {
      collector.transient_vars.get_mut(collector.current_snippet).unwrap().insert(id_string);
    }
  }

  fn visit_expr(tree : &'a Expr, collector : &mut VariableCollector<'a>) {
    // Check def-before-use for first operand
    let &Expr::Expr(ref op1, ref expr_right) = tree;
    if op1.is_id() &&
       collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(op1.get_id()) == None &&
       collector.persistent_vars.get_mut(collector.current_snippet).unwrap().get(op1.get_id()) == None {
      panic!("{} used before definition", op1.get_id());
    }

    // Check for the remaining operands
    match expr_right {
      &ExprRight::BinOp(_, ref op2) => {
        if op2.is_id() &&
           collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(op2.get_id()) == None &&
           collector.persistent_vars.get_mut(collector.current_snippet).unwrap().get(op2.get_id()) == None {
          panic!("{} used before definition", op2.get_id());
        }
      }
      &ExprRight::Cond(ref true_op, ref false_op) => {
        if true_op.is_id()  &&
           collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(true_op.get_id())  == None &&
           collector.persistent_vars.get_mut(collector.current_snippet).unwrap().get(true_op.get_id()) == None {
          panic!("{} used before definition", true_op.get_id());
        }

        if false_op.is_id() &&
           collector.transient_vars.get_mut(collector.current_snippet).unwrap().get(false_op.get_id()) == None &&
           collector.persistent_vars.get_mut(collector.current_snippet).unwrap().get(false_op.get_id()) == None {
          panic!("{} used before definition", false_op.get_id());
        }
      }
      &ExprRight::Empty() => ()
    }
  }

  // 1. Make sure snippets that are connected are defined.
  // 2. Make sure that variables within a connection are defined in their respective snippets.
  fn visit_connections(tree : &'a Connections, collector: &mut VariableCollector<'a>) {
    let &Connections::Connections(ref connection_vector) = tree;
    for connection in connection_vector {
      let from_snippet = connection.from_snippet.get_string();
      let to_snippet   = connection.to_snippet.get_string();
      if collector.snippet_set.get(from_snippet) == None {
        panic!("{} connected, but undefined", from_snippet);
      }
      if collector.snippet_set.get(to_snippet) == None {
        panic!("{} connected, but undefined", to_snippet);
      }
      for variable_pair in &connection.variable_pairs {
        let from_var = variable_pair.from_var.get_string();
        let to_var   = variable_pair.to_var.get_string();
        if collector.transient_vars.get(from_snippet).unwrap().get(from_var) == None {
          panic!("Trying to connect non-existent variable {} from snippet {}", from_var, from_snippet);
        }
        if collector.transient_vars.get(to_snippet).unwrap().get(to_var) == None {
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
  use super::VariableCollector;
  use super::super::tree_fold::TreeFold;
  use std::collections::HashMap;
  use std::collections::HashSet;
  
  fn run_def_use(input_program : &str) {
    // Lexing
    let tokens = & mut lexer::get_tokens(input_program);
  
    // parsing
    let token_iter = & mut tokens.iter().peekable();
    let parse_tree = parser::parse_prog(token_iter);
    assert!(token_iter.peek().is_none(), "token_iter is not empty.");
    println!("Parse tree: {:?}\n", parse_tree);
  
    // Check that identifiers are defined before use
    let mut def_use_collector = VariableCollector { current_snippet : "",
                                                    transient_vars : HashMap::new(),
                                                    persistent_vars : HashMap::new(),
                                                    snippet_set : HashSet::new() };
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
  #[should_panic(expected="x used before definition")]
  fn test_def_use_undefined2(){
    let input_program = r"snippet fun(a,) {
                            x = x + 1;
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
  fn test_def_use_defined_in_persistent(){
    let input_program = r"snippet foo(a, b, c, ) {
                            persistent d : bit<1> = 1;
                            x = d;
                          }
                          ";
    run_def_use(input_program);
  }

  #[test]
  fn test_def_use_defined_in_persistent2(){
    let input_program = r"snippet foo(a, b, c, ) {
                            persistent d : bit<1> = 1;
                            y = d + a;
                            x = d ? a : b;
                          }
                          ";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="Persistent variable x has same name as fun's argument variable x")]
  fn test_def_use_redefined_persistent_arglist(){
    let input_program = r"snippet fun(a, b, c, x, y, ) {
                            persistent x : bit<1> = 0;
                          }
                          ";
    run_def_use(input_program);
  }

  #[test]
  fn test_def_use_connections(){
    let input_program = r"snippet foo() {} snippet bar() {}";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="foo connected, but undefined")]
  fn test_def_use_connections_undefined_snippet() {
    let input_program = r"(foo, fun)";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="Trying to connect non-existent variable c from snippet foo")]
  fn test_def_use_connections_undefined_variable() {
    let input_program = r"snippet foo(a, b,) {} snippet fun(c, d,) {} (foo, fun):c->d,";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="Variable a repeated twice in foo's argument list")]
  fn test_def_use_repeated_arguments() {
    let input_program = r"snippet foo(a, a,) {}";
    run_def_use(input_program);
  }

  #[test]
  #[should_panic(expected="Can't redefine transient var a in foo. Transients are immutable for now.")]
  fn test_def_use_redefine_transient_var() {
    let input_program = r"snippet foo(a, b,) { a = 1; }";
    run_def_use(input_program);
  }

  #[test]
  fn test_def_use_redefine_transient_persistent() {
    let input_program = r"snippet foo(a, b,) { persistent d : bit<2> = 1; d = 5; }";
    run_def_use(input_program);
  }
}
