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
  Connections(Vec<(Connection<'a>)>)
}

#[derive(Debug)]
pub struct Connection<'a> {
  pub from_function  : Identifier<'a>,
  pub to_function    : Identifier<'a>,
  pub variable_pairs : Vec<VariablePair<'a>>
}

// TODO: We seem to be mixing up structs and enums.
// Need some convention for this.
#[derive(Debug)]
pub struct VariablePair<'a> {
  pub from_var : Identifier<'a>,
  pub to_var   : Identifier<'a>
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

// Enum of binary operation types
macro_rules! bin_op_type {
  ($($x:ident),*) => {
    #[derive(Debug)]
    pub enum BinOpType {
      $($x,)*
    }
  };
}
bin_op_type!(BooleanAnd, BooleanOr, Plus, Minus, Mul, Div, Modulo, Equal, NotEqual, LTEQOp, GTEQOp, LessThan, GreaterThan);

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
      _                            => panic!("Can't call get_id if operand isn't identifier.") // TODO: Should use assert
    }
  }
  pub fn get_val(&self) -> String {
    match self {
      &Operand::Value(ref val) => val.get_string(),
      _                        => panic!("Can't call get_val if operand isn't a value.") // TODO: Should use assert
    }
  }
}
