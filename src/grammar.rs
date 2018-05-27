#[derive(Debug)]
pub struct Prog<'a> {
  pub snippets : Snippets<'a>,
  pub connections : Connections<'a>
}

#[derive(Debug)]
pub struct Snippets<'a> {
  pub snippet_vector : Vec<Snippet<'a>>
}

#[derive(Debug)]
pub struct Snippet<'a> {
  pub snippet_id : Identifier<'a>,
  pub arg_list   : IdList<'a>,
  pub persistent_decls : PersistentDecls<'a>,
  pub transient_decls : TransientDecls<'a>,
  pub statements : Statements<'a>
}

#[derive(Debug)]
pub struct Connections<'a> {
  pub connection_vector : Vec<Connection<'a>>
}

#[derive(Debug)]
pub struct Connection<'a> {
  pub from_snippet   : Identifier<'a>,
  pub to_snippet     : Identifier<'a>,
  pub variable_pairs : Vec<VariablePair<'a>>
}

#[derive(Debug)]
pub struct VariablePair<'a> {
  pub from_var : Identifier<'a>,
  pub to_var   : Identifier<'a>
}

#[derive(Debug)]
pub struct IdList<'a> {
  pub id_vector : Vec<Identifier<'a>>
}

#[derive(Debug)]
pub struct PersistentDecls<'a> {
  pub decl_vector : Vec<PersistentDecl<'a>>
}

#[derive(Debug)]
pub struct PersistentDecl<'a> {
  pub identifier : Identifier<'a>,
  pub initial_values : Vec<Value>,
  pub bit_width  : u32
}

#[derive(Debug)]
pub struct TransientDecls<'a> {
  pub decl_vector : Vec<TransientDecl<'a>>
}

#[derive(Debug)]
pub struct TransientDecl<'a> {
  pub identifier : Identifier<'a>,
  pub bit_width  : u32
}

#[derive(Debug)]
pub struct Statements<'a> {
  pub stmt_vector : Vec<Statement<'a>>
}

#[derive(Debug)]
pub struct Statement<'a> {
  pub lvalue : LValue<'a>,
  pub expr   : Expr<'a>
}

#[derive(Debug)]
pub struct Expr<'a> {
  pub op1        : Operand<'a>,
  pub expr_right : ExprRight<'a>
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
pub struct Identifier<'a> {
  pub id_name : &'a str,
}

impl<'a> Identifier<'a> {
  pub fn get_string(&self) -> &str{
    return self.id_name;
  }
}

#[derive(Debug)]
pub struct Value {
  pub value : u32
}

impl Value {
  pub fn get_string(&self) -> String {
    return self.value.to_string();
  }
}

#[derive(Debug)]
pub enum Operand<'a> {
  LValue(LValue<'a>),
  Value(Value),
}

impl<'a> Operand<'a>{
  pub fn is_id(&self) -> bool {
    match self {
      &Operand::LValue(LValue::Identifier(_))     => true,
      _                                           => false
    }
  }
  pub fn is_val(&self) -> bool { !self.is_id() }
  pub fn get_id(&self) -> &str {
    match self {
      &Operand::LValue(LValue::Identifier(ref id)) => id.get_string(),
      _ =>  { assert!(false, "Can't call get_id if operand isn't identifier.");"" }
    }
  }
  pub fn get_val(&self) -> String {
    match self {
      &Operand::Value(ref val) => val.get_string(),
      _ => { assert!(false, "Can't call get_val if operand isn't a value."); return String::new();}
    }
  }
}

#[derive(Debug)]
pub enum LValue<'a> {
  Identifier(Identifier<'a>),
  Array(Identifier<'a>, Box<Operand<'a>>)
}
