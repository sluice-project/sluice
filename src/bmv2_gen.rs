
use grammar::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::path::Path;
use trans_snippet::*;

const META_HEADER : &str = "mdata";
const TAB : &str = "    ";

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
pub fn get_p4_header_trans<'a> (node_type : &'a DagNodeType<'a>) -> P4Header {
    let mut my_p4_header : P4Header = P4Header {meta:String::new(), meta_init:String::new(), register:String::new(), define:String::new()};
    let mut my_vardecl : VarDecl;
    match &node_type {
        DagNodeType::Decl(my_decl) => {
            // Based on the type, the variable decl should be either a register/meta.
            // Push it to HashMap
            //let my_decl_g : VariableDecl<'a> = **my_decl;
            //decl_map.insert(my_decl.identifier.id_name, my_decl);
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
pub fn handle_statement<'a> (my_statement :  &Statement<'a>, node_type : &DagNodeType<'a>,
    decl_map : &'a HashMap<String, VarDecl> ) -> Vec<String> {
        let mut my_p4_body = Vec::new();

        match my_statement.lvalue {
            LValue::Scalar(ref my_id) => {
                let my_lval : String = String::from(my_id.id_name);
                let my_decl = decl_map.get(&my_lval).unwrap();

            }
            _ => {
                //TODO. Do this for Array
            }
        }
        return my_p4_body;
    }


// Ideally to get both ingress and egress parts of conversion [0] for ingress and [1] for egress and [2] for actions
pub fn get_p4_body_trans<'a> (node_type : &DagNodeType<'a>, decl_map : &'a HashMap<String, VarDecl>) -> Vec<String> {
    let my_p4_ingress : String = String::new();
    let my_p4_egress : String = String::new();
    let my_p4_commons : String = String::new();
    let mut my_p4_body = Vec::new();

    match &node_type {
        DagNodeType::Cond(my_cond) => {
            // TODO : If Statements
            return my_p4_body;
        }
        DagNodeType::Stmt(my_statement) => {
            return handle_statement(&my_statement, node_type, decl_map);
        }
        _ => {
            return my_p4_body;
        }
    }
}

pub fn fill_p4code<'a> (my_dag :  &mut Dag<'a>) {

    for mut my_dag_node in &mut my_dag.dag_vector {
        let mut decl_map : HashMap<String, VarDecl>= HashMap::new();
        my_dag_node.p4_code.p4_header = get_p4_header_trans(&my_dag_node.node_type);
        // Insert nodes to decl_map
        match my_dag_node.node_type {
            DagNodeType::Decl(my_decl) => {
                let mut my_vardecl : VarDecl;
                let my_id : String = String::from(my_decl.identifier.id_name);
                let mut my_varinfo : VarInfo<'a>;
                match my_decl.var_type.var_info {
                    VarInfo::BitArray(bit_width, var_size) => {
                        my_varinfo = VarInfo::BitArray(bit_width, var_size);
                    }
                    _ => {
                        my_varinfo = VarInfo::BitArray(0, 0);
                    }
                }
                let my_typequalifier : TypeQualifier = my_decl.var_type.type_qualifier;
                my_vardecl = VarDecl{id : my_id, var_info : my_varinfo, type_qualifier : my_typequalifier};
                decl_map.insert(String::from(my_decl.identifier.id_name), my_vardecl);
            }
            _ => {}

        }

        let my_code : Vec<String> = get_p4_body_trans(&my_dag_node.node_type, &decl_map);

    }
}

fn gen_p4_includes<'a> ( p4_file : &mut File) {
    p4_file.write(b"#include <core.p4>\n#include <v1model.p4>\n");
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

fn gen_p4_parser<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    // TODO
}

fn gen_p4_body<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();

    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_commons.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_commons;
        }
    }

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

pub fn gen_p4_code<'a> (snippet_name : &str, snippet_dag : &Dag<'a>){
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
    gen_p4_parser(&snippet_dag, &mut p4_file);
    //gen_p4_actions(&snippet_dag, &mut p4_file);
    gen_p4_body(&snippet_dag, &mut p4_file);
}
