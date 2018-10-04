use grammar::*;
use std::collections::HashSet;
use std::collections::HashMap;
use tree_fold::TreeFold;

// Defined => Declared, but not the other way around.
#[derive(PartialEq)]
pub enum VarState {
  Declared,
  Defined,
  Updated,
}

pub struct VariableMetadata<'a> {
  var_type  : &'a VarType,
  var_state : VarState
}

pub struct DefUse<'a> {
  current_snippet : &'a str,
  symbol_table    : HashMap<&'a str, HashMap<&'a str, VariableMetadata<'a>>>,
  snippet_set     : HashSet<&'a str>
}

impl<'a> DefUse<'a> {
  pub fn get_symbol_table(&'a self) -> &'a HashMap<&'a str, VariableMetadata<'a>> {
    self.symbol_table.get(self.current_snippet).unwrap()
  }

  pub fn is_defined(&'a self, id_name : &'a str) -> bool {
    let sym_table = self.get_symbol_table();
    if sym_table.get(id_name).is_none() {
      // It's not even declared
      return false;
    } else if sym_table.get(id_name).unwrap().var_state == VarState::Declared {
      return false;
    } else {
      assert!(sym_table.get(id_name).unwrap().var_state == VarState::Defined, "var_state should be VarState::Defined.");
      return true;
    }
  }

  pub fn new() -> DefUse<'a> {
    DefUse {
      current_snippet : "",
      symbol_table : HashMap::new(),
      snippet_set  : HashSet::new(),
    }
  }
}

impl<'a> TreeFold<'a> for DefUse<'a> {
  fn visit_variable_decl(&mut self, tree : &'a VariableDecl) {
    let id_name = &tree.identifier.id_name;
    if self.symbol_table.get_mut(self.current_snippet).unwrap().get(id_name).is_some() {
      panic!("Variable {} is declared twice in {}.",
             id_name,
             self.current_snippet);
    } else {
      let var_type = &tree.var_type;
      let type_qualifier = &var_type.type_qualifier;
      let var_state = if (*type_qualifier == TypeQualifier::Input) ||
                         (*type_qualifier == TypeQualifier::Const) ||
                         (*type_qualifier == TypeQualifier::Persistent) { VarState::Defined }
                      else { VarState::Declared };
      self.symbol_table.get_mut(self.current_snippet).unwrap().insert(id_name, VariableMetadata{var_type, var_state});
    }
  }

  fn visit_snippet(&mut self, tree : &'a Snippet) {
    // Initialize symbol table for this snippet
    self.current_snippet = &tree.snippet_id.get_str();
    if self.snippet_set.get(self.current_snippet) != None {
      panic!("Can't have two snippets named {}.", self.current_snippet);
    } else {
      self.symbol_table.insert(self.current_snippet, HashMap::new());
      self.snippet_set.insert(self.current_snippet);
    }
    self.visit_variable_decls(&tree.variable_decls);
    self.visit_ifblocks(&tree.ifblocks);
  }


  fn visit_statement(&mut self, tree : &'a Statement) {
    let id_name =
      match &tree.lvalue {
        &LValue::Identifier(ref identifier) => { identifier.id_name },
        &LValue::Array(ref identifier, _) => { identifier.id_name }
      };

    // First visit expression because that is conceptually processed first
    self.visit_expr(&tree.expr);

    // Update var_state in self for id_name
    let sym_table = self.symbol_table.get_mut(self.current_snippet).unwrap();
    match sym_table.get(id_name) {
      None
      => panic!("Defining variable {} that isn't declared in {}.", id_name, self.current_snippet),

      Some(&VariableMetadata{var_type, var_state : VarState::Defined})
      =>  if var_type.type_qualifier == TypeQualifier::Persistent { sym_table.get_mut(id_name).unwrap().var_state = VarState::Updated;
          } else {
            if var_type.type_qualifier == TypeQualifier::Const {
              panic!("Trying to update const variable {} in {}.", id_name, self.current_snippet);
            } else if var_type.type_qualifier == TypeQualifier::Input {
              panic!("Trying to update input variable {} in {}. Inputs are implicity defined by caller.", id_name, self.current_snippet);
            } else {
              panic!("Redefining variable {} that is already defined in {}.", id_name, self.current_snippet);
            }
          },

      Some(&VariableMetadata{var_type, var_state : VarState::Updated})
      =>  {assert!(var_type.type_qualifier == TypeQualifier::Persistent, "Only persistent variables can be in updated state.");
           panic!("Can update a persistent variable at most once.");},

      _
      => {assert!(sym_table.get(id_name).unwrap().var_state == VarState::Declared,
         "var_state should be VarState::Declared.");
          sym_table.get_mut(id_name).unwrap().var_state = VarState::Defined;}
    }
  }

  fn visit_expr(&mut self, tree : &'a Expr) {
    // Check def-before-use for first operand
    if tree.op1.is_id() &&
       !self.is_defined(tree.op1.get_id()) {
      panic!("{} used before definition", &tree.op1.get_id());
    }

    // Check for the remaining operands
    match &tree.expr_right {
      &ExprRight::BinOp(_, ref op2) => {
        if op2.is_id() &&
           !self.is_defined(op2.get_id()) {
          panic!("{} used before definition", op2.get_id());
        }
      }
      &ExprRight::Cond(ref true_op, ref false_op) => {
        if true_op.is_id()  &&
           !self.is_defined(true_op.get_id()) {
          panic!("{} used before definition", true_op.get_id());
        }

        if false_op.is_id() &&
           !self.is_defined(false_op.get_id()) {
          panic!("{} used before definition", false_op.get_id());
        }
      }
      &ExprRight::Empty() => ()
    }
  }

  // 1. Make sure snippets that are connected are defined.
  // 2. Make sure that variables within a connection are defined in their respective snippets and
  //    are output/input variables in the source/destination snippets respectively.
  fn visit_connections(&mut self, tree : &'a Connections) {
    for connection in &tree.connection_vector {
      let from_snippet = connection.from_snippet.get_str();
      let to_snippet   = connection.to_snippet.get_str();
      if self.snippet_set.get(from_snippet) == None {
        panic!("{} connected, but undefined", from_snippet);
      }
      if self.snippet_set.get(to_snippet) == None {
        panic!("{} connected, but undefined", to_snippet);
      }
      for variable_pair in &connection.variable_pairs {
        let from_var = variable_pair.from_var.get_str();
        let to_var   = variable_pair.to_var.get_str();
        if self.symbol_table.get(from_snippet).unwrap().get(from_var).is_none() {
          panic!("Trying to connect non-existent variable {} from snippet {}", from_var, from_snippet);
        } else {
          if self.symbol_table.get(from_snippet).unwrap().get(from_var).unwrap().var_type.type_qualifier !=
             TypeQualifier::Output {
            panic!("Trying to connect non-output variable {} in origin snippet {}", from_var, from_snippet);
          }
        }

        if self.symbol_table.get(to_snippet).unwrap().get(to_var).is_none() {
          panic!("Trying to connect non-existent variable {} from snippet {}", to_var, to_snippet);
        } else {
          if self.symbol_table.get(to_snippet).unwrap().get(to_var).unwrap().var_type.type_qualifier !=
             TypeQualifier::Input {
            panic!("Trying to connect non-input variable {} in destination snippet {}", to_var, to_snippet);
          }
        }

        assert!(self.symbol_table.get(to_snippet).unwrap().get(to_var).is_some(), "to_var undefined");
        assert!(self.symbol_table.get(from_snippet).unwrap().get(from_var).is_some(), "from_var undefined");
        if self.symbol_table.get(to_snippet).unwrap().get(to_var).unwrap().var_type.bit_width !=
           self.symbol_table.get(from_snippet).unwrap().get(from_var).unwrap().var_type.bit_width {
          panic!("Bit widths differ in the connection from {}.{} to {}.{}.", from_snippet, from_var, to_snippet, to_var);
        }
        if self.symbol_table.get(to_snippet).unwrap().get(to_var).unwrap().var_type.var_size !=
           self.symbol_table.get(from_snippet).unwrap().get(from_var).unwrap().var_type.var_size {
          panic!("Var sizes differ in the connection from {}.{} to {}.{}.", from_snippet, from_var, to_snippet, to_var);
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
  use super::super::tree_fold::TreeFold;

  fn run_def_use(input_program : &str) {
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
  }

  macro_rules! test_pass {
    ($input_code:expr,$test_name:ident) => (
      #[test]
      fn $test_name() {
        let input_program = $input_code;
        run_def_use(input_program);
      }
    )
  }

  macro_rules! test_fail {
    ($input_code:expr,$test_name:ident,$panic_msg:expr) => (
      #[test]
      #[should_panic(expected=$panic_msg)]
      fn $test_name() {
        let input_program = $input_code;
        run_def_use(input_program);
      }
    )
  }

  test_fail!(r"snippet fun() {
                 input x : bit<2>;
                 b = y;
                 m = 5;
               }
             ", test_def_use_undefined_fail,
             "y used before definition");

  test_fail!(r"snippet fun() {
                 input a : bit<2>;
                 x = x + 1;
               }
             ", test_def_use_undefined2_fail,
             "x used before definition");

  test_fail!(r"snippet foo() {} snippet foo() {}",
             test_def_use_duplicate_snippets_fail,
             "Can't have two snippets named foo.");

  test_fail!(r"snippet foo() {
                 input a : bit<2>;
                 input b : bit<2>;
                 input c : bit<2>;
                 x = a;
             }", test_def_use_undeclared_fail,
             "Defining variable x that isn't declared in foo.");

  test_pass!(r"snippet foo() {
                 input a : bit<2>;
                 input b : bit<2>;
                 input c : bit<2>;
                 transient x: bit<2>;
                 x = a;
             }", test_def_use1);

  test_pass!(r"snippet foo() {
                 transient d : bit<2>;
                 transient x : bit<2>;
                 d = 1;
                 x = d;
             }", test_def_use2);

  test_pass!(r"snippet foo() {
                 transient x : bit<1> = 1;
                 persistent d : bit<1> = 1;
                 x = d;
             }", test_def_use3);

  test_pass!(r"snippet foo() {
                input a : bit<2>;
                input b : bit<2>;
                persistent d : bit<1> = 1;
                transient x : bit<1>;
                transient y : bit<1>;
                y = d + a;
                x = d ? a : b;
             }", test_def_use4);

  test_fail!(r"snippet fun() {
                 input c : bit<2>;
                 input x : bit<2>;
                 persistent x : bit<1> = 0;
             }", test_def_use_redefined_persistent_arglist_fail,
             "Variable x is declared twice in fun.");

  test_pass!(r"snippet foo() {} snippet bar() {}",
             test_def_use_connections_empty);

  test_pass!(r"snippet foo() {output c : bit<2>;} snippet fun() { input d : bit<2>;} (foo, fun):c->d,",
             test_def_use_connections);

  test_pass!(r"snippet foo() {output c : bit<2>[1];} snippet fun() { input d : bit<2>;} (foo, fun):c->d,",
             test_def_use_connections2);

  test_fail!(r"snippet foo() {output c : bit<1>;} snippet fun() { input d : bit<2>;} (foo, fun):c->d,",
             test_def_use_connections_bitwidth_fail,
             "Bit widths differ in the connection from foo.c to fun.d.");

  test_fail!(r"snippet foo() {output c : bit<1>[2];} snippet fun() { input d : bit<1>;} (foo, fun):c->d,",
             test_def_use_connections_varsize_fail,
             "Var sizes differ in the connection from foo.c to fun.d");

  test_fail!(r"snippet foo() {input c : bit<2>;} snippet fun() { input d : bit<2>;} (foo, fun):c->d,",
             test_def_use_connection_fail1,
             "Trying to connect non-output variable c in origin snippet foo");

  test_fail!(r"snippet foo() {output c : bit<2>;} snippet fun() { output d : bit<2>;} (foo, fun):c->d,",
             test_def_use_connection_fail2,
             "Trying to connect non-input variable d in destination snippet fun");

  test_fail!(r"(foo, fun)", test_def_use_connections_undefined_snippet_fail,
             "foo connected, but undefined");

  test_fail!(r"snippet foo() {
               }
               snippet fun() {
                 input d : bit<2>;
               } (foo, fun):c->d,
             ", test_def_use_connections_undefined_variable_fail,
             "Trying to connect non-existent variable c from snippet foo");

  test_fail!(r"snippet foo() {
                 input a : bit<2>;
                 input a : bit<2>;
             }", test_def_use_repeated_arguments_fail, "Variable a is declared twice in foo.");

  test_pass!(r"snippet foo() {
                 input a : bit<2>;
                 input b : bit<2>;
             }", test_def_use_two_arguments);

  test_fail!(r"snippet foo() {
                 input a : bit<2>;
                 a = 1;
             }", test_def_use_redefine_input_fail,
             "Trying to update input variable a in foo. Inputs are implicity defined by caller.");

  test_pass!(r"snippet foo() {
                 input a : bit<2>;
                 input b : bit<2>;
                 persistent d : bit<2> = 1;
                 d = 5;
             }", test_def_use_redefine_persistent);

  test_fail!(r"snippet foo() {
                 input a : bit<2>;
                 input b : bit<2>;
                 persistent d : bit<2> = 1;
                 d = 5;
                 d = 6;
             }", test_def_use_reupdate_persistent_fail,
             "Can update a persistent variable at most once.");

  test_fail!(r"snippet foo() {
                 input a : bit<2>;
                 input b : bit<2>;
                 const x : bit<2> = 1;
                 x = 1;
             }", test_def_use_const_update_fail,
             "Trying to update const variable x in foo.");
}
