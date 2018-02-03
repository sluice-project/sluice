use super::grammar::*;
use std::collections::HashSet;
use tree_fold::TreeFold;

// Compiler pass to check that identifiers are defined before being used
pub struct DefineBeforeUsePass;

// Add definitions from initializers, idlist, snippet names, and statements
// Check use of these definitions in visit_expr and visit_connections
impl<'a> TreeFold<'a, HashSet<&'a str>> for DefineBeforeUsePass {
  fn visit_initializer(tree : &'a Initializer, collector : &mut HashSet<&'a str>) {
    let &Initializer::Initializer(ref identifier, _) = tree;
    let &Identifier::Identifier(id_string) = identifier;
    if collector.get(id_string) != None { panic!("Can't initialize {} that is already defined", id_string); }
    collector.insert(id_string);
  }

  fn visit_idlist(tree : &'a IdList, collector : &mut HashSet<&'a str>) {
    let &IdList::IdList(ref id_vector) = tree;
    for id in id_vector { collector.insert(id.get_string()); }
  }

  fn visit_snippet(tree : &'a Snippet, collector: &mut HashSet<&'a str>) {
    let &Snippet::Snippet(ref identifier, ref id_list, ref initializers, ref statements) = tree;
    collector.insert(identifier.get_string());
    Self::visit_idlist(id_list, collector);
    Self::visit_initializers(initializers, collector);
    Self::visit_statements(statements, collector);
  }

  fn visit_statement(tree : &'a Statement, collector : &mut HashSet<&'a str>) {
    let &Statement::Statement(ref identifier, ref expr) = tree;
    let &Identifier::Identifier(ref id_string) = identifier;
    collector.insert(id_string);
    Self::visit_expr(expr, collector);
  }

  fn visit_connections(tree : &'a Connections, collector: &mut HashSet<&'a str>) {
    let &Connections::Connections(ref connection_vector) = tree;
    for connection in connection_vector {
      if collector.get(connection.from_function.get_string()) == None { panic!("{} connected, but undefined", connection.from_function.get_string()); }
      if collector.get(connection.to_function.get_string()) == None { panic!("{} connected, but undefined", connection.to_function.get_string()); }
    }
  }

  fn visit_expr(tree : &'a Expr, collector : &mut HashSet<&'a str>) {
    // Check def-before-use for first operand
    let &Expr::Expr(ref op1, ref expr_right) = tree;
    if op1.is_id() && collector.get(op1.get_id()) == None { panic!("{} used before definition", op1.get_id()); }

    // Check for the remaining operands
    match expr_right {
      &ExprRight::BinOp(_, ref op2) => {
        if op2.is_id() && collector.get(op2.get_id()) == None { panic!("{} used before definition", op2.get_id()); }
      }
      &ExprRight::Cond(ref true_op, ref false_op) => {
        if true_op.is_id()  && collector.get(true_op.get_id())  == None { panic!("{} used before definition", true_op.get_id());}
        if false_op.is_id() && collector.get(false_op.get_id()) == None { panic!("{} used before definition", false_op.get_id());}
      }
      &ExprRight::Empty() => ()
    }
  }
}
