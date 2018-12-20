// Translation from Sluice to p4 for each snippet.
// This works by first constructing a DAG.
use grammar::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::path::Path;

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
    pub prev_nodes : Vec<usize>
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct P4Code {
    pub p4_header : P4Header,
    pub p4_ingress : String,
    pub p4_egress : String,
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
            // Populate prev_nodes
            for (_my_id_1,p_index_1) in my_indices_1 {
                let my_dag_option = my_dag.dag_vector.get_mut(i);
                match my_dag_option {
                    Some(mut my_dag_node) => {
                        if !&my_dag_node.prev_nodes.contains(&p_index_1) {
                            my_dag_node.prev_nodes.push(p_index_1);
                            //println!("My_dag_node: {:?}", my_dag_node);
                        }
                    }
                    None => {}
                }
            }
            for (_my_id_2,p_index_2) in my_indices_2 {
                let my_dag_option = my_dag.dag_vector.get_mut(i);
                match my_dag_option {
                    Some(mut my_dag_node) => {
                        if !&my_dag_node.prev_nodes.contains(&p_index_2) {
                            my_dag_node.prev_nodes.push(p_index_2);
                            //println!("My_dag_node: {:?}", my_dag_node);
                        }
                    }
                    None => {}
                }
            }
            for (_my_id_3,p_index_3) in my_indices_3 {
                let my_dag_option = my_dag.dag_vector.get_mut(i);
                match my_dag_option {
                    Some(mut my_dag_node) => {
                        if !&my_dag_node.prev_nodes.contains(&p_index_3) {
                            my_dag_node.prev_nodes.push(p_index_3);
                            //println!("My_dag_node: {:?}", my_dag_node);
                        }
                    }
                    None => {}
                }
            }
            for (_my_id_4,p_index_4) in my_indices_4 {
                let my_dag_option = my_dag.dag_vector.get_mut(i);
                match my_dag_option {
                    Some(mut my_dag_node) => {
                        if !&my_dag_node.prev_nodes.contains(&p_index_4) {
                            my_dag_node.prev_nodes.push(p_index_4);
                            //println!("My_dag_node: {:?}", my_dag_node);
                        }
                    }
                    None => {}
                }
            }
            for (_my_id_5,p_index_5) in my_indices_5 {
                let my_dag_option = my_dag.dag_vector.get_mut(i);
                match my_dag_option {
                    Some(mut my_dag_node) => {
                        if !&my_dag_node.prev_nodes.contains(&p_index_5) {
                            my_dag_node.prev_nodes.push(p_index_5);
                            //println!("My_dag_node: {:?}", my_dag_node);
                        }
                    }
                    None => {}
                }
            }
            //println!("decl map: {:?}\n ", decl_map);
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
            let dummpyp4 = P4Code{p4_header: dummyheader, p4_ingress:String::new(), p4_egress:String::new()};
            let my_dag_start_node = DagNode {node_type : DagNodeType::Decl(my_variable_decl),
                p4_code : dummpyp4, next_nodes : Vec::new(), prev_nodes : Vec::new()};
            //println!("{:?}\n", my_dag_start_node);
            my_dag.dag_vector.push(my_dag_start_node);
        }
        for my_if_block in &my_snippet.ifblocks.ifblock_vector {
            for my_statement in &my_if_block.statements.stmt_vector {
                let dummyheader = P4Header{meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
                let dummpyp4 = P4Code{p4_header:dummyheader, p4_ingress:String::new(), p4_egress:String::new()};
                let mut my_dag_node = DagNode {node_type: DagNodeType::Stmt(&my_statement),
                    p4_code : dummpyp4, next_nodes: Vec::new(), prev_nodes: Vec::new()};
                my_dag.dag_vector.push(my_dag_node);
            }
        }
        dag_map.insert(&my_snippet.snippet_id.id_name, my_dag);
    }
    dag_map
}

pub fn handle_transient_decl<'a> (my_decl :  &VariableDecl<'a>) -> P4Header {
    let mut my_p4_header : P4Header = P4Header {meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
    match my_decl.var_type.var_info {
        VarInfo::BitArray(bit_width, var_size) => {
            if var_size == 1 {
                let initial_val_index : usize = 0;
                my_p4_header.meta = format!("{} : {};\n",my_decl.identifier.id_name, bit_width);
                let my_option = my_decl.initial_values.get(initial_val_index);
                match my_option {
                    Some (initial_value) => {
                        my_p4_header.meta_init = format!("set_metadata({}.{},{});\n",
                            META_HEADER, my_decl.identifier.id_name, initial_value.value);
                    }
                    _ => {}
                }

            } else {
                panic!("[Error]: Array Unsupported on transient type!\n");
            }
        }
        _ => { }
    }
    return my_p4_header;
}

pub fn handle_persistent_decl<'a> (my_decl :  &VariableDecl<'a>) -> P4Header {
    let mut my_p4_header : P4Header = P4Header {meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
    match my_decl.var_type.var_info {
        VarInfo::BitArray(bit_width, var_size) => {
            let initial_val_index : usize = 0;
            my_p4_header.register = format!("register {} {{\n{} width : {}; \n{} instance_count : {};\n}}\n",
            my_decl.identifier.id_name, TAB, bit_width, TAB, var_size);
            let my_option = my_decl.initial_values.get(initial_val_index);
            match my_option {
                Some (initial_value) => {
                    my_p4_header.meta_init = format!("set_metadata({}.{},{});\n",
                        META_HEADER, my_decl.identifier.id_name, initial_value.value);
                }
                _ => {}
            }
        }
        _ => { }
    }
    return my_p4_header;
}
pub fn get_p4_header_trans<'a> (node_type : &DagNodeType<'a>) -> P4Header {
    let mut my_p4_header : P4Header = P4Header {meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
    match &node_type {
        DagNodeType::Decl(my_decl) => {
            // Based on the type, the variable decl should be either a register/meta.
            match my_decl.var_type.type_qualifier {
                TypeQualifier::Transient => {
                    my_p4_header = handle_transient_decl(my_decl);
                }
                TypeQualifier::Persistent => {
                    my_p4_header = handle_persistent_decl(my_decl);
                }
                _ => {}
            }
            return my_p4_header;
        }
        _ => {
            my_p4_header
        }
    }
}


pub fn get_p4_body_trans<'a> (node_type : &DagNodeType<'a>) -> Vec<&'a str> {
    let my_p4_ingress : String = String::new();
    let my_p4_egress : String = String::new();
    let mut my_p4_body = Vec::new();

    match &node_type {
        DagNodeType::Cond(my_decl) => {
            return my_p4_body;
        }
        DagNodeType::Stmt(my_decl) => {
            return my_p4_body;
        }
        _ => {
            return my_p4_body;
        }
    }
}

fn fill_p4code<'a> (my_dag : &mut Dag<'a>) {
    for mut my_dag_node in &mut my_dag.dag_vector {
        my_dag_node.p4_code.p4_header = get_p4_header_trans(&my_dag_node.node_type);
        //my_dag_node.p4_code.p4_ingress, my_dag_node.p4_code.p4_egress
        let pair = get_p4_body_trans(&my_dag_node.node_type);

    }
}

fn gen_p4_includes<'a> ( p4_file : &mut File) {
    p4_file.write(b"#include <tofino/intrinsic_metadata.p4>\n#include <tofino/constants.p4>\n");
    p4_file.write(b"#include <tofino/primitives.p4>\n#include \"tofino/stateful_alu_blackbox.p4\"\n");
    p4_file.write(b"#include \"tofino/lpf_blackbox.p4\"\n#include \"tofino/wred_blackbox.p4\"\n");
}

fn gen_p4_globals<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();
    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_header.define.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_header.define
        }
    }
    p4_file.write(contents.as_bytes());
}
fn gen_p4_headers<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    // TODO
}

fn gen_p4_metadata<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();
    let mut meta_found = 0;
    contents = contents + &format!("header_type metadata_t {{ \n");
    contents = contents + &format!("{}fields {{\n", TAB);
    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_header.meta.len() != 0) {
            contents = contents + &format!("{}{}{}",TAB, TAB,my_dag_node.p4_code.p4_header.meta);
            meta_found = 1;
        }
    }
    contents = contents + &format!("{}}}\n}}\n", TAB);
    if meta_found == 1 {
        p4_file.write(contents.as_bytes());
    }

}

fn gen_p4_registers<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();
    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_header.register.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_header.register;
        }
    }
    p4_file.write(contents.as_bytes());
}

fn gen_p4_body<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();
    contents = contents + &format!("control ingress {{\n");
    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_ingress.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_ingress;
        }
    }
    contents = contents + &format!("}}\n");

    contents = contents + &format!("control egress {{\n");
    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_ingress.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_egress;
        }
    }
    contents = contents + &format!("}}\n");
    p4_file.write(contents.as_bytes());
}

fn gen_p4_code<'a> (dag_map : HashMap<&'a str, Dag<'a>>){
    for (snippet_name, snippet_dag) in dag_map {
        let p4_filename : String = format!("out/{}.p4", snippet_name);
        let path = Path::new(p4_filename.as_str());
        let display  = path.display();
        let mut p4_file = match File::create(path) {
            Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
            Ok(p4_file) => p4_file,
        };
        gen_p4_includes(&mut p4_file);
        gen_p4_globals(&snippet_dag, &mut p4_file);
        gen_p4_headers(&snippet_dag, &mut p4_file);
        gen_p4_metadata(&snippet_dag, &mut p4_file);
        gen_p4_registers(&snippet_dag, &mut p4_file);
        // gen_p4_parser(&mut p4_file);
        gen_p4_body(&snippet_dag, &mut p4_file);
    }
}

pub fn trans_snippets<'a> (my_snippets : &Snippets<'a>) {
    // TODO : Deal with mutability of my_dag
    let mut dag_map = create_dag_nodes(&my_snippets);
    //println!("Dag Map: {:?}\n", dag_map);
    for my_snippet in &my_snippets.snippet_vector {
    //for (snippet_name, mut  snippet_dag) in dag_map {
        let mut my_option = dag_map.get_mut(&my_snippet.snippet_id.id_name);
        match my_option {
           Some(mut snippet_dag) => {
                create_connections(&my_snippet, &mut snippet_dag);
                fill_p4code(&mut snippet_dag);
                println!("Snippet DAG: {:?}\n", snippet_dag);
           }
           None => {}
        }
    }
    gen_p4_code(dag_map);
}
