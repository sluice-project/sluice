#[derive(Debug)]
#[derive(PartialEq)]
pub struct Prog<'a> {
  pub globals : Globals<'a>,
  pub packets : Packets<'a>,
  pub snippets : Snippets<'a>,
  pub connections : Connections<'a>
}


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Globals<'a> {
  pub global_vector : Vec<Global<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Global<'a> {
  pub identifier       : Identifier<'a>,
  pub initial_values   : Vec<Value>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Packets<'a> {
  pub packet_vector : Vec<Packet<'a>>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Packet<'a> {
  pub identifier       : Identifier<'a>,
  pub variable_decls   : VariableDecls<'a>,
}


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Snippets<'a> {
  pub snippet_vector : Vec<Snippet<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Snippet<'a> {
  pub snippet_id       : Identifier<'a>,
  pub variable_decls   : VariableDecls<'a>,
  pub ifblocks         : IfBlocks<'a>,
  // pub statements       : Statements<'a>,
  // pub callstacks       : CallStacks<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct IfBlocks<'a> {
  pub ifblock_vector : Vec<IfBlock<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct IfBlock<'a> {
  pub id         : u32,
  pub condtype   : u32,
  pub condition  : Condition<'a>,
  pub statements : Statements<'a>,
  pub callstacks : CallStacks<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Condition<'a> {
//    pub lvalue : LValue<'a>,
    pub expr   : Expr<'a>,
}


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Connections<'a> {
  pub connection_vector : Vec<Connection<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Connection<'a> {
  pub from_snippet   : Identifier<'a>,
  pub to_snippet     : Identifier<'a>,
  pub variable_pairs : Vec<VariablePair<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct VariablePair<'a> {
  pub from_var : Identifier<'a>,
  pub to_var   : Identifier<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct VariableDecls<'a> {
  pub decl_vector : Vec<VariableDecl<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum TypeQualifier {
  Input,
  Output,
  Persistent,
  Transient,
  Const,
}

// #[derive(Debug)]
// #[derive(PartialEq)]
// pub struct VarType {
//   pub bit_width : u32,
//   pub var_size  : u32,
//   pub type_qualifier : TypeQualifier,
//   // var_size 1 is a scalar, > 1 is an array.
// }


#[derive(Debug)]
#[derive(PartialEq)]
pub struct VarType<'a> {
  pub bit_width : u32,
  pub var_size  : u32,
  pub type_qualifier : TypeQualifier,
  pub packet_name : Identifier<'a>,
  // var_size 1 is a scalar, > 1 is an array.
}

// #[derive(Debug)]
// #[derive(PartialEq)]
// pub enum VarType<'a> {
//   Scalar(Identifier<'a>),
//   Array(Identifier<'a>, Box<Operand<'a>>),
//   Field(Identifier<'a>, Identifier<'a>)
// }

// #[derive(Debug)]
// #[derive(PartialEq)]
// pub enum LValue<'a> {
//   Scalar(Identifier<'a>),
//   Array(Identifier<'a>, Box<Operand<'a>>),
//   Field(Identifier<'a>, Identifier<'a>)
// // }

// impl<'a> LValue<'a> {
//   pub fn get_string(&self) -> String {
//     match self {
//       &LValue::Scalar(ref id) => id.get_str().to_owned(),
//       &LValue::Array(ref id, ref address) => {
//         id.get_str().to_owned() + " [ " + &address.get_string() + " ] "
//       },
//       &LValue::Field(ref id, ref field_name) => {
//         id.get_str().to_owned() + " . " + &field_name.get_str()
//       }
//     }
//   }
// }

// #[derive(Debug)]
// #[derive(PartialEq)]
// pub struct PacketType<'a> {
//     pub identifier     : Identifier<'a>
//   // pub var_size  : u32,
//   // pub type_qualifier : TypeQualifier,
//   // var_size 1 is a scalar, > 1 is an array.
// }


#[derive(Debug)]
#[derive(PartialEq)]
pub struct VariableDecl<'a> {
  pub identifier     : Identifier<'a>,
  pub initial_values : Vec<Value>,
  pub var_type       : VarType<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Statements<'a> {
  pub stmt_vector : Vec<Statement<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Statement<'a> {
  pub lvalue : LValue<'a>,
  pub expr   : Expr<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct CallStacks<'a> {
  pub callstack_vector : Vec<CallStack<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct CallStack<'a> {
  pub next_snippet : Identifier<'a>,
//  pub condition    : Expr<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Expr<'a> {
  pub op1         : Operand<'a>,
  pub expr_right : ExprRight<'a>
}

// Enum of binary operation types
macro_rules! bin_op_type {
  ($($x:ident),*) => {
    #[derive(Debug)]
    #[derive(PartialEq)]
    pub enum BinOpType {
      $($x,)*
    }
  };
}
bin_op_type!(BooleanAnd, BooleanOr, Plus, Minus, Mul, Div, Modulo, Equal, NotEqual, LTEQOp, GTEQOp, LessThan, GreaterThan);

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ExprRight<'a> {
  BinOp(BinOpType, Operand<'a>),
  Cond(Operand<'a>, Operand<'a>),
  Empty()
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Identifier<'a> {
  pub id_name : &'a str,
}

impl<'a> Identifier<'a> {
  pub fn get_str(&self) -> &str{
    return self.id_name;
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Value {
  pub value : u32
}

impl Value {
  pub fn get_string(&self) -> String {
    return self.value.to_string();
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Operand<'a> {
  LValue(LValue<'a>),
  Value(Value),
}

impl<'a> Operand<'a> {
  pub fn is_scalar(&self) -> bool {
    match self {
      &Operand::LValue(LValue::Scalar(_))     => true,
      _                                       => false
    }
  }
  pub fn is_val(&self) -> bool { !self.is_scalar() }
  pub fn get_id(&self) -> &str {
    match self {
      &Operand::LValue(LValue::Scalar(ref id)) => id.get_str(),
      _ =>  { assert!(false, "Can't call get_id if operand isn't identifier.");"" }
    }
  }
  pub fn get_val(&self) -> String {
    match self {
      &Operand::Value(ref val) => val.get_string(),
      _ => { assert!(false, "Can't call get_val if operand isn't a value."); return String::new();}
    }
  }

  pub fn get_string(&self) -> String {
    match self {
      &Operand::Value(ref val) => val.get_string(),
      &Operand::LValue(ref lval) => lval.get_string()
    }
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LValue<'a> {
  Scalar(Identifier<'a>),
  Array(Identifier<'a>, Box<Operand<'a>>),
  Field(Identifier<'a>, Identifier<'a>)
}

impl<'a> LValue<'a> {
  pub fn get_string(&self) -> String {
    match self {
      &LValue::Scalar(ref id) => id.get_str().to_owned(),
      &LValue::Array(ref id, ref address) => {
        id.get_str().to_owned() + " [ " + &address.get_string() + " ] "
      },
      &LValue::Field(ref id, ref field_name) => {
        id.get_str().to_owned() + " . " + &field_name.get_str()
      }
    }
  }
}
