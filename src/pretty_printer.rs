use grammar::*;
use tree_fold::TreeFold;

pub struct PrettyPrinter {
  pretty_print_str : String,
}

impl PrettyPrinter {
  pub fn new() -> PrettyPrinter {
    PrettyPrinter{ pretty_print_str : "".to_string() }
  }
}

impl<'a> TreeFold<'a> for PrettyPrinter {
  fn visit_snippet(&mut self, tree : &'a Snippet) {
    self.pretty_print_str.push_str("snippet ");
    self.pretty_print_str.push_str(tree.snippet_id.get_str());
    self.pretty_print_str.push_str("() {");
    self.visit_variable_decls(&tree.variable_decls);
    self.visit_ifblocks(&tree.ifblocks);
    self.pretty_print_str.push_str("}");
  }

  fn visit_connection(&mut self, tree : &'a Connection) {
    self.pretty_print_str.push_str("(");
    self.pretty_print_str.push_str(tree.from_snippet.get_str());
    self.pretty_print_str.push_str(",");
    self.pretty_print_str.push_str(tree.to_snippet.get_str());
    if !tree.variable_pairs.is_empty() {
      self.pretty_print_str.push_str(":");
      for pair in &tree.variable_pairs {
        self.pretty_print_str.push_str("(");
        self.pretty_print_str.push_str(pair.from_var.get_str());
        self.pretty_print_str.push_str("->");
        self.pretty_print_str.push_str(pair.to_var.get_str());
        self.pretty_print_str.push_str("),");
      }
    }
    self.pretty_print_str.push_str(")");
  }

  fn visit_statement(&mut self, tree : &'a Statement) {
    self.pretty_print_str.push_str(&tree.lvalue.get_string());
    self.pretty_print_str.push_str(" = ");
    self.visit_expr(&tree.expr);
    self.pretty_print_str.push_str(";");
  }

  fn visit_expr(&mut self, tree : &'a Expr) {
    self.pretty_print_str.push_str(&tree.op1.get_string());
    match &tree.expr_right {
      ExprRight::Empty() => {},
      ExprRight::BinOp(btype, op2) => {
        match btype {
          BinOpType::BooleanAnd => self.pretty_print_str.push_str(" and "),
          BinOpType::BooleanOr  => self.pretty_print_str.push_str(" or "),
          BinOpType::Plus       => self.pretty_print_str.push_str(" + "),
          BinOpType::Minus      => self.pretty_print_str.push_str(" - "),
          BinOpType::Mul        => self.pretty_print_str.push_str(" * "),
          BinOpType::Div        => self.pretty_print_str.push_str(" / "),
          BinOpType::Modulo     => self.pretty_print_str.push_str(" % "),
          BinOpType::Equal      => self.pretty_print_str.push_str(" = "),
          BinOpType::NotEqual   => self.pretty_print_str.push_str(" != "),
          BinOpType::LTEQOp     => self.pretty_print_str.push_str(" <= "),
          BinOpType::GTEQOp     => self.pretty_print_str.push_str(" >= "),
          BinOpType::LessThan   => self.pretty_print_str.push_str(" < "),
          BinOpType::GreaterThan=> self.pretty_print_str.push_str(" > "),
        };
        self.pretty_print_str.push_str(&op2.get_string());
      },
      ExprRight::Cond(op_true, op_false) => {
        self.pretty_print_str.push_str(" ? ");
        self.pretty_print_str.push_str(&op_true.get_string());
        self.pretty_print_str.push_str(" : ");
        self.pretty_print_str.push_str(&op_false.get_string());
      }
    }
  }

  fn visit_variable_decl(&mut self, tree : &'a VariableDecl) {
    self.pretty_print_str.push_str(
      match tree.var_type.type_qualifier {
        TypeQualifier::Const => "const",
        TypeQualifier::Persistent => "persistent",
        TypeQualifier::Input => "input",
        TypeQualifier::Output => "output",
        TypeQualifier::Transient => "transient",
        TypeQualifier::Field => "field",
      });
    self.pretty_print_str.push_str(" ");
    self.pretty_print_str.push_str(tree.identifier.get_str());
    self.pretty_print_str.push_str(" : bit<");

    let varinfo =  &tree.var_type.var_info;
    match varinfo {
      VarInfo::BitArray(bit_width, var_size) => {
        self.pretty_print_str.push_str(&bit_width.to_string());
        self.pretty_print_str.push_str(">[");
        self.pretty_print_str.push_str(&var_size.to_string());
      }

      VarInfo::Packet(_) => {}
      // VarInfo::Packet(_, _) => {}
    }

    self.pretty_print_str.push_str("]");
    if tree.initial_values.is_empty() {
      self.pretty_print_str.push_str(";");
    } else {
      self.pretty_print_str.push_str(" = {");
      for val in &tree.initial_values {
        self.pretty_print_str.push_str(&val.get_string());
        self.pretty_print_str.push_str(", ");
      }
      self.pretty_print_str.push_str("}");
      self.pretty_print_str.push_str(";");
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
    let mut pretty_printer = PrettyPrinter::new();
    pretty_printer.visit_prog(&parse_tree);
    println!("Pretty printed code: {}", pretty_printer.pretty_print_str);

    // Reparse pretty printed code
    let new_tokens = &mut lexer::get_tokens(&pretty_printer.pretty_print_str);
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
