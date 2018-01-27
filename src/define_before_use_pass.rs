// (3) static variables are redefined as local variables.
use super::parser::*;
use std::collections::HashSet;
use tree_fold::TreeFold;

// Compiler pass to check that identifiers are defined before being used
pub struct DefineBeforeUsePass;

// Add definitions from initializers, idlist, snippet names, and statements
// Check use of these definitions in visit_expr and visit_connections
impl TreeFold<HashSet<String>> for DefineBeforeUsePass {
  fn visit_initializer(tree : & Initializer, collector : &mut HashSet<String>) {
    let &Initializer::Initializer(ref identifier, _) = tree;
    let &Identifier::Identifier(ref id_string) = identifier;
    collector.insert(id_string.clone());
  }

  fn visit_idlist(tree : & IdList, collector : &mut HashSet<String>) {
    if let &IdList::IdList(ref identifier, ref rest_of_list) = tree {
      collector.insert(identifier.get_string().clone());
      Self::visit_idlist(rest_of_list, collector);
    }
  }

  fn visit_snippet(tree : & Snippet, collector: &mut HashSet<String>) {
    let &Snippet::Snippet(ref id, ref id_list, ref inits, ref stmts) = tree;
    collector.insert(id.get_string().clone());
    Self::visit_idlist(id_list, collector);
    Self::visit_initializers(inits, collector);
    Self::visit_statements(stmts, collector);
  }

  fn visit_statement(tree : &Statement, collector : &mut HashSet<String>) {
    let &Statement::Statement(ref identifier, ref expr) = tree;
    let &Identifier::Identifier(ref id_string) = identifier;
    collector.insert(id_string.clone());
    Self::visit_expr(expr, collector);
  }

  fn visit_connections(tree : & Connections, collector: & mut HashSet<String>) {
    if let &Connections::Connections(ref s1, ref s2, ref the_rest) = tree {
      if collector.get(s1.get_string()) == None { panic!("{} connected, but not defined", s1.get_string()); }
      if collector.get(s2.get_string()) == None { panic!("{} connected, but not defined", s2.get_string()); }
      Self::visit_connections(the_rest, collector);
    }
  }

  fn visit_expr(tree : &Expr, collector : &mut HashSet<String>) {
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
