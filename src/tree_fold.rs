// A tree fold trait. This trait walks through an immutable tree and updates an
// accumulator of type Acc in place. Implementations of this trait can override
// methods that process specific types of tree nodes, while using default
// methods for other types of tree nodes.

use super::grammar::*;

pub trait TreeFold<'a, Acc> {
  fn visit_prog(tree : &'a Prog, collector : &mut Acc) {
    match tree {
      &Prog::Prog(ref snippets, ref connections) => {
        Self::visit_snippets(snippets, collector);
        Self::visit_connections(connections, collector);
      }
    }
  }

  fn visit_snippets(tree : &'a Snippets, collector : &mut Acc) {
    let &Snippets::Snippets(ref snippet_vector) = tree;
    for snippet in snippet_vector { Self::visit_snippet(snippet, collector); }
  }
 
  fn visit_snippet(tree : &'a Snippet, collector : &mut Acc) {
    match tree {
      &Snippet::Snippet(ref identifier, ref id_list, ref persistent_decls, ref transient_decls, ref statements) => {
        // TODO: do something with transient_decls
        Self::visit_identifier(identifier, collector);
        Self::visit_idlist(id_list, collector);
        Self::visit_persistent_decls(persistent_decls, collector);
        Self::visit_statements(statements, collector);
      }
    }
  }

  fn visit_connections(tree : &'a Connections, collector : &mut Acc) {
    let &Connections::Connections(ref connection_vector) = tree;
    for connection in connection_vector {
      Self::visit_identifier(&connection.from_snippet, collector);
      Self::visit_identifier(&connection.to_snippet, collector);
    }
  }
  
  fn visit_idlist(tree : &'a IdList, collector : &mut Acc) {
    let &IdList::IdList(ref id_vector) = tree;
    for id in id_vector { Self::visit_identifier(id, collector); }
  }
  
  fn visit_persistent_decls(tree : &'a PersistentDecls, collector : &mut Acc ) {
    let &PersistentDecls::PersistentDecls(ref init_vector) = tree;
    for init in init_vector { Self::visit_persistent_decl(init, collector); }
  }

  fn visit_persistent_decl(tree : &'a PersistentDecl, collector : &mut Acc) {
    Self::visit_identifier(&tree.identifier, collector);
    Self::visit_initial_value(&tree.initial_value, collector);
  }

  fn visit_initial_value(tree : &'a InitialValue, collector : &mut Acc) {
    match tree {
      &InitialValue::Value(ref value) => Self::visit_value(value, collector),
      &InitialValue::ValueList(ValueList::ValueList(ref value_vector)) =>
       { for value in value_vector { Self::visit_value(value, collector); } }
    }
  }
  
  fn visit_statements(tree : &'a Statements, collector : &mut Acc) {
    let &Statements::Statements(ref stmt_vector) = tree;
    for stmt in stmt_vector { Self::visit_statement(stmt, collector); }
  }
  
  fn visit_statement(tree : &'a Statement, collector : &mut Acc) {
    match tree {
      &Statement::Statement(ref lvalue, ref expr) => {
        Self::visit_lvalue(lvalue, collector);
        Self::visit_expr(expr, collector);
      }
    }
  }
  
  fn visit_expr(tree : &'a Expr, collector : &mut Acc) {
    match tree {
      &Expr::Expr(ref operand, ref expr_right) => {
        Self::visit_operand(operand, collector);
        Self::visit_expr_right(expr_right, collector);
      }
    }
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
