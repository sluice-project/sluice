use super::grammar::*;
use tree_fold::TreeFold;

pub struct PrettyPrinter;

impl<'a> TreeFold<'a, String> for PrettyPrinter {
  fn visit_snippet(tree : &'a Snippet, collector : &mut String) {
    collector.push_str("snippet ");
    collector.push_str(tree.snippet_id.get_str());
    collector.push_str("() {");
    Self::visit_variable_decls(&tree.variable_decls, collector);
    Self::visit_statements(&tree.statements, collector);
    collector.push_str("}");
  }

  fn visit_connection(tree : &'a Connection, collector : &mut String) {
    collector.push_str("(");
    collector.push_str(tree.from_snippet.get_str());
    collector.push_str(",");
    collector.push_str(tree.to_snippet.get_str());
    if !tree.variable_pairs.is_empty() {
      collector.push_str(":");
      for pair in &tree.variable_pairs {
        collector.push_str("(");
        collector.push_str(pair.from_var.get_str());
        collector.push_str("->");
        collector.push_str(pair.to_var.get_str());
        collector.push_str("),");
      }
    }
    collector.push_str(")");
  }

  fn visit_statement(tree : &'a Statement, collector : &mut String) {
    collector.push_str(&tree.lvalue.get_string());
    collector.push_str(" = ");
    Self::visit_expr(&tree.expr, collector);
    collector.push_str(";");
  }

  fn visit_expr(tree : &'a Expr, collector : &mut String) {
    collector.push_str(&tree.op1.get_string());
    match &tree.expr_right {
      ExprRight::Empty() => {},
      ExprRight::BinOp(btype, op2) => {
        match btype {
          BinOpType::BooleanAnd => collector.push_str(" and "),
          BinOpType::BooleanOr  => collector.push_str(" or "),
          BinOpType::Plus       => collector.push_str(" + "),
          BinOpType::Minus      => collector.push_str(" - "),
          BinOpType::Mul        => collector.push_str(" * "),
          BinOpType::Div        => collector.push_str(" / "),
          BinOpType::Modulo     => collector.push_str(" % "),
          BinOpType::Equal      => collector.push_str(" = "),
          BinOpType::NotEqual   => collector.push_str(" != "),
          BinOpType::LTEQOp     => collector.push_str(" <= "),
          BinOpType::GTEQOp     => collector.push_str(" >= "),
          BinOpType::LessThan   => collector.push_str(" < "),
          BinOpType::GreaterThan=> collector.push_str(" > "),
        };
        collector.push_str(&op2.get_string());
      },
      ExprRight::Cond(op_true, op_false) => {
        collector.push_str(" ? ");
        collector.push_str(&op_true.get_string());
        collector.push_str(" : ");
        collector.push_str(&op_false.get_string());
      }
    }
  }

  fn visit_variable_decl(tree : &'a VariableDecl, collector : &mut String) {
    collector.push_str(
      match tree.var_type.type_qualifier {
        TypeQualifier::Const => "const",
        TypeQualifier::Persistent => "persistent",
        TypeQualifier::Input => "input",
        TypeQualifier::Output => "output",
        TypeQualifier::Transient => "transient",
      });
    collector.push_str(" ");
    collector.push_str(tree.identifier.get_str());
    collector.push_str(" : bit<");
    collector.push_str(&tree.var_type.bit_width.to_string());
    collector.push_str(">[");
    collector.push_str(&tree.var_type.var_size.to_string());
    collector.push_str("]");
    if tree.initial_values.is_empty() {
      collector.push_str(";");
    } else {
      collector.push_str(" = {");
      for val in &tree.initial_values {
        collector.push_str(&val.get_string());
        collector.push_str(", ");
      }
      collector.push_str("}");
      collector.push_str(";");
    }
  }
}

#[cfg(test)]
mod tests {
  use super::super::lexer;
  use super::super::parser;
  use super::PrettyPrinter;
  use super::super::tree_fold::TreeFold;
 
  fn run_pretty_printer_and_reparse(input_program : &str) {
    // Lexing
    let tokens = & mut lexer::get_tokens(input_program);

    // parsing
    let token_iter = & mut tokens.iter().peekable();
    let parse_tree = parser::parse_prog(token_iter);
    assert!(token_iter.peek().is_none(), "token_iter is not empty.");
    println!("Parse tree: {:?}\n", parse_tree);

    // Run pretty printer
    let mut pretty_printed_code = String::new();
    PrettyPrinter::visit_prog(&parse_tree, &mut pretty_printed_code);
    println!("Pretty printed code: {}", pretty_printed_code);

    // Reparse pretty printed code
    let new_tokens = &mut lexer::get_tokens(&pretty_printed_code);
    let new_token_iter = &mut new_tokens.iter().peekable();
    let new_parse_tree = parser::parse_prog(new_token_iter);
    assert!(new_token_iter.peek().is_none(), "new_token_iter is not empty.");
    assert!(new_parse_tree == parse_tree, "Old and new parse trees don't match.");
  }

  #[test]
  fn test_pretty_printer(){
    let input_program = r"snippet fun(){
                            input a : bit<2>;
                            input b : bit<2>;
                            input c : bit<2>;
                            input x : bit<2>;
                            input y : bit<2>;
                            transient z : bit<2>;
                            transient r : bit<2>;
                            transient q : bit<2>;
                            transient m : bit<2>;
                            z = a + b;
                            q = x;
                            r = y;
                            m = 5;
                          }
                          snippet foo() {
                            input a : bit<2>;
                            input b : bit<2>;
                            input c : bit<2>;
                            persistent p : bit<2> = 1;
                            persistent m : bit<2>[3] = {1, 2, 3, };
                            transient z : bit<2>;
                            transient h : bit<2>;
                            transient q : bit<2>;
                            q = 5;
                            z[5] = 6;
                            h = z[7];
                            m = 5;
                          }
                          (foo, fun)
                          ";
    run_pretty_printer_and_reparse(input_program);
  }
}
