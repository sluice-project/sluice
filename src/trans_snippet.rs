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
use std::process::Command;
use std::env;
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
    pub device_type : &'a str,
    pub device_vector : Vec<Identifier<'a>>,
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

    // If lval is a packet field, concat the packet name and field name then search in decl_map
    match lval {
        LValue::Field(ref id, ref field_name) => {
            let a = format!("{}.{}", id.id_name.to_string(), field_name.id_name.to_string());
            let my_option = decl_map.get(&a);
            match my_option {
                Some(index) => {
                    my_indices.insert(a.to_string(), *index);
                    return my_indices;
                }
                None => {}
            }
        }
        _ => {}
    }

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
// TODO : add support for packet fields here and in code gen file. Connections currently not being made for statements
// containing only packet fields and/or values
pub fn create_connections<'a> (_my_snippet: &'a Snippet<'a>, my_packets : &Packets<'a>, 
                                pkt_tree : &Packets<'a>, my_imports : &Imports<'a>, my_dag : &mut Dag<'a>) {
    // A HashMap to keep track of declarations.
    let mut decl_map : HashMap<String, usize>= HashMap::new();

    // find the index where the variable_decl nodes end 
    let mut insert_ind : usize = 0;
    for dagnode in my_dag.dag_vector.clone() {
        match &dagnode.node_type {
            // All vardecls will always be parsed first, before any other lines of code. If/else blocks follow
            DagNodeType::Decl(var_decl) => {
                insert_ind += 1;
            }
            _ => {}
        }
    }

    // First create new nodes for each packet header (packet.np, psa.np, and user-defined packet) 
    // as a Decl(VariableDecl) (with identifier: Identifier { id_name: "pac.header" }, with initial_values: [],
    //  with VarType::Type_Qualifier = field, and VarType::Var_Info = bitarray(len, 1)) 
    // Clone the whole my_dag and insert these new nodes right after the var_decl nodes so they may
    // be treated as var_decls later on. Now all the indicies calculations should remain correct...

    // Create new nodes for each packet header (packet.np, (not psa.np for now), and user-defined packets) and insert them
    // into the dag_vector starting at insert_ind
    for my_packet in &my_packets.packet_vector {

        for field in &my_packet.packet_fields.field_vector {
            let field_name  = format!("{}.{}", my_packet.packet_id.id_name.clone(), field.identifier.id_name.clone());
            
            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
            let header_decl = VariableDecl {identifier : Identifier{id_name : Box::leak(field_name.into_boxed_str()) },
                initial_values : Vec::<Value>::new(), var_type : field.var_type.clone()};
            let packet_decl_node = DagNode {node_type : DagNodeType::Decl(header_decl.clone()),
                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
            my_dag.dag_vector.insert(insert_ind, packet_decl_node);
            insert_ind += 1
        }

        for my_pkt in &pkt_tree.packet_vector {
            for my_pkt_field in &my_pkt.packet_fields.field_vector {

                let my_id = my_pkt_field.identifier.id_name.clone();
                let field_name  = format!("{}.{}{}", my_packet.packet_id.id_name.clone(), my_pkt.packet_id.id_name.clone(), my_id);
                
                let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                let header_decl = VariableDecl {identifier : Identifier{id_name : Box::leak(field_name.into_boxed_str()) },
                    initial_values : Vec::<Value>::new(), var_type : my_pkt_field.var_type.clone()};
                let packet_decl_node = DagNode {node_type : DagNodeType::Decl(header_decl.clone()),
                    p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
                my_dag.dag_vector.insert(insert_ind, packet_decl_node);
                insert_ind += 1
            }
        }
    }

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

    create_dependency_dag(&mut my_dag.clone());
    println!("new nodes{:?}\n\n", my_dag);
    create_offload_header(&mut my_dag.clone());
    process::exit(1);
}


pub fn get_write_var<'a> (decl_map : &HashMap<String, usize>, lval : LValue<'a>) -> usize {

    // If lval is a packet field, concat the packet name and field name then search in decl_map
    match lval {
        LValue::Field(ref id, ref field_name) => {
            let a = format!("{}.{}", id.id_name.to_string(), field_name.id_name.to_string());
            let my_option = decl_map.get(&a);
            match my_option {
                Some(index) => {
                    return *index;
                }
                None => {panic!("write_var not found in decl_map");}
            }
        }

        LValue::Array(name, ind) => {
            let my_option = decl_map.get(name.id_name);
            match my_option {
                Some(index) => {
                    return *index;
                }
                None => {panic!("write_var not found in decl_map");}
            }
        }

        LValue::Scalar(id) => {
            let my_option = decl_map.get(id.id_name);
            match my_option {
                Some(index) => {
                    return *index;
                }
                None => {panic!("write_var not found in decl_map");}
            }
        }
        _ => {{panic!("write_var is not an lvalue");}}
    }
}


pub fn get_array_ind_val<'a> (decl_map : &HashMap<String, usize>, operand :  &Operand<'a>) -> usize {

    match operand {
        Operand::LValue(ref lval) => {
            match lval {
                LValue::Scalar(ref my_id) => {
                    let my_option = decl_map.get(my_id.id_name);
                    match my_option {
                        Some(index) => {
                            return *index;
                        }
                        None => {panic!("read_var not found in decl_map");}
                    }
                }

                LValue::Field(ref id, ref field_name) => {
                    let a = format!("{}.{}", id.id_name.to_string(), field_name.id_name.to_string());
                    let my_option = decl_map.get(&a);
                    match my_option {
                        Some(index) => {
                            return *index;
                        }
                        None => {panic!("read_var not found in decl_map");}
                    }
                }

                _ => {panic!("array index cannot be an array itself");}
            }
        }

        Operand::Value(ref rval_val) => {
            return std::usize::MAX;
        }
    }
}



pub fn get_operand_val<'a> (decl_map : &HashMap<String, usize>, operand :  &Operand<'a>) -> Vec<usize> {

    let mut read_vec = Vec::new();

    match operand {
        Operand::LValue(ref lval) => {
            match lval {
                LValue::Field(ref id, ref field_name) => {
                    let a = format!("{}.{}", id.id_name.to_string(), field_name.id_name.to_string());
                    let my_option = decl_map.get(&a);
                    match my_option {
                        Some(index) => {
                            read_vec.push(*index);
                        }
                        None => {}
                    }
                }
        
                LValue::Array(name, ind) => {
                    let my_option = decl_map.get(name.id_name);
                    match my_option {
                        Some(index) => {
                            read_vec.push(*index);
                        }
                        None => {panic!("write_var not found in decl_map");}
                    }

                    let val = get_array_ind_val(&decl_map, ind);
                    if val != std::usize::MAX {
                        read_vec.push(val);
                    }
                }

                LValue::Scalar(id) => {
                    let my_option = decl_map.get(id.id_name);
                    match my_option {
                        Some(index) => {
                            read_vec.push(*index);
                        }
                        None => {panic!("write_var not found in decl_map");}
                    }
                }

            }

        }

        _ => {}
    }

    return read_vec; 
}


pub fn get_read_vars<'a> (decl_map : &HashMap<String, usize>, my_statement : Statement<'a>) -> Vec<usize> {

    let mut read_vec = Vec::new();
    // If lval is a packet field, concat the packet name and field name then search in decl_map
    match my_statement.lvalue {
        LValue::Array(name, ind) => {
            let index = get_array_ind_val(&decl_map, &ind);
            if index != std::usize::MAX {
                read_vec.push(index);
            }
        }

        _ => {}
    }

    read_vec.append(&mut get_operand_val(&decl_map, &my_statement.expr.op1));

    match my_statement.expr.expr_right {
        ExprRight::BinOp(bin_op_type, ref operand) => {
            read_vec.append(&mut get_operand_val(&decl_map, operand));
        }

        ExprRight::Cond(ref operand1, ref operand2) => {
            read_vec.append(&mut get_operand_val(&decl_map, operand1));
            read_vec.append(&mut get_operand_val(&decl_map, operand2));
        }

        ExprRight::Empty() => {}
    }

    return read_vec;
}



// TODO : create new vardecl nodes for psa.np variables
pub fn create_RAW_connections<'a> (_my_snippet: &'a Snippet<'a>, my_packets : &Packets<'a>, 
                                pkt_tree : &Packets<'a>, my_imports : &Imports<'a>, my_dag : &mut Dag<'a>) {
    // A HashMap to keep track of declarations.
    let mut decl_map : HashMap<String, usize>= HashMap::new();
    let mut i : usize = 0;
    let mut j : usize = 0;
    let mut write_var : usize = 0;
    let mut read_vars : Vec<usize> = Vec::new();

    let mut print_write_var : usize = 0;
    let mut print_read_vars : Vec<usize> = Vec::new();

    // building RAW (read-after-write) dependency dag. Control dependencies were eliminated
    // after converting if/else to single-line cond statements.
    for dagnode in my_dag.dag_vector.clone() {

        match &dagnode.node_type {
            // All vardecls will always be parsed first, before any other lines of code. If/else blocks follow
            DagNodeType::Decl(var_decl) => {
                decl_map.insert(var_decl.identifier.id_name.to_string(), i);
                i += 1;
            }

            DagNodeType::Stmt(my_statement) => {

                print_write_var = get_write_var(&decl_map, my_statement.lvalue.clone());
                println!("write_var {:?} {:?}", i, my_dag.dag_vector.get_mut(print_write_var));
                println!("");

                print_read_vars = get_read_vars(&decl_map, my_statement.clone());
                for r in print_read_vars {
                    println!("read_var {:?} {:?}", r, my_dag.dag_vector.get_mut(r));
                    println!("");
                }

                j = i + 1;
                write_var = get_write_var(&decl_map, my_statement.lvalue.clone());
                
                while j < my_dag.dag_vector.len() {

                    let next_statement = my_dag.dag_vector.get_mut(j).unwrap();
                    // println!("AHHH {:?}", next_statement);
                    // process::exit(1);
                    match &next_statement.node_type {
                        DagNodeType::Stmt(stmt) => {
                            read_vars = get_read_vars(&decl_map, stmt.clone());
                        }
                        _ => {}
                    }
                    
                    for r in read_vars.clone() {
                        if r == write_var {
                            let my_parent_dag_option = my_dag.dag_vector.get_mut(i);
                            match my_parent_dag_option {
                                Some(mut my_parent_dag_node) => {
                                    if !&my_parent_dag_node.next_nodes.contains(&j) {
                                        my_parent_dag_node.next_nodes.push(j);
                                    }
                                }
                                None => {}
                            }                            
                        }
                    }

                    j += 1;
                }

                i += 1;
            }

            _ => {}
        }
    }

    create_dependency_dag(&mut my_dag.clone());
    println!("new nodes{:?}\n\n", my_dag);
    create_offload_header(&mut my_dag.clone());
    process::exit(1);
}



pub fn create_offload_header<'a> (my_dag : &mut Dag<'a>) {

    let mut offload_header = Vec::<DagNode>::new();
    let mut var_map : HashMap<String, Vec<String>> = HashMap::new();
    let mut color = Vec::new();
    let mut k : usize = 0;
    let mut i = 0;

    for dagnode in my_dag.dag_vector.clone() {
        match &dagnode.node_type {
            DagNodeType::Decl(var_decl) => {
                color.insert(k, 2);
                k += 1;
                i += 1;   
            }
            DagNodeType::Stmt(my_statement) => {
                color.insert(k, 0);
                k += 1;
            }

            _ => {}
        }
    }

    // DFS is used to traverse the dependency DAG in topological order.
    // DFS algorithm : white = 0, gray = 1, black = 2
    let mut accum : usize = 0;

    while i < color.len() {
        // let c =  color[i];
        if(color[i] == 0) {
            println!("HEYYYYYY");
            DFS_visit(my_dag.dag_vector.clone(), i, &mut color, &mut offload_header);
            // println!("{:?}", i);
        }
        println!("YOOOO {:?}", i);
        i += 1;  
    }
}


pub fn DFS_visit<'a> (G : Vec<DagNode>, i : usize, color : &mut Vec<usize>, offload_header : &mut Vec<DagNode> ) {
    println!("{:?}", color.get_mut(i).unwrap().to_string());
    color[i] = 1;
    println!("{:?}", color.get_mut(i).unwrap().to_string());
    println!("{:?}", G.get(i).unwrap());
    println!();
    // var_map[] = ;

    let mut accum : usize = 1;
    for v in G.get(i).unwrap().next_nodes.clone() {
        // let c = color[v];
        if(color[v] == 0) {
            DFS_visit(G.clone(), v,  color,  offload_header); 

        }
    }
    color[i] = 2;
}



pub fn create_dependency_dag<'a> (my_dag : &mut Dag<'a>) {

    let out_f : String = format!("plots/dependency_dag.txt");
    let path = Path::new(out_f.as_str());
    let display  = path.display();
    let mut out_file1 = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(out_file1) => out_file1,
    };

    let out_f : String = format!("plots/next_prev_nodes.txt");
    let path = Path::new(out_f.as_str());
    let display  = path.display();
    let mut out_file2 = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(out_file2) => out_file2,
    };

    let mut contents = String::new();
    let mut next_prev_nodes = String::new();

    for dagnode in my_dag.dag_vector.clone() {
        match &dagnode.node_type {

            DagNodeType::Decl(var_decl) => {
                contents += &format!("{:?} {:?} : {:?}", var_decl.var_type.type_qualifier, 
                    var_decl.identifier.id_name, var_decl.var_type.var_info);
                next_prev_nodes += &format!("{:?}:{:?}\n", dagnode.next_nodes, dagnode.prev_nodes);

                if(!var_decl.initial_values.is_empty()) {
                    contents += &format!("[");
                    for i in var_decl.initial_values.clone() {
                        contents += &format!("{:?},", i.value);
                    }
                    contents += &format!("]");
                }
                contents +=  &format!("\n");
            }

            DagNodeType::Stmt(stmt) => {
                
                match &stmt.lvalue {
                    LValue::Scalar(ref id) => {
                        contents += &format!("{:?} = ", id.id_name);
                    }

                    LValue::Array(id1,  id2) => {
                        contents += &format!("{:?}[{:?}] = ", id1.id_name, handle_array_op(id2));
                    }

                    LValue::Field(ref p, ref f) => {
                        contents += &format!("{:?}.{:?} = ", p.id_name, f.id_name);
                    }
                }

                contents += &handle_operand(&stmt.expr.op1);

                match &stmt.expr.expr_right {
                    ExprRight::BinOp(bin_op_type, ref operand) => {
                        contents += &handle_binop(*bin_op_type);
                        contents += &handle_operand(operand);                    
                    }

                    ExprRight::Cond(ref operand1, ref operand2) => {
                        contents += &format!(" ? ");
                        contents += &handle_operand(operand1);
                        contents += &format!(" : ");
                        contents += &handle_operand(operand2);
                    }

                    ExprRight::Empty() => {}
                }

                contents += &format!("\n");
                next_prev_nodes += &format!("{:?}:{:?}\n", dagnode.next_nodes, dagnode.prev_nodes);
            }

            _ => {}
        }
    }

    let clean_contents : String = contents.replace('"', "").replace('\\',"");
    out_file1.write(clean_contents.as_bytes());
    out_file2.write(next_prev_nodes.as_bytes());

    let mut root = Path::new("plots");
    assert!(env::set_current_dir(&root).is_ok());
    let _output = Command::new("python3")
            .arg("gen_dependency_dag.py")
            .output()
            .expect("failed to execute process");

    root = Path::new("..");
    assert!(env::set_current_dir(&root).is_ok());

    // println!("new nodes{:?}", my_dag);
    // process::exit(1);
}



pub fn handle_binop<'a> (bin_op_type : BinOpType) -> String {
    let mut contents = String::new();
    match bin_op_type {
        BinOpType::BooleanAnd => {
            contents = " & ".to_string();
        }
        BinOpType::BooleanOr => {
            contents += " | ";
        }
        BinOpType::ShiftLeft => {
            contents += " << ";
        }
        BinOpType::ShiftRight => {
            contents += " >> ";
        }
        BinOpType::Plus => {
            contents += " + ";
        }
        BinOpType::Minus => {
            contents += " - ";
        }
        BinOpType::Mul => {
            contents += " * ";
        }
        BinOpType::Div => {
            contents += " / ";
        }
        BinOpType::Modulo => {
            contents += " % ";
        } 
        BinOpType::Equal => {
            contents += " == ";
        }
        BinOpType::NotEqual => {
            contents += " != ";
        }
        BinOpType::GreaterThan => {
            contents += " > ";
        }
        BinOpType::LessThan => {
            contents += " < ";
        }
        BinOpType::GTEQOp => {
            contents += " >= ";
        }
        BinOpType::LTEQOp => {
            contents += " <= ";
        }
        _ => {}
    }

    return contents;
}


pub fn handle_operand<'a> (operand :  &Operand<'a>) -> String {
    let mut contents = String::new();
    match operand {
        Operand::LValue(ref lval) => {
            match lval {
                LValue::Scalar(ref id) => {
                    contents += &format!("{:?}", id.id_name);
                }
                LValue::Array(id1,  id2) => {
                    contents += &format!("{:?}[{:?}]", id1.id_name, handle_array_op(id2));
                }
                LValue::Field(ref p, ref f) => {
                    contents += &format!("{:?}.{:?}", p.id_name, f.id_name);
                }
            }
        }
        Operand::Value(ref val) => {
            contents += &format!("{:?}", val.value);
        }
    }

    return contents;
}


pub fn handle_array_op<'a> (operand :  &Operand<'a>) -> String {
    let mut contents = String::new();
    match operand {
        Operand::LValue(ref lval) => {
            match lval {
                LValue::Scalar(ref my_id) => {
                    contents += &format!("{:?}", my_id.id_name);
                }
                LValue::Field(ref p, ref f) => {
                    contents += &format!("{:?}.{:?}", p.id_name, f.id_name);
                }
                _ => {}
            }
        }
        Operand::Value(ref rval_val) => {
            contents += &format!("{:?}", rval_val.value);
        }
    }
    
    return contents;
}



pub fn insert_packet_decls<'a> (my_dag : &mut Dag<'a>, my_packets : &Packets<'a>, pkt_tree : &Packets<'a>) {

    let mut insert_ind : usize = 0;
    for dagnode in my_dag.dag_vector.clone() {
        match &dagnode.node_type {
            // All vardecls will always be parsed first, before any other lines of code. If/else blocks follow
            DagNodeType::Decl(var_decl) => {
                insert_ind += 1;
            }
            _ => {}
        }
    }

    for my_packet in &my_packets.packet_vector {

        for field in &my_packet.packet_fields.field_vector {
            let field_name  = format!("{}.{}", my_packet.packet_id.id_name.clone(), field.identifier.id_name.clone());
            
            let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
            let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
            let header_decl = VariableDecl {identifier : Identifier{id_name : Box::leak(field_name.into_boxed_str()) },
                initial_values : Vec::<Value>::new(), var_type : field.var_type.clone()};
            let packet_decl_node = DagNode {node_type : DagNodeType::Decl(header_decl.clone()),
                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
            my_dag.dag_vector.insert(insert_ind, packet_decl_node);
            insert_ind += 1
        }

        for my_pkt in &pkt_tree.packet_vector {
            for my_pkt_field in &my_pkt.packet_fields.field_vector {

                let my_id = my_pkt_field.identifier.id_name.clone();
                let field_name  = format!("{}.{}{}", my_packet.packet_id.id_name.clone(), my_pkt.packet_id.id_name.clone(), my_id);
                
                let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
                let header_decl = VariableDecl {identifier : Identifier{id_name : Box::leak(field_name.into_boxed_str()) },
                    initial_values : Vec::<Value>::new(), var_type : my_pkt_field.var_type.clone()};
                let packet_decl_node = DagNode {node_type : DagNodeType::Decl(header_decl.clone()),
                    p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
                my_dag.dag_vector.insert(insert_ind, packet_decl_node);
                insert_ind += 1
            }
        }
    }
}



pub fn branch_removal<'a> (my_dag : &mut Dag<'a>, packet_map : &HashMap<String, String>, my_snippet : &Snippet<'a>, field_decls : &HashMap<String, VarType>) {

    let mut symbol_table : HashMap<&'a str, VarType<'a>> = HashMap::new();

    for my_variable_decl in &my_snippet.variable_decls.decl_vector {

        let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
        let dummpyp4 = P4Code{p4_header: dummyheader, p4_control:String::new(), p4_actions:String::new(), p4_commons:String::new()};
        let my_dag_start_node = DagNode {node_type : DagNodeType::Decl(my_variable_decl.clone()),
            p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new(), pre_condition : None};
        my_dag.dag_vector.push(my_dag_start_node);
        // populate symbol table here
        symbol_table.insert(my_variable_decl.identifier.id_name, my_variable_decl.var_type.clone());
    }

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

                // adds node for if_bit declaration
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

                        // get varinfo's 1st index in bitarray from varinfo field of my_statement.expr.lvalue.scalar.id_name variable(match on scalar/
                        // array/packet_field then extract id_name) in symbol table
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
                            // _ => {}
                            LValue::Field(ref p, ref f) => {
                                let field = format!("{}.{}", p.id_name, f.id_name);
                                let field_name_map = packet_map.get(&field).unwrap();
                                let vtype = field_decls.get(field_name_map).unwrap();
                                let width =  match vtype.var_info {
                                              VarInfo::BitArray(bit_width, _var_size) => bit_width,
                                              _ => {0}
                                            };
                                vinfo = VarInfo::BitArray(width, 1);
                            }
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
                            println!("Assigning a cond expr\n");
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
                        println!("Assigning a cond expr\n");

                    } else if my_if_block.condtype == 2 {
                        let if_var =  format!("if_block_tmp_{}", my_if_block.id - 1);
                        tmp_expr = Expr { op1: Operand::LValue(LValue::Scalar(Identifier{id_name: Box::leak(if_var.into_boxed_str()),})),
                                        expr_right: ExprRight::Cond(Operand::LValue(my_statement.lvalue.clone()),
                                         my_statement.expr.op1.clone()) };
                    }
                    println!("tmp_expr : {:?}\n", tmp_expr);

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
}



// This func creates the snippet dag. It performs branch removal (to convert if/else
// statements to single line ternary conditionals) and single-static assignment for each snippet 
// TODO need to handle packet field nodes
pub fn create_dag_nodes<'a> (my_snippets : &'a Snippets, packet_map : &HashMap<String, String>,
    my_packets : &Packets<'a>, pkt_tree : &Packets<'a>,) -> HashMap<&'a str, Dag<'a>>  {

    let mut dag_map : HashMap<&str, Dag>= HashMap::new();
    let mut field_decls : HashMap<String, VarType> = HashMap::new();

    // put all packet fields in hashmap
    for my_pkt in  &pkt_tree.packet_vector {
        for my_pkt_field in &my_pkt.packet_fields.field_vector {
            let f = my_pkt_field.identifier.id_name.clone();
            let field_name = format!("{}.{}", my_pkt.packet_id.id_name.clone(),f);
            field_decls.insert(field_name, my_pkt_field.var_type.clone());
        }
    }

    for my_packet in &my_packets.packet_vector {
        for field in &my_packet.packet_fields.field_vector {
            let field_name  = format!("{}.{}", my_packet.packet_id.id_name.clone(), field.identifier.id_name.clone());
            field_decls.insert(field_name, field.var_type.clone());
        }
    }

    for my_snippet in &my_snippets.snippet_vector {

        let mut my_dag : Dag = Dag { snippet_id : my_snippet.snippet_id.id_name,
            device_type : my_snippet.device_annotation.device_type.id_name, 
            device_vector : my_snippet.device_annotation.device_vector.clone(), dag_vector : Vec::new()};

        insert_packet_decls(&mut my_dag, my_packets, pkt_tree);
        branch_removal(&mut my_dag, &packet_map, my_snippet, &field_decls);
        dag_map.insert(&my_snippet.snippet_id.id_name, my_dag);
    }

    return dag_map;
}



pub fn gen_code<'a> (my_packets : &Packets<'a>, dag_map : HashMap<&'a str, Dag<'a>>) {

    for (snippet_name, snippet_dag) in dag_map {
        if snippet_dag.device_type.contains("bmv2") {
            bmv2_gen::gen_p4_code(&snippet_name, my_packets, &snippet_dag);
            bmv2_gen::gen_control_plane_commands(&snippet_name, my_packets, &snippet_dag);
        } else if snippet_dag.device_type.contains("tofino"){
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
        let tokens = &mut lexer::get_tokens(&contents);
        let token_iter = &mut tokens.iter().peekable();
        let dev_tree = parser::parse_device(token_iter);
        for my_dev_field in dev_tree.device_fields.field_vector {
            let field_name = format!("{}.{}", dev_tree.device_id.id_name.clone(), my_dev_field.identifier.id_name.clone());
            let identifier = format!("{}", my_dev_field.identifier.id_name.clone());
            import_map.insert(field_name, identifier);
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
        println!("Importing Packet{}\n", packet_file);
        let mut f = File::open(packet_file).expect("File not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("Something went wrong reading the file");
        let tokens = & mut lexer::get_tokens(&contents);
        let token_iter = & mut tokens.iter().peekable();
        let pkt_tree = parser::parse_import_packets(token_iter);
        for my_pkt in  pkt_tree.packet_vector {
            for my_pkt_field in &my_pkt.packet_fields.field_vector {
                let my_id = my_pkt_field.identifier.id_name.clone();
                let field_name  = format!("{}.{}{}", my_packet.packet_id.id_name.clone(), my_pkt.packet_id.id_name.clone(), my_id);
                let identifier = format!("{}.{}", my_pkt.packet_id.id_name.clone(),my_id);
                packet_map.insert(field_name, identifier);
            }
            println!("Packet : {:?}\n", my_pkt);
        }

        for field in &my_packet.packet_fields.field_vector {
            let field_name  = format!("{}.{}", my_packet.packet_id.id_name.clone(), field.identifier.id_name.clone());
            let identifier = field_name.clone();
            packet_map.insert(field_name, identifier);
        }
    }
    println!("Packet Map:{:?}\n", packet_map);
    return packet_map;
}


// Currently, if there is only a single snippet, then the program
// installs that single snippet on all devices, regardless of any annotations
pub fn gen_topology_json<'a> (dag_map : &HashMap<&'a str, Dag<'a>>) {

    let topo : String = format!("bmv2_sim/topology.json");
    let path = Path::new(topo.as_str());
    let display  = path.display();
    let mut topo_file = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(topo_file) => topo_file,
    };

    let mut contents = String::new();

    if dag_map.len() == 1 {
        for key in dag_map.keys() {
            contents += &format!("{{\"snippet_loc\" : \"{}\"}}", key);
        }
    } else {

        contents += "{\"snippet_loc\" : {";
        for (snippet_name, snippet_dag) in dag_map {
            for d in &snippet_dag.device_vector {
                contents += &format!("\"{}\" : \"{}\",", d.id_name, snippet_name);
            }
        }

        assert_eq!(contents.pop(), Some(','));
        contents += "}}";
    }

    topo_file.write(contents.as_bytes());
    let mut root = Path::new("bmv2_sim");
    assert!(env::set_current_dir(&root).is_ok());
    let _output = Command::new("python")
            .arg("gen_topo.py")
            .output()
            .expect("failed to execute process");

    root = Path::new("..");
    assert!(env::set_current_dir(&root).is_ok());
}



// need to use either 'bmv2' or 'tofino' for device annotation
pub fn trans_snippets<'a> (my_imports : &Imports<'a>, my_globals : &Globals<'a>, my_packets : &Packets<'a>, my_snippets : &Snippets<'a>, pkt_tree : &Packets<'a>) {
    // TODO : Deal with mutability of my_dag
    let import_map = create_import_map(my_imports);
    let packet_map = create_packet_map(my_packets);
    let mut dag_map = create_dag_nodes(&my_snippets, &packet_map, my_packets, pkt_tree);
    println!("\n\n\n Empty Dag Map: {:?}\n\n\n\n", dag_map);


    for my_snippet in &my_snippets.snippet_vector {

        let mut my_option = dag_map.get_mut(&my_snippet.snippet_id.id_name);
        let device_type : String = String::from(my_snippet.device_annotation.device_type.id_name);
        match my_option {
           Some(mut snippet_dag) => {
                // create_connections(&my_snippet, &my_packets, &pkt_tree, &my_imports, &mut snippet_dag.clone());
                create_RAW_connections(&my_snippet, &my_packets, &pkt_tree, &my_imports, &mut snippet_dag.clone());
                // println!("Snippet DAG with connections: {:?}\n", snippet_dag);
                if device_type.contains("bmv2") {
                    bmv2_gen::fill_p4code(&import_map, &my_globals, &packet_map, &mut snippet_dag, &pkt_tree,  &my_packets);
                } else if device_type.contains("tofino") {
                    tofino_gen::fill_p4code(&import_map, &my_globals, &packet_map, &mut snippet_dag, &pkt_tree, &my_packets);
                }
                // println!("Snippet DAG: {:?}\n", snippet_dag);
           }
           None => {}
        }
    }

    gen_topology_json(&dag_map);    
    // dag_map now contains p4 code and connection information (next/prev node)
    println!("\n\n\n Filled Dag Map: {:?}\n\n\n\n", dag_map);
    // process::exit(1);
    gen_code(&my_packets, dag_map);
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
        let pkt_tree = parser::parse_import_packets(token_iter);
        // TODO : need to replace &parse_tree.packets (the 4th func input) with the actual pkt_tree
        $trans_snippet_routine(&parse_tree.imports, &parse_tree.globals, &parse_tree.packets, &parse_tree.snippets, &pkt_tree);
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
  test_trans_success!(r"  @ bmv2
                          snippet fun(){
                              transient z : bit<1>;
                              transient r : bit<32>;
                              transient q : bit<32>;
                              transient m : bit<32>;
                              transient l : bit<32>;
                              transient i : bit<32>;
                              persistent reg1 : bit<32> = 0;
                              persistent reg2 : bit<32> = 0;
                              persistent reg3 : bit<32> = 0;
                              q = 10;
                              r = 5;
                              if(q > r) {
                                l = reg3;
                                i = q + l;
                              } else {
                                l = reg1;
                                i = q - l;
                              }
                              reg1 = 11;
                              z = q >= 10;
                              m = z ? q : r;
                              reg2 = i + 5;
                            }
                        ", trans_snippets, test_trans_snippets_if_else);
    test_trans_success!(r"  @ bmv2
                            snippet first1() {
                              transient z : bit<1>;
                              transient r : bit<32>;
                              transient q : bit<32>;
                              transient m : bit<32>;
                              transient l : bit<32>;
                              transient i : bit<32>;
                              persistent reg1 : bit<32> = 0;
                              persistent reg2 : bit<32> = 0;
                              persistent reg3 : bit<32> = 0;
                              transient if_block_tmp_2 : bit<1>;
                              transient tmp_0_if_2 : bit<32>;
                              transient tmp_1_if_3 : bit<32>;
                              q = 10;
                              r = 5;
                              if_block_tmp_2 = q > r;
                              l = if_block_tmp_2 ? reg3 : l;
                              tmp_0_if_2 = q + l;
                              i = if_block_tmp_2 ? tmp_0_if_2 : i;
                              l = if_block_tmp_2 ? l : reg1;
                              tmp_1_if_3 = q - l;
                              i = if_block_tmp_2 ? i : tmp_1_if_3;
                              reg1 = 11;
                              z = q >= 10;
                              m = z ? q : r;
                              reg2 = i + 5;
                            }
                        ", trans_snippets, test_trans_snippets_ternary_cond);

 }
