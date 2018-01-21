#[derive(Debug)]
pub enum Prog {
  Prog(Snippets, Connections)
}

#[derive(Debug)]
pub enum Snippets {
  Snippets(Snippet, Box<Snippets>),
  Empty()
}

#[derive(Debug)]
pub enum Snippet {
  Snippet(Identifier, IdList, Initializers, Statements) 
}

#[derive(Debug)]
pub enum Connections {
  Connections(Identifier, Identifier, Box<Connections>),
  Empty()
}

#[derive(Debug)]
pub enum IdList {
  IdList(Identifier, Box<IdList>),
  Empty()
}

#[derive(Debug)]
pub enum Initializers {
  Initializers(Initializer, Box<Initializers>),
  Empty()
}

#[derive(Debug)]
pub enum Initializer {
  Initializer(Identifier, Value)
}

#[derive(Debug)]
pub enum Statements {
  Statements(Statement, Box<Statements>),
  Empty()
}

#[derive(Debug)]
pub enum Statement {
  Statement(Identifier, Expr)
}

#[derive(Debug)]
pub enum Expr {
  Expr(Operand, ExprRight)
}

// XXX: The use ... is ugly, but required.
// This is because BinOpType is generated
// in the parser_impl.rs module by a macro.
use parser_impl::BinOpType;
#[derive(Debug)]
pub enum ExprRight {
  BinOp(BinOpType, Operand),
  Cond(Operand, Operand),
  Empty()
}

#[derive(Debug)]
pub enum Identifier {
  Identifier(String),
}

#[derive(Debug)]
pub enum Value {
  Value(String)
}

#[derive(Debug)]
pub enum Operand {
  Identifier(String),
  Value(String),
}
