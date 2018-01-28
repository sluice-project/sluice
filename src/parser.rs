#[derive(Debug)]
pub enum Prog {
  Prog(Snippets, Connections)
}

#[derive(Debug)]
pub enum Snippets {
  Snippets(Vec<Snippet>),
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

impl Identifier {
  pub fn get_string(&self) -> &String{
    let &Identifier::Identifier(ref s) = self;
    return s;
  }
}

#[derive(Debug)]
pub enum Value {
  Value(String)
}

impl Value {
  pub fn get_string(&self) -> &String {
    let &Value::Value(ref s) = self;
    return s;
  }
}

#[derive(Debug)]
pub enum Operand {
  Identifier(Identifier),
  Value(Value),
}

impl Operand{
  pub fn is_id(&self) -> bool {
    match self {
      &Operand::Identifier(_)     => true,
      _                           => false
    }
  }
  pub fn is_val(&self) -> bool { !self.is_id() }
  pub fn get_id(&self) -> &String {
    match self {
      &Operand::Identifier(ref id) => id.get_string(),
      _                            => panic!("Can't call get_id if operand isn't an identifier.") // TODO: Should use assert
    }
  }
  pub fn get_val(&self) -> &String {
    match self {
      &Operand::Value(ref val) => val.get_string(),
      _                        => panic!("Can't call get_val if operand isn't a value.") // TODO: Should use assert
    }
  }
}
