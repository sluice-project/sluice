// Translation from Sluice to p4 for each snippet.
// This works by first constructing a DAG.
//extern crate handlebars;
use lexer;
use parser;
use grammar::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::path::Path;
use bmv2_gen;
use tofino_gen;


use std::process;

//use handlebars::Handlebars;

const META_HEADER : &str = "mdata";
const TAB : &str = "    ";
const INCLUDE_DIR : &str = "net-progs/include/";
// natesh edit...removed &'a from VariableDecl
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum DagNodeType<'a> {
    Decl(VariableDecl<'a>),
    Cond(Expr<'a>),
    Stmt(Statement<'a>),
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct DagNode<'a> {
    pub node_type : DagNodeType<'a>,
    pub p4_code : P4Code,
    pub next_nodes : Vec<usize>,
    pub prev_nodes : Vec<usize>,
    pub pre_condition : Option<Statement<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct P4Code {
    pub p4_header : P4Header,
    pub p4_control : String,
    pub p4_actions : String,
    pub p4_commons : String,

}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct P4Header {
    pub meta : String,
    pub meta_init : String,
    pub register : String,
    pub define : String
}

// For now, using a simplistic DAG dc using vectors.
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct Dag<'a> {
    pub snippet_id       : &'a str,
    pub device_id        : &'a str,
    pub dag_vector : Vec<DagNode<'a>>
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub struct VarDecl<'a> {
  pub id             : String,
  pub var_info       : VarInfo<'a>,
  pub type_qualifier : TypeQualifier,
}

// function not used anywhere yet
pub fn get_identifiers<'a> (my_operand : &'a Operand<'a>) -> Vec<&'a str> {
    match &my_operand {
        Operand::LValue(ref lval) => {
            let mut nex_vec = lval.get_string_vec();
            return nex_vec;
        },
        _ =>  { return Vec::new(); }
    }

}

// pub fn create_connections<'a> (my_snippet: &'a Snippet<'a>, my_dag : &mut Dag<'a>) {
pub fn get_indices_lval<'a> (decl_map : &HashMap<String, usize>, lval : LValue<'a>) -> HashMap< String, usize> {

    let mut my_indices : HashMap< String, usize> = HashMap::new();
    let my_vec = lval.get_string_vec().to_owned();
    let my_vec_ids : Vec<&str> = my_vec.iter().cloned().collect() ;

    for my_id in my_vec_ids {
        let my_option = decl_map.get(my_id);
        match my_option {
            Some(index) => {
                my_indices.insert(my_id.to_string(), *index);
            }
            None => {}
        }
    }
    return my_indices;
}

// Getting pre-cond var from decl map leads to wrong condition statement since only the latest dag-vector index
// of the pre-cond variable will be returned i.e. if the pre-cond var has been used after its initial
// assignment, then the wrong pre-condition is populated in the current node being processed...
// May have to store a separate map solely for pre-cond var assignments like "if_block_tmp_2 = q > r"
// for example:                             if_block_tmp_2 is used in the 2nd line so the pre-cond index
//   if_block_tmp_2 = q > r;                for the current node (i = if_block_tmp_2 ? ...) will be this 2nd
//   l = if_block_tmp_2 ? reg3 : l;         line index instead of the 1st line index (1st line indicates the
//   tmp_0_if_2 = q + l;                    actual assignment of the pre-cond variable)
//   i = if_block_tmp_2 ? tmp_0_if_2 : i;

pub fn get_pre_condition<'a> (decl_map : &'a HashMap<String, usize>, lval : LValue<'a>) -> Option<&'a usize> {
    let _my_indices : HashMap<&'a str, usize> = HashMap::new();

    let my_id = &lval.get_string();
    let my_option : Option<&'a usize> = decl_map.get(my_id.as_str());
    return my_option;

}

pub fn get_pre_condition_op<'a> (decl_map : &'a HashMap<String, usize>, op :  Operand<'a>) -> Option<&'a usize> {
    match &op {
        Operand::LValue(ref lval) => {
            return get_pre_condition(decl_map, lval.clone());
        },
        _ =>  { return None; }
    }
}


pub fn get_indices_op<'a> (decl_map : &'a HashMap<String, usize>, op : Operand<'a>) -> HashMap<String, usize> {
    let empty : HashMap<String, usize> = HashMap::new();
    match &op {
        Operand::LValue(ref lval) => {
            return get_indices_lval(decl_map, lval.clone());
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
                                                        Operand::Value(ref _val) => {

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

// Construct the connections between each line of code within a snippet to create dag-vector for that snippet
// TODO : Make it modular. Curently baffled by how to pass mutable reference of Dag again
// TODO : add support for packet fields here and in code gen file
pub fn create_connections<'a> (_my_snippet: &'a Snippet<'a>, my_dag : &mut Dag<'a>) {
    // A HashMap to keep track of declarations.
    let mut decl_map : HashMap<String, usize>= HashMap::new();

    //First, process variable decls
    // after adding in if/else handling, for loop through my_dag.dag_vector and match for variabledecl. Then use that
    // variable decl's id_name since new decls will have been added to dag_vcctor and my_snippet.variable_decls.decl_vector will
    // not contain all of them
    let mut i : usize = 0;

    // not using DagNodeType::Cond for now
    for dagnode in my_dag.dag_vector.clone() {

        match &dagnode.node_type {
            // All vardecls will always be parsed first, before any other lines of code. If/else blocks follow
            DagNodeType::Decl(var_decl) => {
                decl_map.insert(var_decl.identifier.id_name.to_string(), i);
                i += 1;
            }

            DagNodeType::Stmt(my_statement) => {
                let mut my_indices_1 : HashMap<String, usize>;
                let mut my_indices_2 : HashMap<String, usize>;
                let mut my_indices_3 : HashMap<String, usize> = HashMap::new();
                let mut my_indices_4 : HashMap<String, usize> = HashMap::new();
                let mut my_indices_5 : HashMap<String, usize> = HashMap::new();
                let mut pre_condition = None;
                let mut pre_condition_vector : usize = 0;
                let mut pre_condition_found = false;
                // Connect based on LValue of statements
                my_indices_1 = get_indices_lval(&decl_map, my_statement.lvalue.clone());

                // Connect based on the first operand
                my_indices_2 = get_indices_op(&decl_map, my_statement.expr.op1.clone());

                // Connect based on the rest of operand
                match &my_statement.expr.expr_right {
                    ExprRight::BinOp(_btype, op2) => {
                        my_indices_3 = get_indices_op(&decl_map, op2.clone());
                    }
                    ExprRight::Cond(op_true, op_false) => {
                        my_indices_4 = get_indices_op(&decl_map, op_true.clone());
                        my_indices_5 = get_indices_op(&decl_map, op_false.clone());
                        // Fill in the pre-condition statement
                        let my_option = get_pre_condition_op(&decl_map, my_statement.expr.op1.clone());
                        // println!("node : {:?}\n\n", dagnode);
                        // println!("pre-cond : {:?}\n\n", my_option);
                        // println!("decl_map : {:?}\n\n", decl_map);

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

                // let condition_statement_lvalue : LValue;
                // let condition_statement_op1 : Operand;
                // let condition_statement_exprright : ExprRight;
                // let condition_statement_expr : Expr;

                // // TODO :  Currently unable to handle use of same pre-condition var
                // // in multiple statements...see example above
                // {
                //     // Populate pre-condition if any,
                //     if pre_condition_found == true {
                //         println!("current node {:?}\n\n\n", dagnode);
                //         let my_pre_condition_dag_option = my_dag.dag_vector.get(pre_condition_vector);
                //         println!("Condition {:?}\n\n\n", my_pre_condition_dag_option);
                //         //pre_condition = check_clone_condition(my_pre_condition_dag_option);
                //         match my_pre_condition_dag_option {
                //             Some (my_pre_condition_dag) => {
                //                 //let condition_statement_op :
                //                 match &my_pre_condition_dag.node_type {
                //                     DagNodeType::Stmt(my_statement) => {
                //                         match my_statement.lvalue {
                //                             LValue::Scalar(ref my_identifier) => {
                //                                 condition_statement_lvalue = LValue::Scalar(Identifier{id_name : my_identifier.id_name});
                //                                 // println!("{:?}\n", condition_statement_lvalue);
                //                                 match my_statement.expr.op1 {
                //                                     Operand::LValue(ref lval) => {
                //                                         match lval {
                //                                             LValue::Scalar(ref my_identifier2) => {
                //                                                 let my_lval = LValue::Scalar(Identifier{id_name : my_identifier2.id_name});
                //                                                 println!("{:?}\n", my_lval);
                //                                                 // println!("HELLO {:?}",my_statement.expr.expr_right );
                //                                                 match my_statement.expr.expr_right {
                //                                                     ExprRight::BinOp(bin_op_type, ref operand) => {
                //                                                         match operand {
                //                                                             Operand::LValue(ref lval2) => {
                //                                                                 match lval2 {
                //                                                                     LValue::Scalar(ref my_identifier3) => {
                //                                                                         let my_lval2 = LValue::Scalar(Identifier{id_name : my_identifier3.id_name});
                //                                                                         //println!("{:?}\n", bin_op_type);
                //                                                                         //println!("{:?}\n", my_lval2);
                //                                                                         condition_statement_op1 = Operand::LValue(my_lval);
                //                                                                         condition_statement_exprright = ExprRight::BinOp(bin_op_type, Operand::LValue(my_lval2));

                //                                                                         //println!("{:?}\n",condition_statement_op1);
                //                                                                         //println!("{:?}\n",condition_statement_exprright);
                //                                                                         condition_statement_expr = Expr{op1: condition_statement_op1, expr_right:condition_statement_exprright};
                //                                                                         pre_condition = Some(Statement{lvalue: condition_statement_lvalue, expr : condition_statement_expr });
                //                                                                     }
                //                                                                     _ => {
                //                                                                         panic!("Condition's Right Expression must be a scalar");
                //                                                                     }
                //                                                                 }
                //                                                             }
                //                                                             Operand::Value(ref val) => {

                //                                                             }
                //                                                         }
                //                                                     }
                //                                                     _ => {
                //                                                         panic!("Condition needs to be boolean");
                //                                                     }
                //                                                 }
                //                                             }
                //                                             _ => {
                //                                                 panic!("Condition not Supported ->  ");
                //                                             }
                //                                         }
                //                                     }
                //                                     _ => {
                //                                         panic!("Condition not Supported ->  ");
                //                                     }
                //                                 }
                //                             }
                //                             _ => {
                //                                 panic!("Condition not Supported ->  ");
                //                             }
                //                         }

                //                     }
                //                     _ => {
                //                         panic!("Condition not Supported ->  ");
                //                     }
                //                 }
                //             }
                //             None => {}
                //         }
                //     }
                // }

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

                i += 1;

            }
            _ => {}
        }
    }
}

// This func creates the snippet dag and uses Domino's branch removal step to convert if/else statements to single line condtypes
// TODO need to handle packet field nodes
pub fn create_dag_nodes<'a> (my_snippets : &'a Snippets) -> HashMap<&'a str, Dag<'a>>  {

    let mut dag_map : HashMap<&str, Dag>= HashMap::new();

    for my_snippet in &my_snippets.snippet_vector {

        let mut symbol_table : HashMap<&'a str, VarType<'a>> = HashMap::new();
        let mut my_dag : Dag = Dag { snippet_id : my_snippet.snippet_id.id_name,
            device_id : my_snippet.device_id.id_name, dag_vector : Vec::new()};

        for my_variable_decl in &my_snippet.variable_decls.decl_vector {

            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
            let my_dag_start_node = DagNode {node_type : DagNodeType::Decl(my_variable_decl.clone()),
                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
            //println!("{:?}\n", my_dag_start_node);
            my_dag.dag_vector.push(my_dag_start_node);
            // populate symbol table here
            symbol_table.insert(my_variable_decl.identifier.id_name, my_variable_decl.var_type.clone());
        }
        // println!("CHECK  {:?} \n\n\n\n",symbol_table );
        // process::exit(1);
        let mut last_decl_ind : usize = my_dag.dag_vector.len();
        let mut tmp_var_count : usize = 0;

        for my_if_block in &my_snippet.ifblocks.ifblock_vector {

            if my_if_block.condtype == 3 {

                for my_statement in &my_if_block.statements.stmt_vector {
                    let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                    let dummpyp4 = P4Code{p4_header:dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                    let mut my_dag_node = DagNode {node_type: DagNodeType::Stmt(my_statement.clone()),
                        p4_code : dummpyp4, next_nodes: Vec::new(), prev_nodes: Vec::new(), pre_condition : None};
                    my_dag.dag_vector.push(my_dag_node);
                }

            } else {

                if my_if_block.condtype == 1 {

                    {
                        // need to change variable names so they are more unique and do not conflict with
                        // variable names in other snippets i.e. include snippet_id, device_id in if_var string
                        let if_var =  format!("if_block_tmp_{}", my_if_block.id);
                        let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                        let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                        let if_bit_decl = VariableDecl {identifier : Identifier{id_name : Box::leak(if_var.into_boxed_str()) },
                                    initial_values : Vec::<Value>::new(),
                                    var_type : VarType { var_info : VarInfo::BitArray(1, 1), type_qualifier : TypeQualifier::Transient }};

                        let mut if_bit_node = DagNode {node_type : DagNodeType::Decl(if_bit_decl.clone()),
                            p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};

                        my_dag.dag_vector.insert(last_decl_ind, if_bit_node);
                        last_decl_ind += 1;
                    }

                    // adds node for statement of setting if_bit to condition expression
                    {
                        let if_var =  format!("if_block_tmp_{}", my_if_block.id);
                        let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                        let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                        let if_bit_stmt = Statement {
                                            lvalue : LValue::Scalar(Identifier { id_name : Box::leak(if_var.into_boxed_str()) }),
                                            expr : my_if_block.condition.expr.clone()};

                        let mut if_bit_node = DagNode {node_type : DagNodeType::Stmt(if_bit_stmt.clone()),
                            p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};

                        my_dag.dag_vector.push(if_bit_node);
                    }
                }


                for my_statement in &my_if_block.statements.stmt_vector {

                    if my_statement.expr.expr_right != ExprRight::Empty() {
                        // if expr_right exists (Binop or Cond), then create a new var for the RHS of the statement

                        {
                            let tmp_var =  format!("tmp_{}_if_{}", tmp_var_count, my_if_block.id);
                            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                            let mut vinfo : VarInfo = VarInfo::BitArray(1,1); // temp value for vinfo

                            // get varinfo's 2nd index in bitarray from varinfo field of my_statement.expr.lvalue.scalar.id_name variable(match on scalar/
                            // array/packet_field then extract id_name) in symbol table
                            //not handling packet fields for now
                            match my_statement.lvalue {

                                LValue::Scalar(ref id) => {
                                    vinfo = symbol_table.get_mut(id.id_name).unwrap().var_info.clone();
                                }

                                LValue::Array(ref id, _) => {
                                    let a = symbol_table.get_mut(id.id_name).unwrap();
                                    let width =  match a.var_info {
                                                  VarInfo::BitArray(bit_width, _var_size) => bit_width,
                                                  _ => {0}
                                                };
                                    vinfo = VarInfo::BitArray(width, 1);
                                }

                                _ => {}
                            }

                            let tmp_var_decl = VariableDecl {identifier : Identifier{id_name : Box::leak(tmp_var.into_boxed_str()) },
                                        initial_values : Vec::<Value>::new(),
                                        var_type : VarType { var_info : vinfo, type_qualifier : TypeQualifier::Transient }};

                            let mut tmp_node = DagNode {node_type : DagNodeType::Decl(tmp_var_decl.clone()),
                                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};

                            my_dag.dag_vector.insert(last_decl_ind, tmp_node);
                        }

                        {
                            let tmp_var =  format!("tmp_{}_if_{}", tmp_var_count, my_if_block.id);
                            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                            let tmp_stmt = Statement {
                                                lvalue : LValue::Scalar(Identifier { id_name : Box::leak(tmp_var.into_boxed_str()) }),
                                                expr : my_statement.expr.clone()};

                            let mut tmp_node = DagNode {node_type : DagNodeType::Stmt(tmp_stmt.clone()),
                                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};

                            my_dag.dag_vector.push(tmp_node);
                        }

                        // add node to set variable conditional on if bit
                        {

                            let tmp_var =  format!("tmp_{}_if_{}", tmp_var_count, my_if_block.id);
                            let mut tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: ""})),
                                                expr_right: ExprRight::Cond(Operand::LValue(LValue::Scalar(Identifier{id_name: ""})),
                                                Operand::LValue(LValue::Scalar(Identifier{id_name: ""})))};

                            if my_if_block.condtype == 1 {
                                let if_var =  format!("if_block_tmp_{}", my_if_block.id);
                                tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(if_var.into_boxed_str()),})),
                                                expr_right: ExprRight::Cond(Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(tmp_var.into_boxed_str()),})),
                                                Operand::LValue(my_statement.lvalue.clone())) };

                            } else if my_if_block.condtype == 2 {
                                let if_var =  format!("if_block_tmp_{}", my_if_block.id - 1); // for else condition, use the previous
                                                                                             // if block's condition bit var to set
                                                                                             // else statements
                                // For else statement, switch op1 and op2 in cond
                                tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(if_var.into_boxed_str()),})),
                                                expr_right: ExprRight::Cond(Operand::LValue(my_statement.lvalue.clone()),
                                                Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(tmp_var.into_boxed_str()),}))) };
                            }

                            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                            let tmp_stmt = Statement {
                                                lvalue : my_statement.lvalue.clone(),
                                                expr : tmp_expr};

                            let mut tmp_node = DagNode {node_type : DagNodeType::Stmt(tmp_stmt.clone()),
                                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
                            my_dag.dag_vector.push(tmp_node);

                        }

                        last_decl_ind += 1;
                        tmp_var_count += 1;


                    } else {

                        let mut tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: ""})),
                                            expr_right: ExprRight::Cond(Operand::LValue(LValue::Scalar(Identifier{id_name: ""})),
                                            Operand::LValue(LValue::Scalar(Identifier{id_name: ""})))};

                        if my_if_block.condtype == 1 {
                            let if_var =  format!("if_block_tmp_{}", my_if_block.id);
                            tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(if_var.into_boxed_str()),})),
                                            expr_right: ExprRight::Cond(my_statement.expr.op1.clone(),
                                            Operand::LValue(my_statement.lvalue.clone())) };

                        } else if my_if_block.condtype == 2 {
                            let if_var =  format!("if_block_tmp_{}", my_if_block.id - 1);
                            tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(if_var.into_boxed_str()),})),
                                            expr_right: ExprRight::Cond(Operand::LValue(my_statement.lvalue.clone()),
                                             my_statement.expr.op1.clone()) };
                        }

                        let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                        let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};

                        let tmp_stmt = Statement {
                                            lvalue : my_statement.lvalue.clone(),
                                            expr : tmp_expr};

                        let mut tmp_node = DagNode {node_type : DagNodeType::Stmt(tmp_stmt.clone()),
                            p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};

                        my_dag.dag_vector.push(tmp_node);
                    }
                }

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

pub fn gen_p4_code<'a> (my_packets : &Packets<'a>, dag_map : HashMap<&'a str, Dag<'a>>){
    for (snippet_name, snippet_dag) in dag_map {

        if snippet_dag.device_id.contains("bmv2") {
            bmv2_gen::gen_p4_code(&snippet_name, my_packets, &snippet_dag);
        } else if snippet_dag.device_id.contains("tofino"){
            tofino_gen::gen_p4_code(&snippet_name, my_packets,  &snippet_dag);
        }
    }
}
pub fn create_import_map<'a> (my_imports : &Imports<'a>) ->HashMap<String, String>  {
    let mut import_map : HashMap<String, String>= HashMap::new();
    for my_import in &my_imports.import_vector {
        let import_file = format!("{}{}.np", INCLUDE_DIR, my_import.import_id.id_name);
        println!("Importing {}\n", import_file);
        let mut f = File::open(import_file).expect("File not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("Something went wrong reading the file");
        let tokens = & mut lexer::get_tokens(&contents);
        let token_iter = & mut tokens.iter().peekable();
        let dev_tree = parser::parse_device(token_iter);
        for my_dev_field in  dev_tree.device_fields.field_vector {
            let field_name  = format!("{}.{}", dev_tree.device_id.id_name.clone(), my_dev_field.identifier.id_name.clone());
            let identifier = format!("{}", my_dev_field.identifier.id_name.clone());
            import_map.insert(field_name, identifier);
            //import_map.push()
        }
    }
    println!("Import Map:{:?}\n", import_map);
    return import_map;
}

pub fn create_packet_map<'a> (my_packets : &Packets<'a>) ->HashMap<String, String>  {
    let mut packet_map : HashMap<String, String>= HashMap::new();
    for my_packet in &my_packets.packet_vector {
        println!("my Packet : {:?}\n", my_packet);
        let packet_file = format!("{}packet.np", INCLUDE_DIR);
        println!("Importing {}\n", packet_file);
        let mut f = File::open(packet_file).expect("File not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("Something went wrong reading the file");
        let tokens = & mut lexer::get_tokens(&contents);
        let token_iter = & mut tokens.iter().peekable();
        let pkt_tree = parser::parse_import_packets(token_iter);
        for my_pkt in  pkt_tree.packet_vector {
            for my_pkt_field in &my_pkt.packet_fields.field_vector {
                let my_id = my_pkt_field.identifier.id_name.clone();
                let field_name  = format!("{}.{}", my_packet.packet_id.id_name.clone(), my_id);
                let identifier = format!("{}.{}", my_pkt.packet_id.id_name.clone(),my_id);
                packet_map.insert(field_name, identifier);
            }
            println!("Packet : {:?}\n", my_pkt);
            // let field_name  = format!("{}.{}", dev_tree.device_id.id_name.clone(), my_dev_field.identifier.id_name.clone());
            // let identifier = format!("{}", my_dev_field.identifier.id_name.clone());
            // import_map.insert(field_name, identifier);
            //import_map.push()
        }
    }
    println!("Packet Map:{:?}\n", packet_map);
    return packet_map;
}

// need to use either 'bmv2' or 'tofino' for device annotation
pub fn trans_snippets<'a> (my_imports : &Imports<'a>, my_packets : &Packets<'a>, my_snippets : &Snippets<'a>) {
    // TODO : Deal with mutability of my_dag
    let mut dag_map = create_dag_nodes(&my_snippets);
    let import_map = create_import_map(my_imports);
    let packet_map = create_packet_map(my_packets);
    println!("\n\n\n Empty Dag Map: {:?}\n\n\n\n", dag_map);

    for my_snippet in &my_snippets.snippet_vector {

        let mut my_option = dag_map.get_mut(&my_snippet.snippet_id.id_name);
        let device_type : String = String::from(my_snippet.device_id.id_name);
        match my_option {
           Some(mut snippet_dag) => {
                create_connections(&my_snippet, &mut snippet_dag);
                // println!("Snippet DAG: {:?}\n", snippet_dag);
                if device_type.contains("bmv2") {
                    bmv2_gen::fill_p4code(&import_map, &packet_map, &mut snippet_dag);
                } else if device_type.contains("tofino") {
                    tofino_gen::fill_p4code(&import_map, &packet_map, &mut snippet_dag);
                }
                // println!("Snippet DAG: {:?}\n", snippet_dag);

           }
           None => {}
        }
    }
    // dag_map now contains p4 code and connection information (next/prev node)
    println!("\n\n\n Filled Dag Map: {:?}\n\n\n\n", dag_map);
    // process::exit(1);
    gen_p4_code(&my_packets, dag_map);
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
        $trans_snippet_routine(&parse_tree.packets, &parse_tree.snippets);
        assert!(token_iter.peek().is_none(), "token iterator is not empty");
      }
    )
  }

  test_trans_success!(r"  @ bmv2
                          snippet fun(){
                            transient z : bit<1>;
                            transient y : bit<1>;
                            z = q > 10;
                            y = p < 20;
                            m = z? 5 : 10;
                          }
                        ", trans_snippets, test_trans_snippets1);
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
                        ", trans_snippets, test_trans_snippets2);


 }
