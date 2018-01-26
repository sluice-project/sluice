// Check if:
// (2) snippets are connected before being defined.
// (3) static variables are redefined as local variables.

use super::parser::*;
use std::collections::HashSet;
use tree_fold::TreeFold;

// Compiler pass to check that identifiers are defined before being used
pub struct DefineBeforeUsePass;

impl TreeFold<HashSet<String>> for DefineBeforeUsePass {
  // Add definitions from initializers
  fn visit_initializer(tree : & Initializer, collector : &mut HashSet<String>) {
    let &Initializer::Initializer(ref identifier, _) = tree;
    let &Identifier::Identifier(ref id_string) = identifier;
    collector.insert(id_string.clone());
  }

  // Add definitions from statements
  fn visit_statement(tree : &Statement, collector : &mut HashSet<String>) {
    let &Statement::Statement(ref identifier, ref expr) = tree;
    let &Identifier::Identifier(ref id_string) = identifier;
    collector.insert(id_string.clone());
    Self::visit_expr(expr, collector);
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
