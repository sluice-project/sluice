#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Prog<'a> {
  pub globals : Globals<'a>,
  pub packets : Packets<'a>,
  pub snippets : Snippets<'a>,
  pub connections : Connections<'a>
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Globals<'a> {
  pub global_vector : Vec<VariableDecl<'a>>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Packets<'a> {
  pub packet_vector : Vec<Packet<'a>>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Packet<'a> {
  pub packet_id       : Identifier<'a>,
  pub packet_base     : Identifier<'a>,
  pub packet_fields   : PacketFields<'a>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct PacketFields<'a> {
  pub field_vector : Vec<PacketField<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct PacketField<'a> {
  pub identifier     : Identifier<'a>,
  pub var_type       : VarType<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Snippets<'a> {
  pub snippet_vector : Vec<Snippet<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Snippet<'a> {
  pub snippet_id       : Identifier<'a>,
  pub device_id        : Identifier<'a>,
  pub variable_decls   : VariableDecls<'a>,
  pub ifblocks         : IfBlocks<'a>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct IfBlocks<'a> {
  pub ifblock_vector : Vec<IfBlock<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct IfBlock<'a> {
  pub id         : u64,
  pub condtype   : u64,
  pub condition  : Condition<'a>,
  pub statements : Statements<'a>,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Condition<'a> {
//    pub lvalue : LValue<'a>,
    pub expr   : Expr<'a>,
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Connections<'a> {
  pub connection_vector : Vec<Connection<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Connection<'a> {
  pub from_snippet   : Identifier<'a>,
  pub to_snippet     : Identifier<'a>,
  pub variable_pairs : Vec<VariablePair<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct VariablePair<'a> {
  pub from_var : Identifier<'a>,
  pub to_var   : Identifier<'a>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct VariableDecls<'a> {
  pub decl_vector : Vec<VariableDecl<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct VariableDecl<'a> {
  pub identifier     : Identifier<'a>,
  pub initial_values : Vec<Value>,
  pub var_type       : VarType<'a>
}


#[derive(Debug)]
#[derive(PartialEq)]
pub enum TypeQualifier {
  Input,
  Output,
  Persistent,
  Transient,
  Const,
  Field,
  Global,
}

impl Copy for TypeQualifier {}

impl Clone for TypeQualifier {
    fn clone(&self) -> TypeQualifier {
        return *self;
    }
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct VarType<'a> {
  pub var_info : VarInfo<'a>,
  pub type_qualifier : TypeQualifier,
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct BitArray {
  pub bit_width : u64,
  pub var_size  : u64,
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum VarInfo<'a> {
  BitArray(u64, u64),
  Packet(Identifier<'a>),
  // Packet(Identifier<'a>, VariableDecls<'a>),
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Statements<'a> {
  pub stmt_vector : Vec<Statement<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Statement<'a> {
  pub lvalue : LValue<'a>,
  pub expr   : Expr<'a>
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Expr<'a> {
  pub op1         : Operand<'a>,
  pub expr_right : ExprRight<'a>
}

// impl<'a> Clone for Expr<'a> {
//     fn clone(&self) -> Expr<'a> {
//         Expr{op1 : self.op1, expr_right : self.expr_right}
//     }
// }
// Enum of binary operation types
macro_rules! bin_op_type {
  ($($x:ident),*) => {
    #[derive(Debug)]
    #[derive(PartialEq)]
    #[derive(Copy,Clone)]
    pub enum BinOpType {
      $($x,)*
    }
  };
}
bin_op_type!(BooleanAnd, BooleanOr, Plus, Minus, Mul, Div, Modulo, Equal, NotEqual, LTEQOp, GTEQOp, LessThan, GreaterThan);

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum ExprRight<'a> {
  BinOp(BinOpType, Operand<'a>),
  Cond(Operand<'a>, Operand<'a>),
  Empty()
}

#[derive(Debug)]
#[derive(PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
  pub id_name : &'a str,
}

impl<'a> Copy for Identifier<'a> {}

impl<'a> Clone for Identifier<'a> {
    fn clone(&self) -> Identifier<'a> {
        Identifier {id_name : self.id_name.clone()}
    }
}

impl<'a> Identifier<'a> {
  pub fn get_str(&self) -> &str{
    return self.id_name;
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Value {
  pub value : u64
}

impl Copy for Value {}

impl Clone for Value {
    fn clone(&self) -> Value {
        Value {value : self.value}
    }
}

impl Value {
  pub fn get_string(&self) -> String {
    return self.value.to_string();
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
//#[derive(Copy, Clone)]
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
#[derive(Clone)]
//#[derive(Copy, Clone)]
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


  // convert lvalues to string vec (eg. a to [a] and b[3] to [b,3])
  pub fn get_string_vec(&self) -> Vec<&str> {
      
      let mut my_vec = Vec::new();

      match self {
        &LValue::Scalar(ref id) => {
            my_vec.push(id.get_str());
            return my_vec;
        },
        &LValue::Array(ref id, ref address) => {
            my_vec.push(id.get_str());
            match **address {
                Operand::LValue(ref lval) => {
                    let mut nex_vec = lval.get_string_vec();
                    for nex_id in nex_vec {
                        my_vec.push(nex_id);
                    }
                },
                _ =>  { }
            }
            return my_vec;
        },
        &LValue::Field(ref id, ref field_name) => {
            my_vec.push(id.get_str());
            my_vec.push(&field_name.get_str());
            return my_vec;
        }
      }
  }
}
