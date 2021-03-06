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
  var_type  : &'a VarType<'a>,
  var_state : VarState
}

impl <'a> VariableMetadata<'a> {
  pub fn get_var_type(&self) -> &'a VarType { self.var_type }
}

pub struct DefUse<'a> {
  current_snippet : &'a str,
  current_packet : &'a str,
  symbol_table    : HashMap<&'a str, HashMap<&'a str, VariableMetadata<'a>>>,
  global_table    : HashMap<&'a str, VariableMetadata<'a>>,
  packet_table    : HashMap<&'a str, HashMap<&'a str, VariableMetadata<'a>>>,
  field_table     : HashMap<String, HashMap<(String, String), VariableMetadata<'a>>>,
  snippet_set     : HashSet<&'a str>,
  packet_set     : HashSet<&'a str>
}


impl<'a> DefUse<'a> {

  // pub fn get_symbol_table(&'a self) -> &'a HashMap<&'a str, VariableMetadata<'a>> {
  //   self.symbol_table.get(self.current_snippet).unwrap()
  pub fn get_symbol_table(&'a self, snippet : &'a str) -> &'a HashMap<&'a str, VariableMetadata<'a>> {
    self.symbol_table.get(snippet).unwrap()
  }

  pub fn get_packet_table(&'a self, packet : &'a str) -> &'a HashMap<&'a str, VariableMetadata<'a>> {
    self.packet_table.get(packet).unwrap()
  }

  pub fn is_defined(&'a self, id_name : &'a str) -> bool {
    // check if var is global
    if self.global_table.get(id_name).is_some() { 
      return true;
    }

    let sym_table = self.get_symbol_table(self.current_snippet);
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
      current_packet : "",
      symbol_table : HashMap::new(),
      global_table : HashMap::new(),
      packet_table : HashMap::new(),
      field_table  : HashMap::new(),
      snippet_set  : HashSet::new(),
      packet_set   : HashSet::new(),
    }
  }
}




impl<'a> TreeFold<'a> for DefUse<'a> {


  fn visit_snippet(&mut self, tree : &'a Snippet) {
    // Initialize symbol table for this snippet
    self.current_snippet = &tree.snippet_id.get_str();
    if self.snippet_set.get(self.current_snippet) != None {
      panic!("Can't have two snippets named {}.", self.current_snippet);
    } else {
      self.symbol_table.insert(self.current_snippet, HashMap::new());
      self.field_table.insert(self.current_snippet.to_string(), HashMap::new());
      self.snippet_set.insert(self.current_snippet);
    }
    self.visit_variable_decls(&tree.variable_decls);
    self.visit_ifblocks(&tree.ifblocks);
  }


  fn visit_variable_decl(&mut self, tree : &'a VariableDecl) {

    let id_name = &tree.identifier.id_name;
    let type_qualifier = &tree.var_type.type_qualifier;

    if *type_qualifier != TypeQualifier::Global {
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

        // check if variable is a packet and if so, add its fields to field table for this snippet, with state 'Declared'
        match var_type.var_info {

          VarInfo::Packet(ref packet_name) 
             =>  
             {
                if self.packet_table.get_mut(packet_name.id_name).is_some() {
                  for (field_name, var_meta_data) in self.packet_table.get_mut(packet_name.id_name).unwrap() {
                    let v = VariableMetadata{var_type : var_meta_data.var_type, var_state : VarState::Declared};
                    self.field_table.get_mut(self.current_snippet).unwrap().insert((id_name.to_string(),field_name.to_string()), v);
                  }
                } else {
                    panic!("Packet {} not declared.", packet_name.id_name);
                }
              },      
        
          _ => {}
        }
      }
    } else {
        if self.global_table.get(id_name).is_some() {
          panic!("Global variable {} is declared twice", id_name);
        } else {
          let var_type = &tree.var_type;
          let var_state =  VarState::Defined;
          self.global_table.insert(id_name, VariableMetadata{var_type, var_state});
        }
    }
  }


  fn visit_packet(&mut self, tree : &'a Packet) {
    // Initialize field list for this packet
    self.current_packet = &tree.packet_id.get_str();
    if self.packet_set.get(self.current_packet) != None {
      panic!("Can't have two packets named {}.", self.current_packet);
    } else {
      self.packet_table.insert(self.current_packet, HashMap::new());
      self.packet_set.insert(self.current_packet);
    }
    self.visit_packet_fields(&tree.packet_fields);
  }


  fn visit_packet_field(&mut self, tree : &'a PacketField) {

    let id_name = &tree.identifier.id_name;
    let type_qualifier = &tree.var_type.type_qualifier;
    if *type_qualifier == TypeQualifier::Field {
      if self.packet_table.get_mut(self.current_packet).unwrap().get(id_name).is_some() {
        panic!("Field {} is declared twice in {}.",
               id_name,
               self.current_packet);
      } else {
        let var_type = &tree.var_type;
        let var_state =  VarState::Declared ;
        self.packet_table.get_mut(self.current_packet).unwrap().insert(id_name, VariableMetadata{var_type, var_state});
      }
    } else {
      panic!("Packets can only have field variables.\n");
    } 
  }


  fn visit_statement(&mut self, tree : &'a Statement) {

    let id_name =
      match &tree.lvalue {
        &LValue::Scalar(ref identifier) => { identifier.id_name },
        &LValue::Array(ref identifier, _) => { identifier.id_name },
        &LValue::Field(ref identifier, _) => { identifier.id_name }
      };

    let field_name =
      match &tree.lvalue {
        &LValue::Scalar(ref _identifier) => { "" },
        &LValue::Array(ref _identifier, _) => { "" },
        &LValue::Field(_, ref identifier) => { identifier.id_name }
      };

    // First visit expression because that is conceptually processed first
    self.visit_expr(&tree.expr);
    let sym_table = self.symbol_table.get_mut(self.current_snippet).unwrap();

    // if the lvalue is a field, check its validity
    if field_name != "" {

      let f_table = self.field_table.get_mut(self.current_snippet).unwrap();

      // check that the packet has been declared and that the packet contains the field field_name
      match sym_table.get(id_name) {
        None
        => panic!("Defining variable {} that isn't declared in {}.", id_name, self.current_snippet),

        Some(&VariableMetadata{var_type, var_state : VarState::Declared})
        =>  match var_type.var_info {

               VarInfo::Packet(ref identifier) 
               =>   if !self.packet_table.get(identifier.id_name).unwrap().get(field_name).is_some() {
                      panic!("Packet {} has no field named {}.", identifier.id_name, field_name);
                    },

               _ => {panic!("Only packets can have fields");}
            },
        
        _ => ()
      }

      // check that the field is a BitArray and update its var_state to defined in the field_table for the current snippet
      match f_table.get(&(id_name.to_string(), field_name.to_string())) {
        None
        => panic!("Packet field {}.{} isn't declared for snippet {}", id_name, field_name, self.current_snippet),

        Some(&VariableMetadata{var_type, var_state : VarState::Declared})
        =>  match var_type.var_info {

               VarInfo::BitArray(_,_) 
               =>  {
                      f_table.get_mut(&(id_name.to_string(), field_name.to_string())).unwrap().var_state = VarState::Defined;
                    }

               _ => {panic!("Field {}.{} must be a bitarray", id_name, field_name);}
            },

        Some(&VariableMetadata{var_type : _, var_state : VarState::Defined})
        => {  
              assert!(f_table.get(&(id_name.to_string(), field_name.to_string())).unwrap().var_state == VarState::Defined, "var_state should be VarState::Defined.");
              f_table.get_mut(&(id_name.to_string(), field_name.to_string())).unwrap().var_state = VarState::Updated;
            },
        _ 
        => {assert!(f_table.get(&(id_name.to_string(), field_name.to_string())).unwrap().var_state == 
              VarState::Updated,"var_state should be VarState::Updated.");}
      }

    } else {
    // Update var_state in self for id_name
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
  }


  fn visit_expr(&mut self, tree : &'a Expr) {
    // Check def-before-use for first operand
    if tree.op1.is_scalar() &&
       !self.is_defined(tree.op1.get_id()) {
      panic!("{} used before definition", &tree.op1.get_id());
    }

    // Check for the remaining operands
    match &tree.expr_right {
      &ExprRight::BinOp(_, ref op2) => {
        if op2.is_scalar() &&
           !self.is_defined(op2.get_id()) {
          panic!("{} used before definition", op2.get_id());
        }
      }
      &ExprRight::Cond(ref true_op, ref false_op) => {
        if true_op.is_scalar()  &&
           !self.is_defined(true_op.get_id()) {
          panic!("{} used before definition", true_op.get_id());
        }

        if false_op.is_scalar() &&
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
        
        // check if to and from variables have matching bit width and var size
        let varinfo_to =  &self.symbol_table.get(to_snippet).unwrap().get(to_var).unwrap().var_type.var_info;
        let varinfo_from = &self.symbol_table.get(from_snippet).unwrap().get(from_var).unwrap().var_type.var_info;

        let to_width =  match varinfo_to {
          VarInfo::BitArray(bit_width, _var_size) => bit_width,
          VarInfo::Packet(_) => {&0}
        };

        let to_size =  match varinfo_to {
          VarInfo::BitArray(_bit_width, var_size) => var_size,
          VarInfo::Packet(_) => {&0}
        };

        let from_width =  match varinfo_from {
          VarInfo::BitArray(bit_width, _var_size) => bit_width,
          VarInfo::Packet(_) => {&0}
        };

        let from_size =  match varinfo_from {
          VarInfo::BitArray(_bit_width, var_size) => var_size,
          VarInfo::Packet(_) => {&0}
        };

        if to_width != from_width {
          panic!("Bit widths differ in the connection from {}.{} to {}.{}.", from_snippet, from_var, to_snippet, to_var);
        }
        if to_size != from_size {
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

  test_fail!(r"
              snippet foo() {
                 output a : pac;
              }", test_def_use_packet_declared_fail,
             "Packet pac not declared");

  test_fail!(r"
              global a : bit<1> = 1;
              global a : bit<1> = 0;
              ", test_def_use_redeclare_global_fail,
             "Global variable a is declared twice");

  test_fail!(r"
              packet a {}
              packet a {}
              ", test_def_use_redeclare_packet_fail,
             "Can't have two packets named a");
 
  test_fail!(r"
              packet pac {
                a : bit<1>;
                a : bit<1>;
              }
              ", test_def_use_duplicate_field_names_fail,
             "Field a is declared twice in pac.");

  test_fail!(r"
              packet pac {
                q : bit<1>;
              }

              snippet foo() {
                 output x : pac;
                 x.r = 1;
              }", test_def_use_field_in_packet_fail,
             "Packet pac has no field named r.");

  test_fail!(r"
              packet pac {
                q : bit<1>;
              }

              snippet foo() {
                 output x : bit<1>;
                 x.r = 1;
              }", test_def_use_field_in_bitarray_fail,
             "Only packets can have fields");

  test_fail!(r"
              packet pac {
                a : bit<1>;
              }

              snippet foo() {
                 x.a = 1;
              }", test_def_use_field_of_undeclared_var_fail,
             "Defining variable x that isn't declared in foo");


  test_fail!(r"
              snippet foo() {
                 output a : bit<1>;
                 a = 0;
                 a = 1;
              }", test_def_use_field_redefining_variable_fail,
             "Redefining variable a that is already defined in foo");
}
