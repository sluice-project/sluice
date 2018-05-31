// A tree fold trait. This trait walks through an immutable tree and updates an
// accumulator of type Acc in place. Implementations of this trait can override
// methods that process specific types of tree nodes, while using default
// methods for other types of tree nodes.

use super::grammar::*;

pub trait TreeFold<'a, Acc> {
  fn visit_prog(tree : &'a Prog, collector : &mut Acc) {
    Self::visit_snippets(&tree.snippets, collector);
    Self::visit_connections(&tree.connections, collector);
  }

  fn visit_snippets(tree : &'a Snippets, collector : &mut Acc) {
    for snippet in &tree.snippet_vector { Self::visit_snippet(snippet, collector); }
  }
 
  fn visit_snippet(tree : &'a Snippet, collector : &mut Acc) {
    Self::visit_identifier(&tree.snippet_id, collector);
    Self::visit_variable_decls(&tree.variable_decls, collector);
    Self::visit_statements(&tree.statements, collector);
  }

  fn visit_connections(tree : &'a Connections, collector : &mut Acc) {
    for connection in &tree.connection_vector {
      Self::visit_connection(&connection, collector);
    }
  }

  fn visit_connection(tree : &'a Connection, collector : &mut Acc) {
    Self::visit_identifier(&tree.from_snippet, collector);
    Self::visit_identifier(&tree.to_snippet, collector);
  }

  fn visit_variable_decls(tree : &'a VariableDecls, collector : &mut Acc ) {
    for init in &tree.decl_vector { Self::visit_variable_decl(init, collector); }
  }

  fn visit_variable_decl(tree : &'a VariableDecl, collector : &mut Acc) {
    Self::visit_identifier(&tree.identifier, collector);
    for value in &(tree.initial_values) { Self::visit_value(value, collector); };
    Self::visit_var_type(&tree.var_type, collector);
  }

  fn visit_var_type(tree : &'a VarType, collector : &mut Acc) {
    let _ = tree;
    let _ = collector;
    // Do nothing here.    
  }

  fn visit_statements(tree : &'a Statements, collector : &mut Acc) {
    for stmt in &tree.stmt_vector { Self::visit_statement(stmt, collector); }
  }
  
  fn visit_statement(tree : &'a Statement, collector : &mut Acc) {
    Self::visit_lvalue(&tree.lvalue, collector);
    Self::visit_expr(&tree.expr, collector);
  }
  
  fn visit_expr(tree : &'a Expr, collector : &mut Acc) {
    Self::visit_operand(&tree.op1, collector);
    Self::visit_expr_right(&tree.expr_right, collector);
  }
  
  fn visit_expr_right(tree : &'a ExprRight, collector : &mut Acc) {
    match tree {
      &ExprRight::BinOp(_, ref operand) => Self::visit_operand(operand, collector),
      &ExprRight::Cond(ref operand_true, ref operand_false) => {
        Self::visit_operand(operand_true, collector);
        Self::visit_operand(operand_false, collector);
      },
      &ExprRight::Empty() => ()
    }
  }
  
  fn visit_operand(tree : &'a Operand, collector : &mut Acc) {
    match tree {
      &Operand::LValue(ref lvalue) => Self::visit_lvalue(lvalue, collector),
      &Operand::Value(ref value)   => Self::visit_value(value, collector)
    }
  }

  fn visit_lvalue(tree : &'a LValue, collector : &mut Acc) {
    match tree {
      &LValue::Identifier(ref identifier) => Self::visit_identifier(identifier, collector),
      &LValue::Array(ref array, ref operand) => {
        Self::visit_identifier(array, collector);
        Self::visit_operand(operand, collector);
      }
    }
  }
 
  // The awkward let _ is required to suppress the unused variables warning
  // https://github.com/rust-lang/rust/issues/26487
  fn visit_identifier(tree : &'a Identifier, collector : &mut Acc) { let _ = tree; let _ = collector; }
  
  fn visit_value(tree : &'a Value, collector : &mut Acc) { let _ = tree; let _ = collector; }
}
