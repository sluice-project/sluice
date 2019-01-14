// Translation from Sluice to p4 for each snippet.
// This works by first constructing a DAG.
//extern crate handlebars;

use grammar::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::path::Path;
use bmv2_gen;
use tofino_gen;

//use handlebars::Handlebars;

const META_HEADER : &str = "mdata";
const TAB : &str = "    ";

#[derive(Debug)]
#[derive(PartialEq)]
pub enum DagNodeType<'a> {
    Decl(&'a VariableDecl<'a>),
    Cond(&'a Expr<'a>),
    Stmt(&'a Statement<'a>),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct DagNode<'a> {
    pub node_type : DagNodeType<'a>,
    pub p4_code : P4Code,
    pub next_nodes : Vec<usize>,
    pub prev_nodes : Vec<usize>,
    pub pre_condition : Option<Statement<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct P4Code {
    pub p4_header : P4Header,
    pub p4_control : String,
    pub p4_actions : String,
    pub p4_commons : String,

}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct P4Header {
    pub meta : String,
    pub meta_init : String,
    pub register : String,
    pub define : String
}

// For now, using a simplistic DAG dc using vectors.
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Dag<'a> {
    pub snippet_id       : &'a str,
    pub device_id        : &'a str,
    pub dag_vector : Vec<DagNode<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct VarDecl<'a> {
  pub id             : String,
  pub var_info       : VarInfo<'a>,
  pub type_qualifier : TypeQualifier,
}

pub fn get_identifiers<'a> (my_operand : &'a Operand<'a>) -> Vec<&'a str> {
    match &my_operand {
        Operand::LValue(ref lval) => {
            let mut nex_vec = lval.get_string_vec();
            return nex_vec;
        },
        _ =>  { return Vec::new(); }
    }

}

pub fn get_indices_lval<'a> (decl_map : &HashMap<&str, usize>, lval : &'a LValue<'a>) -> HashMap<&'a str, usize> {
    let mut my_indices : HashMap<&'a str, usize> = HashMap::new();

    let my_vec_ids = &lval.get_string_vec();
    for my_id in my_vec_ids {
        //println!("{:?} ", my_id);
        let my_option = decl_map.get(my_id);
        match my_option {
            Some(index) => {
                my_indices.insert(my_id, *index);
            }
            None => {}
        }
    }
    return my_indices;
}

pub fn get_pre_condition<'a> (decl_map : &'a HashMap<&'a str, usize>, lval : &'a LValue<'a>) -> Option<&'a usize> {
    let mut my_indices : HashMap<&'a str, usize> = HashMap::new();

    let my_id = &lval.get_string();
    let my_option : Option<&'a usize> = decl_map.get(my_id.as_str());
    return my_option;

}

pub fn get_pre_condition_op<'a> (decl_map : &'a HashMap<&'a str, usize>, op : &'a Operand<'a>) -> Option<&'a usize> {
    match &op {
        Operand::LValue(ref lval) => {
            return get_pre_condition(decl_map, lval);
        },
        _ =>  { return None; }
    }
}


pub fn get_indices_op<'a> (decl_map : &HashMap<&str, usize>, op : &'a Operand<'a>) -> HashMap<&'a str, usize> {
    let empty : HashMap<&str, usize> = HashMap::new();
    match &op {
        Operand::LValue(ref lval) => {
            return get_indices_lval(decl_map, lval);
        },
        _ =>  { return empty; }
    }
}

pub fn get_dag_node<'a>(my_dag : &'a Dag<'a>,index : &usize) ->  Option<&'a DagNode<'a>> {
    let my_dag_option = &my_dag.dag_vector.get(*index);
    return *my_dag_option;

}

pub fn check_clone_condition<'a> (my_pre_condition_dag_option : Option<&'a DagNode<'a>> ) -> Option<Statement<'a>> {
    let mut pre_condition = None;
    let condition_statement_lvalue : LValue;
    let condition_statement_op1 : Operand;
    let condition_statement_exprright : ExprRight;
    let condition_statement_expr : Expr;

    match my_pre_condition_dag_option {
        Some (my_pre_condition_dag) => {
            println!("Condition : {:?}\n",my_pre_condition_dag);
            //let condition_statement_op :
            match &my_pre_condition_dag.node_type {
                DagNodeType::Stmt(my_statement) => {
                    match my_statement.lvalue {
                        LValue::Scalar(ref my_identifier) => {
                            condition_statement_lvalue = LValue::Scalar(Identifier{id_name : my_identifier.id_name});
                            println!("{:?}\n", condition_statement_lvalue);
                            match my_statement.expr.op1 {
                                Operand::LValue(ref lval) => {
                                    match lval {
                                        LValue::Scalar(ref my_identifier2) => {
                                            let my_lval = LValue::Scalar(Identifier{id_name : my_identifier2.id_name});
                                            println!("{:?}\n", my_lval);
                                            match my_statement.expr.expr_right {
                                                ExprRight::BinOp(bin_op_type, ref operand) => {
                                                    match operand {
                                                        Operand::LValue(ref lval2) => {
                                                            match lval2 {
                                                                LValue::Scalar(ref my_identifier3) => {
                                                                    let my_lval2 = LValue::Scalar(Identifier{id_name : my_identifier3.id_name});
                                                                    println!("{:?}\n", bin_op_type);
                                                                    println!("{:?}\n", my_lval2);
                                                                    condition_statement_op1 = Operand::LValue(my_lval);
                                                                    condition_statement_exprright = ExprRight::BinOp(bin_op_type, Operand::LValue(my_lval2));

                                                                    println!("{:?}\n",condition_statement_op1);
                                                                    println!("{:?}\n",condition_statement_exprright);
                                                                    condition_statement_expr = Expr{op1: condition_statement_op1, expr_right:condition_statement_exprright};
                                                                    pre_condition = Some(Statement{lvalue: condition_statement_lvalue, expr : condition_statement_expr });
                                                                }
                                                                _ => {
                                                                    panic!("Condition's Right Expression must be a scalar");
                                                                }
                                                            }
                                                        }
                                                        Operand::Value(ref val) => {

                                                        }
                                                    }
                                                }
                                                _ => {
                                                    panic!("Condition needs to be boolean");
                                                }
                                            }
                                        }
                                        _ => {
                                            panic!("Condition not Supported ->  ");
                                        }
                                    }
                                }
                                _ => {
                                    panic!("Condition not Supported ->  ");
                                }
                            }
                        }
                        _ => {
                            panic!("Condition not Supported ->  ");
                        }
                    }

                }
                _ => {
                    panic!("Condition not Supported ->  ");
                }
            }
        }
        None => {}
    }
    return pre_condition;
}

// Construct the connections between the nodes to form the Dag
// TODO : Make it modular. Curently baffled by how to pass mutable reference of Dag again
pub fn create_connections<'a> (my_snippet: &'a Snippet<'a>, my_dag : &mut Dag<'a>) {
    // A HashMap to keep track of declarations.
    let mut decl_map : HashMap<&str, usize>= HashMap::new();
    //First, process variable decls
    let mut i : usize = 0;
    for my_variable_decl in &my_snippet.variable_decls.decl_vector {
        decl_map.insert(&my_variable_decl.identifier.id_name, i);
        i = i + 1;
    }
    //println!("decl map : {:?}\n ", decl_map);
    // Next, process statements, for now ignoring if block.
    for my_if_block in &my_snippet.ifblocks.ifblock_vector {
        for my_statement in &my_if_block.statements.stmt_vector {
            //println!("decl map : {:?}\n ", decl_map);
            //println!("Processing Statement : {:?}: ",  my_statement );
            let mut my_indices_1 : HashMap<&str, usize>;
            let mut my_indices_2 : HashMap<&str, usize>;
            let mut my_indices_3 : HashMap<&str, usize> = HashMap::new();
            let mut my_indices_4 : HashMap<&str, usize> = HashMap::new();
            let mut my_indices_5 : HashMap<&str, usize> = HashMap::new();
            let mut pre_condition = None;
            let mut pre_condition_vector : usize = 0;
            let mut pre_condition_found = false;
            // Connect based on LValue of statements
            my_indices_1 = get_indices_lval(&decl_map, &my_statement.lvalue);
            // Connect based on the first operand
            my_indices_2 = get_indices_op(&decl_map, &my_statement.expr.op1);
            // Connect based on the rest of operand
            match &my_statement.expr.expr_right {
                ExprRight::BinOp(_btype, op2) => {
                    my_indices_3 = get_indices_op(&decl_map, &op2);
                }
                ExprRight::Cond(op_true, op_false) => {
                    my_indices_4 = get_indices_op(&decl_map, &op_true);
                    my_indices_5 = get_indices_op(&decl_map, &op_false);
                    // Fill in the pre-condition statement
                    let my_option = get_pre_condition_op(&decl_map, &my_statement.expr.op1);
                    match my_option {
                        Some(vector) => {
                            pre_condition_vector = *vector;
                            pre_condition_found  = true;
                        }
                        None => {}
                    }
                }
                ExprRight::Empty() => {
                }
            }
            // Populate next_nodes
            for (my_id_1,p_index_1) in my_indices_1.clone() {
                let my_parent_dag_option = my_dag.dag_vector.get_mut(p_index_1);
                match my_parent_dag_option {
                    Some(mut my_parent_dag_node) => {
                        if !&my_parent_dag_node.next_nodes.contains(&i) {
                            my_parent_dag_node.next_nodes.push(i);
                            //println!("Parent_dag_node: {:?}", my_parent_dag_node);
                        }
                    }
                    None => {}
                }
                decl_map.insert(my_id_1, i);
            }
            for (my_id_2,p_index_2) in my_indices_2.clone() {
                let my_parent_dag_option = my_dag.dag_vector.get_mut(p_index_2);
                match my_parent_dag_option {
                    Some(mut my_parent_dag_node) => {
                        if !&my_parent_dag_node.next_nodes.contains(&i) {
                            my_parent_dag_node.next_nodes.push(i);
                            //println!("Parent_dag_node: {:?}", my_parent_dag_node);
                        }
                    }
                    None => {}
                }
                decl_map.insert(my_id_2, i);
            }
            for (my_id_3,p_index_3) in my_indices_3.clone() {
                let my_parent_dag_option = my_dag.dag_vector.get_mut(p_index_3);
                match my_parent_dag_option {
                    Some(mut my_parent_dag_node) => {
                        if !&my_parent_dag_node.next_nodes.contains(&i) {
                            my_parent_dag_node.next_nodes.push(i);
                            //println!("Parent_dag_node: {:?}", my_parent_dag_node);
                        }
                    }
                    None => {}
                }
                decl_map.insert(my_id_3, i);
            }
            for (my_id_4,p_index_4) in my_indices_4.clone() {
                let my_parent_dag_option = my_dag.dag_vector.get_mut(p_index_4);
                match my_parent_dag_option {
                    Some(mut my_parent_dag_node) => {
                        if !&my_parent_dag_node.next_nodes.contains(&i) {
                            my_parent_dag_node.next_nodes.push(i);
                            //println!("Parent_dag_node: {:?}", my_parent_dag_node);
                        }
                    }
                    None => {}
                }
                decl_map.insert(my_id_4, i);
            }
            for (my_id_5,p_index_5) in my_indices_5.clone() {
                let my_parent_dag_option = my_dag.dag_vector.get_mut(p_index_5);
                match my_parent_dag_option {
                    Some(mut my_parent_dag_node) => {
                        if !&my_parent_dag_node.next_nodes.contains(&i) {
                            my_parent_dag_node.next_nodes.push(i);
                            //println!("Parent_dag_node: {:?}", my_parent_dag_node);
                        }
                    }
                    None => {}
                }
                decl_map.insert(my_id_5, i);
            }
            let condition_statement_lvalue : LValue;
            let condition_statement_op1 : Operand;
            let condition_statement_exprright : ExprRight;
            let condition_statement_expr : Expr;

            {
                // Populate pre-condition if any,
                if pre_condition_found == true {
                    let my_pre_condition_dag_option = my_dag.dag_vector.get(pre_condition_vector);
                    //pre_condition = check_clone_condition(my_pre_condition_dag_option);
                    match my_pre_condition_dag_option {
                        Some (my_pre_condition_dag) => {
                            println!("Condition : {:?}\n",my_pre_condition_dag);
                            //let condition_statement_op :
                            match &my_pre_condition_dag.node_type {
                                DagNodeType::Stmt(my_statement) => {
                                    match my_statement.lvalue {
                                        LValue::Scalar(ref my_identifier) => {
                                            condition_statement_lvalue = LValue::Scalar(Identifier{id_name : my_identifier.id_name});
                                            println!("{:?}\n", condition_statement_lvalue);
                                            match my_statement.expr.op1 {
                                                Operand::LValue(ref lval) => {
                                                    match lval {
                                                        LValue::Scalar(ref my_identifier2) => {
                                                            let my_lval = LValue::Scalar(Identifier{id_name : my_identifier2.id_name});
                                                            println!("{:?}\n", my_lval);
                                                            match my_statement.expr.expr_right {
                                                                ExprRight::BinOp(bin_op_type, ref operand) => {
                                                                    match operand {
                                                                        Operand::LValue(ref lval2) => {
                                                                            match lval2 {
                                                                                LValue::Scalar(ref my_identifier3) => {
                                                                                    let my_lval2 = LValue::Scalar(Identifier{id_name : my_identifier3.id_name});
                                                                                    //println!("{:?}\n", bin_op_type);
                                                                                    //println!("{:?}\n", my_lval2);
                                                                                    condition_statement_op1 = Operand::LValue(my_lval);
                                                                                    condition_statement_exprright = ExprRight::BinOp(bin_op_type, Operand::LValue(my_lval2));

                                                                                    //println!("{:?}\n",condition_statement_op1);
                                                                                    //println!("{:?}\n",condition_statement_exprright);
                                                                                    condition_statement_expr = Expr{op1: condition_statement_op1, expr_right:condition_statement_exprright};
                                                                                    pre_condition = Some(Statement{lvalue: condition_statement_lvalue, expr : condition_statement_expr });
                                                                                }
                                                                                _ => {
                                                                                    panic!("Condition's Right Expression must be a scalar");
                                                                                }
                                                                            }
                                                                        }
                                                                        Operand::Value(ref val) => {

                                                                        }
                                                                    }
                                                                }
                                                                _ => {
                                                                    panic!("Condition needs to be boolean");
                                                                }
                                                            }
                                                        }
                                                        _ => {
                                                            panic!("Condition not Supported ->  ");
                                                        }
                                                    }
                                                }
                                                _ => {
                                                    panic!("Condition not Supported ->  ");
                                                }
                                            }
                                        }
                                        _ => {
                                            panic!("Condition not Supported ->  ");
                                        }
                                    }

                                }
                                _ => {
                                    panic!("Condition not Supported ->  ");
                                }
                            }
                        }
                        None => {}
                    }
                }
            }

            { // Block to end the mutable borrow
                let my_dag_option = my_dag.dag_vector.get_mut(i);
                match my_dag_option {
                    Some(mut my_dag_node) => {
                        // Populate prev_nodes
                        for (_my_id_1,p_index_1) in my_indices_1 {
                            if !&my_dag_node.prev_nodes.contains(&p_index_1) {
                                my_dag_node.prev_nodes.push(p_index_1);
                            }
                        }
                        for (_my_id_2,p_index_2) in my_indices_2 {
                            if !&my_dag_node.prev_nodes.contains(&p_index_2) {
                                my_dag_node.prev_nodes.push(p_index_2);
                            }
                        }
                        for (_my_id_3,p_index_3) in my_indices_3 {
                            if !&my_dag_node.prev_nodes.contains(&p_index_3) {
                                my_dag_node.prev_nodes.push(p_index_3);
                            }
                        }
                        for (_my_id_4,p_index_4) in my_indices_4 {
                            if !&my_dag_node.prev_nodes.contains(&p_index_4) {
                                my_dag_node.prev_nodes.push(p_index_4);
                            }
                        }
                        for (_my_id_5,p_index_5) in my_indices_5 {
                            if !&my_dag_node.prev_nodes.contains(&p_index_5) {
                                my_dag_node.prev_nodes.push(p_index_5);
                            }
                        }
                        my_dag_node.pre_condition = pre_condition;
                    }
                    None => {}
                }
            }
            i = i + 1;
        }
    }
}

pub fn create_dag_nodes<'a> (my_snippets : &'a Snippets) -> HashMap<&'a str, Dag<'a>>  {
    let mut dag_map : HashMap<&str, Dag>= HashMap::new();
    for my_snippet in &my_snippets.snippet_vector {
        //println!("Snippet : {:?}\n", my_snippet.snippet_id.id_name);
        let mut my_dag : Dag = Dag { snippet_id : my_snippet.snippet_id.id_name,
            device_id : my_snippet.device_id.id_name, dag_vector : Vec::new()};
        //let my_dag_start_node : DagNode;

        for my_variable_decl in &my_snippet.variable_decls.decl_vector {

            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
            let my_dag_start_node = DagNode {node_type : DagNodeType::Decl(my_variable_decl),
                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
            //println!("{:?}\n", my_dag_start_node);
            my_dag.dag_vector.push(my_dag_start_node);
        }
        for my_if_block in &my_snippet.ifblocks.ifblock_vector {
            for my_statement in &my_if_block.statements.stmt_vector {
                let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                let dummpyp4 = P4Code{p4_header:dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                let mut my_dag_node = DagNode {node_type: DagNodeType::Stmt(&my_statement),
                    p4_code : dummpyp4, next_nodes: Vec::new(), prev_nodes: Vec::new(), pre_condition : None};
                my_dag.dag_vector.push(my_dag_node);
            }
        }
        dag_map.insert(&my_snippet.snippet_id.id_name, my_dag);
    }
    dag_map
}


//
// pub fn init_handlebars<'a> (dag_map : HashMap<&'a str, Dag<'a>>) {
//     let mut reg = Handlebars::new();
//     reg.set_strict_mode(true);
//
//     // render without register
//     println!("{}", reg.render_template("Hello {{name}}", &json!({"name": "foo"})).unwrap());
//
//     // register template using given name
//     reg.register_template_string("tpl_1", "Good afternoon, {{name}}").unwrap();
//     //reg.register_template_file("tp1_2", "foobar").unwrap();
//     println!("{}", reg.render("tpl_1", &json!({"name": "foo"})).unwrap());
// }

pub fn gen_p4_code<'a> (dag_map : HashMap<&'a str, Dag<'a>>){
    for (snippet_name, snippet_dag) in dag_map {
        let device_type : String = String::from(snippet_dag.device_id);
        if snippet_dag.device_id.contains("bmv2") {
            bmv2_gen::gen_p4_code(&snippet_name, &snippet_dag);
        } else if snippet_dag.device_id.contains("tofino"){
            tofino_gen::gen_p4_code(&snippet_name, &snippet_dag);
        }
    }
}

pub fn trans_snippets<'a> (my_snippets : &Snippets<'a>) {
    // TODO : Deal with mutability of my_dag
    let mut dag_map = create_dag_nodes(&my_snippets);
    //println!("Dag Map: {:?}\n", dag_map);
    for my_snippet in &my_snippets.snippet_vector {
    //for (snippet_name, mut  snippet_dag) in dag_map {
        let mut my_option = dag_map.get_mut(&my_snippet.snippet_id.id_name);
        let device_type : String = String::from(my_snippet.device_id.id_name);
        match my_option {
           Some(mut snippet_dag) => {
                create_connections(&my_snippet, &mut snippet_dag);
                if device_type.contains("bmv2") {
                    bmv2_gen::fill_p4code(&mut snippet_dag);
                } else if device_type.contains("tofino") {
                    tofino_gen::fill_p4code(&mut snippet_dag);
                }
                println!("Snippet DAG: {:?}\n", snippet_dag);

           }
           None => {}
        }
    }
    gen_p4_code(dag_map);
    //tofino_gen::gen_p4_code(dag_map);

    //init_handlebars(dag_map);
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::lexer::get_tokens;
  use super::super::parser::*;

  macro_rules! test_trans_success {
    ($input_code:expr,$trans_snippet_routine:ident,$test_name:ident) => (
      #[test]
      fn $test_name() {
        let input = $input_code;
        let tokens = &mut get_tokens(input);
        let token_iter = &mut tokens.iter().peekable();
        let parse_tree = parse_prog(token_iter);
        $trans_snippet_routine(&parse_tree.snippets);
        assert!(token_iter.peek().is_none(), "token iterator is not empty");
      }
    )
  }

  test_trans_success!(r"global threshold : bit<32> = 111;
                          packet n {}

                          @ bmv2
                          snippet fun(){
                            transient z : bit<1>;
                            transient r : bit<32>;
                            transient q : bit<32>;
                            transient m : bit<32>;
                            transient l : bit<32>;
                            transient i : bit<32>;
                            persistent reg1 : bit<32> = 0;
                            persistent reg2 : bit<32> = 0;

                            q = 10;
                            r = 5;
                            l = r;
                            i = q + l;
                            reg1 = 11;
                            reg2 = i;
                            z = q > r;
                            m = z? 5 : 10;
                          }
                        ", trans_snippets, test_trans_snippets);
 }
