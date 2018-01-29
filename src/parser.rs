#[derive(Debug)]
pub enum Prog<'a> {
  Prog(Snippets<'a>, Connections<'a>)
}

#[derive(Debug)]
pub enum Snippets<'a> {
  Snippets(Vec<Snippet<'a>>),
}

#[derive(Debug)]
pub enum Snippet<'a> {
  Snippet(Identifier<'a>, IdList<'a>, Initializers<'a>, Statements<'a>) 
}

#[derive(Debug)]
pub enum Connections<'a> {
  Connections(Identifier<'a>, Identifier<'a>, Box<Connections<'a>>),
  Empty()
}

#[derive(Debug)]
pub enum IdList<'a> {
  IdList(Vec<Identifier<'a>>),
}

#[derive(Debug)]
pub enum Initializers<'a> {
  Initializers(Vec<Initializer<'a>>),
}

#[derive(Debug)]
pub enum Initializer<'a> {
  Initializer(Identifier<'a>, Value)
}

#[derive(Debug)]
pub enum Statements<'a> {
  Statements(Vec<Statement<'a>>)
}

#[derive(Debug)]
pub enum Statement<'a> {
  Statement(Identifier<'a>, Expr<'a>)
}

#[derive(Debug)]
pub enum Expr<'a> {
  Expr(Operand<'a>, ExprRight<'a>)
}

// XXX: The use ... is ugly, but required.
// This is because BinOpType is generated
// in the parser_impl.rs module by a macro.
use parser_impl::BinOpType;
#[derive(Debug)]
pub enum ExprRight<'a> {
  BinOp(BinOpType, Operand<'a>),
  Cond(Operand<'a>, Operand<'a>),
  Empty()
}

#[derive(Debug)]
pub enum Identifier<'a> {
  Identifier(&'a str),
}

impl<'a> Identifier<'a> {
  pub fn get_string(&self) -> &str{
    let &Identifier::Identifier(s) = self;
    return s;
  }
}

#[derive(Debug)]
pub enum Value {
  Value(u32)
}

impl Value {
  pub fn get_string(&self) -> String {
    let &Value::Value(ref s) = self;
    return s.to_string();
  }
}

#[derive(Debug)]
pub enum Operand<'a> {
  Identifier(Identifier<'a>),
  Value(Value),
}

impl<'a> Operand<'a>{
  pub fn is_id(&self) -> bool {
    match self {
      &Operand::Identifier(_)     => true,
      _                           => false
    }
  }
  pub fn is_val(&self) -> bool { !self.is_id() }
  pub fn get_id(&self) -> &str {
    match self {
      &Operand::Identifier(ref id) => id.get_string(),
      _                            => panic!("Can't call get_id if operand isn't an identifier.") // TODO: Should use assert
    }
  }
  pub fn get_val(&self) -> String {
    match self {
      &Operand::Value(ref val) => val.get_string(),
      _                        => panic!("Can't call get_val if operand isn't a value.") // TODO: Should use assert
    }
  }
}
