// Trait for parsing that will be implemented by each
// non-terminal in the grammar. Each implementation of
// this trait can be thought of as a parser combinator.

use std;
use grammar::*;
use token::Token;
use std::iter::Peekable;


type TokenIterator<'a> = Peekable<std::slice::Iter<'a, Token<'a>>>;

// Helper function to consume next token and match it against a specified token
// Throw an error if either:
// 1. token_iter is empty
// 2. the next token does not match
fn match_token<'a>(token_iter : & mut TokenIterator<'a>, expected : Token<'a>, error_msg : &'static str) {
  if !token_iter.peek().is_some() {
    panic!("token_iter is empty. Can't consume next token");
  } else {
    let next_token = token_iter.next().unwrap();
    if *next_token == expected {
      return;
    } else {
      panic!("\nInvalid token: {:?}, expected {:?}.\nError message: {:?}", next_token, expected, error_msg);
    }
  }
}

pub fn parse_prog<'a>(token_iter : &mut TokenIterator<'a>) -> Prog<'a> {
  let globals     = parse_globals(token_iter);
  let packets      = parse_packets(token_iter);
  let snippets    = parse_snippets(token_iter);
  let connections = parse_connections(token_iter);
  return Prog { globals, packets, snippets, connections };
}

fn parse_snippets<'a>(token_iter : &mut TokenIterator<'a>) -> Snippets<'a> {
  // Internal helper function to check if it's a snippet or not
  let is_snippet = |token| { match token { &Token::Snippet => true, _ => false, } };

  let mut snippet_vector = Vec::<Snippet>::new();
  loop {
    if !token_iter.peek().is_some() || !is_snippet(*token_iter.peek().unwrap()) {
      return Snippets{snippet_vector};
    } else {
      let snippet = parse_snippet(token_iter);
      snippet_vector.push(snippet);
    }
  }
}

fn parse_snippet<'a>(token_iter : &mut TokenIterator<'a>) -> Snippet<'a> {
  let mut ifid: u32 = 0;
  match_token(token_iter, Token::Snippet, "Snippet definition must start with the keyword snippet.");
  let snippet_id  = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenLeft, "Snippet argument list must start with a left parenthesis.");
  match_token(token_iter, Token::ParenRight, "Snippet argument list must end with a right parenthesis.");
  match_token(token_iter, Token::BraceLeft, "Snippet body must begin with a left brace.");
  let variable_decls    = parse_variable_decls(token_iter);
  let ifblocks          = parse_ifblocks(token_iter, &mut ifid);
  // let statements        = parse_statements(token_iter);
  // let callstacks         = parse_callstacks(token_iter);
  match_token(token_iter, Token::BraceRight, "Snippet body must end with a right brace.");
  return Snippet{snippet_id, variable_decls, ifblocks};
}

fn parse_connections<'a>(token_iter : &mut TokenIterator<'a>) -> Connections<'a> {
  let mut connection_vector = Vec::<Connection<'a>>::new();
  loop {
    if !token_iter.peek().is_some() {
      return Connections{connection_vector};
    } else {
      connection_vector.push(parse_connection(token_iter));
    }
  }
}

fn parse_connection<'a>(token_iter : &mut TokenIterator<'a>) -> Connection<'a> {
  // Helper function to detect identifiers and colons
  let is_ident = |token| { match token { &Token::Identifier(_) => true, _ => false, } };
  let is_colon = |token| { match token { &Token::Colon => true, _ => false, } };

  match_token(token_iter, Token::ParenLeft, "Connection must start with a left parenthesis.");
  let id1   = parse_identifier(token_iter);
  match_token(token_iter, Token::Comma, "Need a comma between snippets that are being connected.");
  let id2   = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenRight, "Connection must end with a right parenthesis.");
  let mut variable_pairs = Vec::<VariablePair>::new();
  if token_iter.peek().is_some() && is_colon(*token_iter.peek().unwrap()) {
    match_token(token_iter, Token::Colon, "Need a colon before variable pairings");
    loop {
      if !token_iter.peek().is_some() || !is_ident(*token_iter.peek().unwrap()) { break; }
      let from_variable = parse_identifier(token_iter);
      match_token(token_iter, Token::Arrow, "Need an arrow between variables.");
      let to_variable   = parse_identifier(token_iter);
      match_token(token_iter, Token::Comma, "Need a comma separating variable connections.");
      variable_pairs.push(VariablePair { from_var : from_variable, to_var : to_variable });
    }
  }
  return Connection { from_snippet : id1, to_snippet : id2, variable_pairs : variable_pairs };
}

fn parse_variable_decls<'a>(token_iter : &mut TokenIterator<'a>) -> VariableDecls<'a> {
  // Helper function to determine if the keyword starts a declaration
  let is_decl = |token| { match token { &Token::Persistent | &Token::Transient | &Token::Const | &Token::Input | &Token::Output | &Token::Packet => true, _ => false, } };

  let mut decl_vector = Vec::<VariableDecl>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_decl(*token_iter.peek().unwrap())) {
      return VariableDecls{decl_vector}; // return empty decl vector if no vars declared
    } else {
      let variable_decl = parse_variable_decl(token_iter);
      decl_vector.push(variable_decl);
    }
  }
}

fn parse_globals<'a>(token_iter : &mut TokenIterator<'a>) -> Globals<'a> {
  let is_global = |token| { match token { &Token::Global => true, _ => false, } };
  let mut global_vector = Vec::<Global>::new();
  loop {
    //println!("is global={}", is_global(*token_iter.peek().unwrap()));
    if !token_iter.peek().is_some() || !is_global(*token_iter.peek().unwrap()) {
      return Globals{global_vector};
    } else {
      let global = parse_global(token_iter);
      global_vector.push(global);
    }
  }
}

fn parse_global<'a>(token_iter : &mut TokenIterator<'a>) -> Global<'a> {
  match_token(token_iter, Token::Global, "Global declaration should start with keyword global.");
  let identifier = parse_identifier(token_iter);
  let is_assign = |token| { match token { &Token::Assign => true, _ => false, } };
  let initial_values = if is_assign(*token_iter.peek().unwrap())  {
                         match_token(token_iter, Token::Assign, "Must separate identifier and value by an assignment symbol.");
                         parse_initial_values(token_iter)
                       } else {
                         Vec::<Value>::new()
                       };
  match_token(token_iter, Token::SemiColon, "Last token in a declaration must be a semicolon.");

  return Global {identifier, initial_values};
}


fn parse_packets<'a>(token_iter : &mut TokenIterator<'a>) -> Packets<'a> {
  // Internal helper function to check if it's a snippet or not
  let is_packet = |token| { match token { &Token::Packet => true, _ => false, } };

  let mut packet_vector = Vec::<Packet>::new();
  loop {
    if !token_iter.peek().is_some() || !is_packet(*token_iter.peek().unwrap()) {
      return Packets{packet_vector};
    } else {
      let packet = parse_packet(token_iter);
      packet_vector.push(packet);
    }
  }
}


fn parse_packet<'a>(token_iter : &mut TokenIterator<'a>) -> Packet<'a> {
  match_token(token_iter, Token::Packet, "Packet definition must start with the keyword packet");
  let identifier = parse_identifier(token_iter);
  match_token(token_iter, Token::BraceLeft, "Packet body must begin with a left brace.");
  let variable_decls    = parse_variable_decls(token_iter);
  match_token(token_iter, Token::BraceRight, "Packet body must end with a right brace.");
  return Packet {identifier, variable_decls};
}


// fn parse_snippet<'a>(token_iter : &mut TokenIterator<'a>) -> Snippet<'a> {
//   let mut ifid: u32 = 0;
//   match_token(token_iter, Token::Snippet, "Snippet definition must start with the keyword snippet.");
//   let snippet_id  = parse_identifier(token_iter);
//   match_token(token_iter, Token::ParenLeft, "Snippet argument list must start with a left parenthesis.");
//   match_token(token_iter, Token::ParenRight, "Snippet argument list must end with a right parenthesis.");
//   match_token(token_iter, Token::BraceLeft, "Snippet body must begin with a left brace.");
//   let variable_decls    = parse_variable_decls(token_iter);
//   let ifblocks          = parse_ifblocks(token_iter, &mut ifid);
//   // let statements        = parse_statements(token_iter);
//   // let callstacks         = parse_callstacks(token_iter);
//   match_token(token_iter, Token::BraceRight, "Snippet body must end with a right brace.");
//   return Snippet{snippet_id, variable_decls, ifblocks};
// }

fn parse_variable_decl<'a>(token_iter : &mut TokenIterator<'a>) -> VariableDecl<'a> {
  let type_qualifier =  parse_type_qualifier(token_iter);
  let identifier = parse_identifier(token_iter);

  let var_type   = parse_type_annotation(token_iter, type_qualifier);
  let is_assign = |token| { match token { &Token::Assign => true, _ => false, } };
  let initial_values = if is_assign(*token_iter.peek().unwrap())  {
                         match_token(token_iter, Token::Assign, "Must separate identifier and value by an assignment symbol.");
                         parse_initial_values(token_iter)
                       } else {
                         Vec::<Value>::new()
                       };
  // Must end declaration with a semi colon regardless of whether there's an initializer or not.
  match_token(token_iter, Token::SemiColon, "Last token in a declaration must be a semicolon.");

  // Check that the initial values are representable using bit vector of bit_width
  for value in &(initial_values) {
    if value.value > 2_u32.pow(var_type.bit_width) - 1 {
      panic!("Initial value {} is outside the range [0, {}] of {}-bit vector.",
             value.value,
             2_u32.pow(var_type.bit_width) - 1,
             var_type.bit_width);
    }
  }

  // Check that the number of initial values matches up with the type for persistent and const
  // variables alone
  if &var_type.type_qualifier == &TypeQualifier::Const || &var_type.type_qualifier == &TypeQualifier::Persistent {
    if initial_values.len() as u32 != var_type.var_size {
      panic!("Found {} initial values. Need {} initial values for variable {}.",
             initial_values.len(),
             var_type.var_size,
             identifier.id_name);
    }
  }
  return VariableDecl {identifier, initial_values, var_type};
}

fn parse_type_qualifier<'a>(token_iter : &mut TokenIterator<'a>) -> TypeQualifier {
  if token_iter.peek().is_none() {
    panic!("No tokens left to parse in parse_type_qualifier.");
  } else {
    let next_token = token_iter.next().unwrap();
    match *next_token {
      Token::Transient  => TypeQualifier::Transient,
      Token::Persistent => TypeQualifier::Persistent,
      Token::Const      => TypeQualifier::Const,
      Token::Input      => TypeQualifier::Input,
      Token::Output     => TypeQualifier::Output,
      _                 => panic!("Unsupported for now!!!")
    }
  }
}

// Retrieve bit width of bit vector. That's the only type for now.
fn parse_type_annotation<'a>(token_iter : &mut TokenIterator<'a>, type_qualifier : TypeQualifier) -> VarType {

  match_token(token_iter, Token::Colon, "Type annotation must start with a colon.");
  match_token(token_iter, Token::Bit, "Invalid type, bit vectors are the only supported type.");
  match_token(token_iter, Token::LessThan, "Need angular brackets to specify width of bit vector.");
  let bit_width = parse_value(token_iter).value;
  if bit_width > 64 { 
    panic!("Bit width can be at most 64.");
  } else if bit_width < 1 {
    panic!("Bit width must be at least 1.");
  }
  match_token(token_iter, Token::GreaterThan, "Need angular brackets to specify width of bit vector.");

  // Check if it's an array
  if token_iter.peek().is_some() && **token_iter.peek().unwrap() == Token::SquareLeft {
    match_token(token_iter, Token::SquareLeft, "Expected [ here.");
    let var_size = parse_value(token_iter).value;
    match_token(token_iter, Token::SquareRight, "Expected ] here.");
    return VarType { var_size, bit_width, type_qualifier };
  } else {
    return VarType { var_size : 1, bit_width, type_qualifier };
  }
}
//static ifid: u32 = 0;

fn parse_ifblocks<'a>(token_iter : &mut TokenIterator<'a>, ifid :&mut u32) -> IfBlocks<'a> {
  // println!("{:?}", token);
  let is_ifblock   = |token| { match token { &Token::If => true, _ => false } };
  let is_elseblock = |token| { match token { &Token::Else => true, _ => false } };
  let is_ident = |token| { match token { &Token::Identifier(_) => true, _ => false } };
  let mut ifblock_vector = Vec::<IfBlock>::new();
  let mut blocktype: u32;

  loop {
    if is_ifblock(*token_iter.peek().unwrap()) {
      *ifid += 1;
      blocktype = 1;
      let ifblock = parse_ifblock(token_iter, *ifid, blocktype);
      ifblock_vector.push(ifblock);
    } else if is_elseblock(*token_iter.peek().unwrap()) {
      blocktype = 2;
      let ifblock = parse_ifblock(token_iter, *ifid, blocktype);
      ifblock_vector.push(ifblock);
      *ifid += 1;
    } else if is_ident(*token_iter.peek().unwrap()) {
      *ifid += 1;
      blocktype = 3; // this 'if block' type serves as generic statements like q = 5
      let ifblock = parse_ifblock(token_iter, *ifid, blocktype);
      ifblock_vector.push(ifblock);
    } else {
      return IfBlocks{ifblock_vector};
    }
  }
}

fn parse_ifblock<'a>(token_iter : &mut TokenIterator<'a>, id : u32, condtype : u32) -> IfBlock<'a> {
  //let mut contents = String::new();
  // let mut contents = String::from("a=1");
  // let cond_tokens = &mut lexer::get_tokens(&mut contents);
  //
  // let cond_token_iter = &mut cond_tokens.iter().peekable();
  let val = Value {value :1};
  let op1 = Operand::Value(val);
  let expr_right = ExprRight::Empty();
  let expr = Expr{op1, expr_right};
  let dummycondition = Condition{expr};
  //let expr= Expr{ , };
  if condtype == 1 {
      //ifblock
      match_token(token_iter, Token::If, "If Block must start with if statement.");
      match_token(token_iter, Token::ParenLeft, "If Block must begin with a left brace.");
      let condition = parse_condition(token_iter);
      match_token(token_iter, Token::ParenRight, "If Block must end with a right brace.");
      match_token(token_iter, Token::BraceLeft, "If Block must begin with a left brace.");
      let statements = parse_statements(token_iter);
      let callstacks = parse_callstacks(token_iter);
      match_token(token_iter, Token::BraceRight, "If Block must end with a right brace.");
      return IfBlock{id, condtype, condition, statements, callstacks};
  } else if condtype == 2 {
      match_token(token_iter, Token::Else, "Else Block must start with else statement.");
      match_token(token_iter, Token::BraceLeft, "If Block must begin with a left brace.");
      let statements = parse_statements(token_iter);
      let callstacks = parse_callstacks(token_iter);
      let condition = dummycondition;//parse_condition(cond_token_iter);
      match_token(token_iter, Token::BraceRight, "If Block must end with a right brace.");
      return IfBlock{id, condtype, condition, statements, callstacks};
  } else { // generic statements, not if/else
      let statements = parse_statements(token_iter);
      let callstacks = parse_callstacks(token_iter);
      let condition = dummycondition;//parse_condition(cond_token_iter);
      return IfBlock{id, condtype, condition, statements, callstacks};
  }
}

fn parse_statements<'a>(token_iter : &mut TokenIterator<'a>) -> Statements<'a> {
  // Helper function to identify beginning of statements
  let is_ident = |token| { match token { &Token::Identifier(_) => true, _ => false } };
  let is_callstack = |token| { match token { &Token::Call => true, _ => false } };

  let mut stmt_vector = Vec::<Statement>::new();
  loop {
    //println!("is callstack={}", is_callstack(*token_iter.peek().unwrap()));
    // println!("is ident={}", is_ident(*token_iter.peek().unwrap()));

    if is_callstack(*token_iter.peek().unwrap()) {
      return Statements{stmt_vector};
    }
    if !token_iter.peek().is_some() || (!is_ident(*token_iter.peek().unwrap())) {
      return Statements{stmt_vector};
    } else {
      let statement = parse_statement(token_iter);
      stmt_vector.push(statement);
    }
  }
}

fn parse_condition<'a>(token_iter : &mut TokenIterator<'a>) -> Condition<'a> {
  //let lvalue = parse_lvalue(token_iter);
  //match_token(token_iter, Token::Assign, "Must separate identifier and expression by an assignment symbol.");
  let expr       = parse_expr(token_iter);
  //match_token(token_iter, Token::SemiColon, "Last token in a statement must be a semicolon.");
  return Condition{expr};
}

fn parse_statement<'a>(token_iter : &mut TokenIterator<'a>) -> Statement<'a> {
  let lvalue = parse_lvalue(token_iter);
  match_token(token_iter, Token::Assign, "Must separate identifier and expression by an assignment symbol.");
  let expr       = parse_expr(token_iter);
  match_token(token_iter, Token::SemiColon, "Last token in a statement must be a semicolon.");
  return Statement{lvalue, expr};
}

fn parse_callstacks<'a>(token_iter : &mut TokenIterator<'a>) -> CallStacks<'a> {
  // Helper function to identify beginning of statements
  let is_ident = |token| { match token { &Token::Call => true, _ => false } };

  let mut callstack_vector = Vec::<CallStack>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_ident(*token_iter.peek().unwrap())) {
      return CallStacks{callstack_vector};
    } else {
      let callstack = parse_callstack(token_iter);
      callstack_vector.push(callstack);
    }
  }
}

fn parse_callstack<'a>(token_iter : &mut TokenIterator<'a>) -> CallStack<'a> {
  match_token(token_iter, Token::Call, "Missing keyword call");
  let next_snippet = parse_identifier(token_iter);
  match_token(token_iter, Token::ParenLeft,  "Snippet argument list must start with a left  parenthesis.");
  match_token(token_iter, Token::ParenRight, "Snippet argument list must start with a right parenthesis.");

  match_token(token_iter, Token::SemiColon, "Last token in a statement must be a semicolon.");
  return CallStack{next_snippet};
}

fn parse_expr<'a>(token_iter : &mut TokenIterator<'a>) -> Expr<'a> {
  if !token_iter.peek().is_some() {
    panic!("Insufficient tokens in call to parse_expr.");
  }
  let op1        = parse_operand(token_iter);
  let expr_right = parse_expr_right(token_iter);
  return Expr{op1, expr_right};
}

// Macro to generate parser for ExprRight given a list of binary operations
macro_rules! expr_right_parser {
  ($($x:ident),*) => {
    fn parse_expr_right<'a>(token_iter : &mut TokenIterator<'a>) -> ExprRight<'a> {
      // generate is_operator helper function
      let is_operator = |token| { match token { $(&Token::$x|)* &Token::Cond => true, _ => false, } };

      // generate get_bin_op helper function
      fn get_bin_op(t : & Token) -> BinOpType {
        match *t {
          $(Token::$x=>BinOpType::$x,)*
          _ => panic!("Invalid BinOpType")
        }
      }

      // use it in parse implementation
      if !token_iter.peek().is_some() || (!is_operator(*token_iter.peek().unwrap())) {
        return ExprRight::Empty();
      }
      let op_type = token_iter.next().unwrap();
      return match op_type {
        $(e @ & Token::$x       => { let operand   = parse_operand(token_iter); // Must be an operand
                                 ExprRight::BinOp(get_bin_op(e), operand)},)*
        & Token::Cond         => { let operand_true = parse_operand(token_iter); // Must be an operand
                                 match_token(token_iter, Token::Colon, "Colon must separate conditional halves.");
                                 let operand_false = parse_operand(token_iter);
                                 ExprRight::Cond(operand_true, operand_false)},
        _                   => { assert!(false, "Cannot get here!"); ExprRight::Empty()}
      }
    }
  };
}

// generate parser using macro
expr_right_parser!(BooleanAnd, BooleanOr, Plus, Minus, Mul, Div, Modulo, Equal, NotEqual, LTEQOp, GTEQOp, LessThan, GreaterThan);

fn parse_identifier<'a>(token_iter : &mut TokenIterator<'a>) -> Identifier<'a> {
  let identifier_token = token_iter.next().unwrap();
  match identifier_token {
    & Token::Identifier(id_name) => Identifier{id_name},
    _                            => panic!("Invalid token: {:?}, expected Token::Identifier", identifier_token)
  }
}

fn parse_lvalue<'a>(token_iter : &mut TokenIterator<'a>) -> LValue<'a> {
  let lvalue_token = token_iter.next().unwrap();
  let is_square_left = |token| { match token { &Token::SquareLeft => true, _ => false, } };
  match lvalue_token {
    & Token::Identifier(id_name) => {
      if token_iter.peek().is_none() || !is_square_left(token_iter.peek().unwrap()) {
        return LValue::Identifier(Identifier{id_name});
      } else {
        match_token(token_iter, Token::SquareLeft, "Expected [ here.");
        let array_address = parse_operand(token_iter);
        match_token(token_iter, Token::SquareRight, "Expected ] here.");
        return LValue::Array(Identifier{id_name}, Box::new(array_address));
      }
    }
    _                      => panic!("Invalid token: {:?}, expected Token::Identifier", lvalue_token)
  }
}

fn parse_operand<'a>(token_iter : &mut TokenIterator<'a>) -> Operand<'a> {
  match token_iter.peek().unwrap() { // && is required because we are using Peekable iterators
    && Token::Identifier(_) => return Operand::LValue(parse_lvalue(token_iter)),
    && Token::Value(_)      => return Operand::Value(parse_value(token_iter)),
    _                       => panic!("Invalid token: {:?}, expected Token::LValue or Token::Value", token_iter.peek().unwrap())
  }
}

fn parse_initial_values<'a>(token_iter : &mut TokenIterator<'a>) -> Vec<Value> {
  match token_iter.peek().unwrap() {
    && Token::Value(_)  => { let mut singleton_vector = Vec::<Value>::new();
                             singleton_vector.push(parse_value(token_iter));
                             return singleton_vector; },
    && Token::BraceLeft => {
      match_token(token_iter, Token::BraceLeft, "PersistentDecl list must start with a left brace.");
      let value_vector = parse_value_vector(token_iter);
      match_token(token_iter, Token::BraceRight, "PersistentDecl list must end with a right brace.");
      return value_vector;
    },
    _                   => panic!("Invalid token: {:?}, expected Token::Value or Token::BraceLeft", token_iter.peek().unwrap())
  }
}

fn parse_value_vector<'a>(token_iter : &mut TokenIterator<'a>) -> Vec<Value> {
  // Helper function to detect values
  let is_value = |token| { match token { &Token::Value(_) => true, _ => false, } };

  let mut value_vector = Vec::<Value>::new();
  loop {
    if !token_iter.peek().is_some() || (!is_value(*token_iter.peek().unwrap())) {
      return value_vector;
    } else {
      let value = parse_value(token_iter);
      match_token(token_iter, Token::Comma, "Expected comma as separator between values.");
      value_vector.push(value);
    }
  }
}

fn parse_value<'a>(token_iter : &mut TokenIterator<'a>) -> Value {
  let value_token = token_iter.next().unwrap();
  match value_token {
    & Token::Value(value)  => return Value{value},
    _                  => panic!("Invalid token: {:?}, expected Token::Value", value_token)
 }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::lexer::get_tokens;

  // Macro to test that parser parses successfully
  macro_rules! test_parser_success {
    ($input_code:expr,$parser_routine:ident,$test_name:ident) => (
      #[test]
      fn $test_name() {
        let input = $input_code;
        let tokens = &mut get_tokens(input);
        let token_iter = &mut tokens.iter().peekable();
        println!("{:?}", $parser_routine(token_iter));
        assert!(token_iter.peek().is_none(), "token iterator is not empty");
      }
    )
  }

  // Macro to test that parser fails to parse with correct panic message
  macro_rules! test_parser_fail {
    ($input_code:expr,$parser_routine:ident,$test_name:ident,$panic_msg:expr) => (
      #[test]
      #[should_panic(expected=$panic_msg)]
      fn $test_name() {
        let input = $input_code;
        let tokens = &mut get_tokens(input);
        let token_iter = &mut tokens.iter().peekable();
        println!("{:?}", $parser_routine(token_iter));
        assert!(token_iter.peek().is_none(), "token iterator is not empty");
      }
    )
  }

  test_parser_success!(r"5", parse_operand, test_parser_operand);
  test_parser_success!(r"a", parse_operand, test_parse_operand_id);
  test_parser_success!(r"a[5]", parse_operand, test_parser_operand_array);
  test_parser_fail!   (r"5", parse_identifier, test_parse_identifier_fail,
                      "Invalid token: Value(5), expected Token::Identifier");
  test_parser_success!(r"a", parse_identifier, test_parse_identifier_pass);
  test_parser_success!(r"a[5]", parse_lvalue, test_parse_lvalue1);
  test_parser_success!(r"a", parse_lvalue, test_parse_lvalue2);
  test_parser_success!(r"7%5", parse_expr, test_parse_expr);
  test_parser_success!(r"x=6+5;", parse_statement, test_parse_statement);
  test_parser_success!(r"x=6+5;y=7*8;", parse_statements, test_parse_statements);
  test_parser_success!(r"transient x : bit<8>;", parse_variable_decls, test_parse_transient_decls);
  test_parser_success!(r"persistent x : bit<3> = 6; persistent y : bit<3> = 7;",
                       parse_variable_decls, test_parse_persistent_decls);
  test_parser_success!(r"persistent x : bit<3>[4] = {4, 5, 6, 7, }; persistent y: bit<3> = 7;",
                       parse_variable_decls, test_parse_persistent_decls2);
  test_parser_fail!   (r"persistent x : bit<3> ={4, 5, 6, 7};persistent y : bit<3> =7;",
                       parse_variable_decls, test_parse_persistent_decls2_fail,
                       "Invalid token: BraceRight, expected Comma.\nError message: \"Expected comma as separator between values.\"");
  test_parser_fail!   (r"persistent x : bit<2> = 4;", parse_variable_decls,
                       test_parse_persistent_decls_outside_range,
                       "Initial value 4 is outside the range [0, 3] of 2-bit vector.");
  test_parser_fail!   (r"persistent x : bit<0> = 4;", parse_variable_decls,
                       test_parse_persistent_decls_bitwidth0, "Bit width must be at least 1.");
  test_parser_fail!   (r"persistent x : bit<31> = 4;", parse_variable_decls,
                       test_parse_persistent_decls_bitwidth31, "Bit width can be at most 64.");
  test_parser_success!(r"persistent x : bit<64>[4] = {1, 2, 3, 4,};", parse_variable_decls,
                       test_parse_persistent_decls_arrays);
  test_parser_fail!   (r"persistent x : bit<64>[2] = {1, 2, 3,};", parse_variable_decls,
                       test_parse_persistent_decls_arrays_fail,
                       "Found 3 initial values. Need 2 initial values for variable x.");
  test_parser_success!(r"snippet fun() {
                           input a : bit<2>;
                           input b : bit<2>;
                           input c : bit<2>;
                           persistent x : bit<3> = 6;
                           persistent y : bit<3> = 7;
                         }",
                       parse_snippet, test_parse_snippet1);
  test_parser_success!(r"snippet fun() {
                           input a : bit<2>;
                           input b : bit<2>;
                           input c : bit<2>;
                           persistent x : bit<3> = 6;
                           x=y+5;
                        }",
                       parse_snippet, test_parse_snippet2);
  test_parser_success!(r"snippet fun() {
                           input a : bit<2>;
                           input b : bit<2>;
                           input c : bit<2>;
                           persistent x : bit<3> = 6;
                           x=y+5;
                         }
                         snippet fun() {
                           input a : bit<2>;
                           input b : bit<2>;
                           input c : bit<2>;
                           persistent x : bit<3> =6;
                           x=y+5;
                         }",
                       parse_snippets, test_parse_snippets);
  test_parser_success!(r"(foo, fun) (bar, foobar)", parse_connections, test_parse_connections);
  test_parser_success!(r"(foo, fun): a->b, c->x, (bar, foobar)", parse_connections, test_parse_connections2);
  test_parser_success!(r"snippet fun () {
                            input a : bit<2>;
                            input b : bit<2>;
                            input c : bit<2>;
                            input x : bit<2>;
                            input y : bit<2>;
                            persistent x : bit<3> = 0;
                            a = x;
                            b = y;
                            m = 5;
                          }
                          snippet foo() {
                            input a : bit<2>;
                            input b : bit<2>;
                            input c : bit<2>;
                            persistent x : bit<3> = 1;
                            x = 5;
                          }
                          (foo, fun)
                          ", parse_prog, test_parse_prog);
  test_parser_success!(r"snippet fun () {
                            input a : bit<2>;
                            input b : bit<2>;
                            input c : bit<2>;
                            input x : bit<2>;
                            input y : bit<2>;
                            persistent x : bit<3> = 0 ;
                            transient k : bit<5>;
                            t1 = a >= b;
                            a = t1 ? x : a;
                            b = t1 ? y : b;
                            t2 = c >= d;
                            t3 = t2 and t1;
                            e = t2 ? m : 5;
                          }
                          snippet foo() {
                            input a : bit<2>;
                            input b : bit<2>;
                            input c : bit<2>;
                            persistent x : bit<3> = 1;
                            x = 5;
                          }
                          (foo, fun)
                          ", parse_prog, test_parse_prog2);
}
