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
      &Snippet::Snippet(ref identifier, ref id_list, ref initializers, ref statements) => {
        Self::visit_identifier(identifier, collector);
        Self::visit_idlist(id_list, collector);
        Self::visit_initializers(initializers, collector);
        Self::visit_statements(statements, collector);
      }
    }
  }
  
  fn visit_connections(tree : &'a Connections, collector : &mut Acc) {
    let &Connections::Connections(ref connection_vector) = tree;
    for connection in connection_vector {
      Self::visit_identifier(&connection.from_function, collector);
      Self::visit_identifier(&connection.to_function, collector);
    }
  }
  
  fn visit_idlist(tree : &'a IdList, collector : &mut Acc) {
    let &IdList::IdList(ref id_vector) = tree;
    for id in id_vector { Self::visit_identifier(id, collector); }
  }
  
  fn visit_initializers(tree : &'a Initializers, collector : &mut Acc ) {
    let &Initializers::Initializers(ref init_vector) = tree;
    for init in init_vector { Self::visit_initializer(init, collector); }
  }

  fn visit_initializer(tree : &'a Initializer, collector : &mut Acc) {
    match tree {
      &Initializer::Initializer(ref identifier, ref value) => {
        Self::visit_identifier(identifier, collector);
        Self::visit_value(value, collector);
      }
    }
  }
  
  fn visit_statements(tree : &'a Statements, collector : &mut Acc) {
    let &Statements::Statements(ref stmt_vector) = tree;
    for stmt in stmt_vector { Self::visit_statement(stmt, collector); }
  }
  
  fn visit_statement(tree : &'a Statement, collector : &mut Acc) {
    match tree {
      &Statement::Statement(ref identifier, ref expr) => {
        Self::visit_identifier(identifier, collector);
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
      &Operand::Identifier(ref identifier) => Self::visit_identifier(identifier, collector),
      &Operand::Value(ref value)           => Self::visit_value(value, collector)
    }
  }
 
  // The awkward let _ is required to suppress the unused variables warning
  // https://github.com/rust-lang/rust/issues/26487
  fn visit_identifier(tree : &'a Identifier, collector : &mut Acc) { let _ = tree; let _ = collector; }
  
  fn visit_value(tree : &'a Value, collector : &mut Acc) { let _ = tree; let _ = collector; }
}
