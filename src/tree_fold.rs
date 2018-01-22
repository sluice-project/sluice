// A tree fold trait. This trait walks through an immutable tree and updates an
// accumulator of type Acc in place. Implementations of this trait can override
// methods that process specific types of tree nodes, while using default
// methods for other types of tree nodes.

use super::parser::*;

pub trait TreeFold<Acc> {
  fn visit_prog(tree : &Prog, collector : &mut Acc) {
    match tree {
      &Prog::Prog(ref snippets, ref connections) => {
        Self::visit_snippets(snippets, collector);
        Self::visit_connections(connections, collector);
      }
    }
  }

  fn visit_snippets(tree : &Snippets, collector : &mut Acc) {
    match tree {
      &Snippets::Snippets(ref snippet, ref snippets) => {
        Self::visit_snippet(snippet, collector);
        Self::visit_snippets(snippets, collector);
      },
      _                                              => ()
    }
  }
  
  fn visit_snippet(tree : &Snippet, collector : &mut Acc) {
    match tree {
      &Snippet::Snippet(ref id, ref id_list, ref inits, ref statements) => {
        Self::visit_identifier(id, collector);
        Self::visit_idlist(id_list, collector);
        Self::visit_initializers(inits, collector);
        Self::visit_statements(statements, collector);
      }
    }
  }
  
  fn visit_connections(tree : &Connections, collector : &mut Acc) {
    match tree {
      &Connections::Connections(ref id1, ref id2, ref connections) => {
        Self::visit_identifier(id1, collector);
        Self::visit_identifier(id2, collector);
        Self::visit_connections(connections, collector);
      },
      &Connections::Empty() => () 
    }
  }
  
  fn visit_idlist(tree : &IdList, collector : &mut Acc) {
    match tree {
      &IdList::IdList(ref identifier, ref id_list) => {
        Self::visit_identifier(identifier, collector);
        Self::visit_idlist(id_list, collector);
      },
      &IdList::Empty() => ()
    }
  }
  
  fn visit_initializers(tree : &Initializers, collector : &mut Acc ) {
    match tree {
      &Initializers::Initializers(ref initializer, ref initializers) => {
        Self::visit_initializer(initializer, collector);
        Self::visit_initializers(initializers, collector);
      },
      &Initializers::Empty() => ()
    }
  }
  
  fn visit_initializer(tree : &Initializer, collector : &mut Acc) {
    match tree {
      &Initializer::Initializer(ref identifier, ref value) => {
        Self::visit_identifier(identifier, collector);
        Self::visit_value(value, collector);
      }
    }
  }
  
  fn visit_statements(tree : &Statements, collector : &mut Acc) {
    match tree {
      &Statements::Statements(ref statement, ref statements) => {
        Self::visit_statement(statement, collector);
        Self::visit_statements(statements, collector);
      },
      &Statements::Empty() => ()
    }
  }
  
  fn visit_statement(tree : &Statement, collector : &mut Acc) {
    match tree {
      &Statement::Statement(ref identifier, ref expr) => {
        Self::visit_identifier(identifier, collector);
        Self::visit_expr(expr, collector);
      }
    }
  }
  
  fn visit_expr(tree : &Expr, collector : &mut Acc) {
    match tree {
      &Expr::Expr(ref operand, ref expr_right) => {
        Self::visit_operand(operand, collector);
        Self::visit_expr_right(expr_right, collector);
      }
    }
  }
  
  fn visit_expr_right(tree : &ExprRight, collector : &mut Acc) {
    match tree {
      &ExprRight::BinOp(_, ref operand) => Self::visit_operand(operand, collector),
      &ExprRight::Cond(ref operand_true, ref operand_false) => {
        Self::visit_operand(operand_true, collector);
        Self::visit_operand(operand_false, collector);
      },
      &ExprRight::Empty() => ()
    }
  }
  
  fn visit_operand(tree : &Operand, collector : &mut Acc) {
    match tree {
      &Operand::Identifier(ref identifier) => Self::visit_identifier(identifier, collector),
      &Operand::Value(ref value)           => Self::visit_value(value, collector)
    }
  }
 
  // The awkward let _ is required to suppress the unused variables warning
  // https://github.com/rust-lang/rust/issues/26487
  fn visit_identifier(tree : &Identifier, collector : &mut Acc) { let _ = tree; let _ = collector; }
  
  fn visit_value(tree : &Value, collector : &mut Acc) { let _ = tree; let _ = collector; }
}
