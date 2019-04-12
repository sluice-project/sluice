extern crate regex;
use self::regex::Regex;
use grammar::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::path::Path;
use trans_snippet::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, Ordering};

const META_HEADER : &str = "mdata";
const TAB : &str = "    ";

//TODO : Deal with warnings
#[allow(unused_must_use)]
#[allow(dead_code)]
#[allow(unused_imports)]
static ACTION_COUNT : AtomicUsize = AtomicUsize::new(1);
static TABLE_COUNT : AtomicUsize = AtomicUsize::new(1);
static TEMP_COUNT : AtomicUsize = AtomicUsize::new(1);
static EQ_TABLE_COUNT : AtomicUsize = AtomicUsize::new(1);
static NEQ_TABLE_COUNT : AtomicUsize = AtomicUsize::new(1);

static NEW_ACTION : AtomicBool = AtomicBool::new(true);

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
            my_p4_header.meta = format!("{} : {};\n",my_decl.identifier.id_name, bit_width);
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

// how to handle input/output packet decls?? (see ecn.np)
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

pub fn get_NEW_ACTION () -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    my_p4_control = my_p4_control + &format!("{}apply(table{:?});\n", TAB, TABLE_COUNT);
    my_p4_actions = my_p4_actions + &format!("action action{:?} () {{\n", ACTION_COUNT);
    my_p4_commons = my_p4_commons + &format!("table table{:?} {{\n", TABLE_COUNT);
    my_p4_commons = my_p4_commons + &format!("{}actions {{\n", TAB);
    my_p4_commons = my_p4_commons + &format!("{}{}action{:?};\n", TAB, TAB, ACTION_COUNT);
    my_p4_commons = my_p4_commons + &format!("{}}}\n", TAB);
    my_p4_commons = my_p4_commons + &format!("}}\n");
    ACTION_COUNT.fetch_add(1, Ordering::SeqCst);
    TABLE_COUNT.fetch_add(1, Ordering::SeqCst);
    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_read_register (my_decl : &VarDecl, my_index : u64) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let (a,b,c) = get_NEW_ACTION();
    my_p4_control = a; my_p4_actions = b; my_p4_commons = c;
    my_p4_actions = my_p4_actions + &format!("{}register_read({}.{}, {}, {});\n", TAB,
        META_HEADER, my_decl.id, my_decl.id, my_index);
    my_p4_actions = my_p4_actions + &format!("}}\n");

    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_value_assignment<'a> ( my_lval_decl : &VarDecl, my_lval_index : u64,  val : u64) -> (String, String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut my_p4_metadecl : String = String::new();

    println!("handling value assignment for  :{:?}\n", my_lval_decl);
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if NEW_ACTION.load(Ordering::SeqCst) {
                let (a, b, c) = get_NEW_ACTION();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, {});\n", TAB, META_HEADER, my_lval_decl.id, val);
            if NEW_ACTION.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }

        TypeQualifier::Persistent => {
            // Register
            if NEW_ACTION.load(Ordering::SeqCst) {
                let (a, b, c) = get_NEW_ACTION();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {});\n", TAB, my_lval_decl.id, my_lval_index, val);
            if NEW_ACTION.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }

        TypeQualifier::Field => {
            // Metadata
            if NEW_ACTION.load(Ordering::SeqCst) {
                let (a, b, c) = get_NEW_ACTION();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}, {});\n", TAB, my_lval_decl.id, val);
            if NEW_ACTION.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }

        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }
    }
}




pub fn handle_read_register_v2 (my_decl : &VarDecl, my_index : u64) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    my_p4_actions = my_p4_actions + &format!("{}register_read({}.{}, {}, {});\n", TAB,
        META_HEADER, my_decl.id, my_decl.id, my_index);
    return (my_p4_control, my_p4_actions, my_p4_commons);
}

// reg1 = if_block_tmp_2 ? tmp_0_if_2 : reg1; (see test1.np)
// handle_ref_assignment(reg1, index, tmp_0_if_2, index, v2)
pub fn handle_ref_assignment<'a> (my_lval_decl : &VarDecl, my_lval_index : u64, my_rval_decl : &VarDecl, my_rval_index : u64,
                            read_reg_func : &Fn(&VarDecl, u64) -> (String, String, String) ) -> (String, String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut my_p4_metadecl : String = String::new();

    let mut prefix = "";
    match my_rval_decl.type_qualifier {
        TypeQualifier::Persistent => {
            // If register, then first need to read the register val to meta.
            let (a,b,c) = read_reg_func(my_rval_decl, my_rval_index);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            prefix = META_HEADER;
        }
        TypeQualifier::Transient => {
            // If register, then first need to read the register val to meta.
            prefix = META_HEADER;
        }
        _ => {
            // For others, nothing to be done.
        }
    }
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if NEW_ACTION.load(Ordering::SeqCst) {
                let (a, b, c) = get_NEW_ACTION();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            if prefix.len()!= 0 {
                my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, {}.{});\n", TAB,
                    META_HEADER, my_lval_decl.id, prefix, my_rval_decl.id);
            } else {
                my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, {});\n", TAB,
                    META_HEADER, my_lval_decl.id, my_rval_decl.id);
            }
            if NEW_ACTION.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }

            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }
        TypeQualifier::Persistent => {
            // Register
            if NEW_ACTION.load(Ordering::SeqCst) {
                let (a, b, c) = get_NEW_ACTION();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            if prefix.len()!= 0 {
                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {}.{});\n", TAB,
                        my_lval_decl.id, my_lval_index, prefix, my_rval_decl.id);
            } else {
                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {});\n", TAB,
                    my_lval_decl.id, my_rval_index, my_rval_decl.id);
            }
            if NEW_ACTION.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }

        TypeQualifier::Field => {
            // Metadata
            if NEW_ACTION.load(Ordering::SeqCst) {
                let (a, b, c) = get_NEW_ACTION();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }

            if prefix.len()!= 0 {
                my_p4_actions = my_p4_actions + &format!("{}modify_field({}, {}.{});\n", TAB,
                    my_lval_decl.id, prefix, my_rval_decl.id);
            } else {
                my_p4_actions = my_p4_actions + &format!("{}modify_field({}, {});\n", TAB,
                    my_lval_decl.id, my_rval_decl.id);
            }
            if NEW_ACTION.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }

            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }

        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }
    }
}

pub fn get_new_eq_table<'a> (my_temp_decl : &String, my_lval_decl : &VarDecl, eq : bool) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();

    let mut tablename : String;
    let mut actionname : String;
    match eq {
        true => {
            tablename = format!("eqtable{:?}", EQ_TABLE_COUNT).to_string();
            actionname = format!("eqaction{:?}_", EQ_TABLE_COUNT).to_string();
            EQ_TABLE_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        false => {
            tablename = format!("neqtable{:?}", NEQ_TABLE_COUNT).to_string();
            actionname = format!("neqaction{:?}_", EQ_TABLE_COUNT).to_string();
            NEQ_TABLE_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }
    my_p4_control = my_p4_control + &format!("{}apply({});\n", TAB, tablename);

    my_p4_actions = my_p4_actions + &format!("action {}0 () {{\n", actionname);
    my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, 0); \n}}\n", TAB, META_HEADER, my_lval_decl.id);
    my_p4_actions = my_p4_actions + &format!("action {}1 () {{\n", actionname);
    my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, 1); \n}}\n", TAB, META_HEADER, my_lval_decl.id);

    my_p4_commons = my_p4_commons + &format!("table {} {{\n", tablename);
    my_p4_commons = my_p4_commons + &format!("{}reads {{\n", TAB);
    my_p4_commons = my_p4_commons + &format!("{}{}{} : exact;\n{}}}\n", TAB, TAB, my_temp_decl, TAB);

    my_p4_commons = my_p4_commons + &format!("{}actions {{\n", TAB);
    my_p4_commons = my_p4_commons + &format!("{}{}{}0;\n", TAB, TAB, actionname);
    my_p4_commons = my_p4_commons + &format!("{}{}{}1;\n", TAB, TAB, actionname);
    my_p4_commons = my_p4_commons + &format!("{}}}\n", TAB);
    my_p4_commons = my_p4_commons + &format!("}}\n");
    EQ_TABLE_COUNT.fetch_add(1, Ordering::SeqCst);

    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_condition_refs<'a> (bin_op_type : BinOpType, my_lval_decl : &VarDecl, prefix1 : &str,
 my_rval1_decl : &VarDecl, prefix2 : &str, my_rval2_decl : &VarDecl) -> (String, String, String, String) {
     let mut my_p4_control : String = String::new();
     let mut my_sub_actio : String = String::new();
     let mut my_p4_actions : String = String::new();

     let mut my_p4_commons : String = String::new();
     let mut my_p4_metadecl : String = String::new();
     let mut my_p4func = "";
     let bit_width = 32;
     let my_temp_decl_id = &format!("temp{:?}", TEMP_COUNT);
     let my_temp_use_id = &format!("{}.{}", META_HEADER, my_temp_decl_id.to_string());
     my_p4_metadecl = my_p4_metadecl + &format!("{} : {};\n",my_temp_decl_id.to_string(), bit_width);
     //ACTION_COUNT.fetch_add(1, Ordering::SeqCst);

     let (a, b, c) = get_NEW_ACTION();
     my_p4_control = my_p4_control + &a;
     my_p4_actions = my_p4_actions + &b;
     my_p4_commons = my_p4_commons + &c;

     my_p4_actions = my_p4_actions + &format!("{}subtract({},", TAB, my_temp_use_id);
     match prefix1.len() {
        0 => {
            my_p4_actions = my_p4_actions + &format!("{},", my_rval1_decl.id);
        }
        _ => {
            my_p4_actions = my_p4_actions + &format!("{}.{},",prefix1, my_rval1_decl.id);
        }
    }
    match prefix2.len() {
        0 => {
            my_p4_actions = my_p4_actions + &format!("{});\n", my_rval2_decl.id);
        }
        _ => {
            my_p4_actions = my_p4_actions + &format!("{}.{});\n",prefix2, my_rval2_decl.id);
        }
    }
    my_p4_actions = my_p4_actions + &format!("}}\n");
     match bin_op_type {
         BinOpType::Equal => {
             let (a, b, c) = get_new_eq_table(my_temp_use_id, my_lval_decl, true);
             my_p4_control = my_p4_control + &a;
             my_p4_actions = my_p4_actions + &b;
             my_p4_commons = my_p4_commons + &c;
         }
         BinOpType::NotEqual => {
             let (a, b, c) = get_new_eq_table(my_temp_use_id, my_lval_decl, false);
             my_p4_control = my_p4_control + &a;
             my_p4_actions = my_p4_actions + &b;
             my_p4_commons = my_p4_commons + &c;
         }
         _ => {}
     }
     return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}


// This method is using p4 control blocks available in pipeline.
pub fn handle_condition_refs_v2<'a> (bin_op_type : &str, my_lval_decl : &VarDecl, prefix1 : &str,
 my_rval1_decl : &VarDecl, prefix2 : &str, my_rval2_decl : &VarDecl) -> (String, String, String, String) {
     let mut my_p4_control : String = String::new();
     let mut my_p4_actions : String = String::new();
     let mut my_p4_commons : String = String::new();
     let mut my_p4_metadecl : String = String::new();

     if prefix1.len() != 0 {
         my_p4_control = my_p4_control + &format!("{}if ({}.{} {} ", TAB, prefix1, my_rval1_decl.id, bin_op_type);
     } else {
         my_p4_control = my_p4_control + &format!("{}if ({} {} ", TAB, my_rval1_decl.id, bin_op_type);
     }
     if prefix2.len() != 0 {
         my_p4_control = my_p4_control + &format!("{}.{}) {{\n{}", prefix2, my_rval2_decl.id, TAB);
     } else {
         my_p4_control = my_p4_control + &format!("{}) {{\n{}", my_rval2_decl.id, TAB);
     }
     let (a, b, c) = get_NEW_ACTION();
     my_p4_control = my_p4_control + &a;
     my_p4_actions = my_p4_actions + &b;
     my_p4_commons = my_p4_commons + &c;

     match my_lval_decl.type_qualifier {
        TypeQualifier::Field => {
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}, 1); \n}}\n", TAB, my_lval_decl.id);
        }
        _ => { my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, 1); \n}}\n", TAB, META_HEADER, my_lval_decl.id); }
     }

     my_p4_control = my_p4_control + &format!("{}}} else {{\n{}", TAB, TAB);
     let (a, b, c) = get_NEW_ACTION();
     my_p4_control = my_p4_control + &a;
     my_p4_actions = my_p4_actions + &b;
     my_p4_commons = my_p4_commons + &c;

     match my_lval_decl.type_qualifier {
        TypeQualifier::Field => {
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}, 0); \n}}\n", TAB, my_lval_decl.id);
        }
        _ => { my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, 0); \n}}\n", TAB, META_HEADER, my_lval_decl.id); }
     }

     my_p4_control = my_p4_control + &format!("{}}}\n", TAB);
     return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}


pub fn handle_condition_refval<'a> (bin_op_type : BinOpType, my_lval_decl : &VarDecl, prefix1 : &str,
 my_rval1_decl : &VarDecl, val : u64) -> (String, String, String, String) {
     let mut my_p4_control : String = String::new();
     let mut my_p4_actions : String = String::new();
     let mut my_p4_commons : String = String::new();
     let mut my_p4_metadecl : String = String::new();
     let my_temp_decl_id = &format!("temp{:?}", TEMP_COUNT);
     let my_temp_use_id = &format!("{}.{}", META_HEADER, my_temp_decl_id.to_string());
     let bit_width = 32;
     my_p4_metadecl = my_p4_metadecl + &format!("{} : {};\n",my_temp_decl_id.to_string(), bit_width);
     //ACTION_COUNT.fetch_add(1, Ordering::SeqCst);

     let (a, b, c) = get_NEW_ACTION();
     my_p4_control = my_p4_control + &a;
     my_p4_actions = my_p4_actions + &b;
     my_p4_commons = my_p4_commons + &c;

     my_p4_actions = my_p4_actions + &format!("{}subtract({},", TAB, my_temp_use_id);
     match prefix1.len() {
         0 => {
             my_p4_actions = my_p4_actions + &format!("{},", my_rval1_decl.id);
         }
         _ => {
             my_p4_actions = my_p4_actions + &format!("{}.{},",prefix1, my_rval1_decl.id);
         }
     }
     my_p4_actions = my_p4_actions + &format!("{});\n", val);

     my_p4_actions = my_p4_actions + &format!("}}\n");
     match bin_op_type {
         BinOpType::Equal => {
             let (a, b, c) = get_new_eq_table(my_temp_use_id, my_lval_decl, true);
             my_p4_control = my_p4_control + &a;
             my_p4_actions = my_p4_actions + &b;
             my_p4_commons = my_p4_commons + &c;
         }
         BinOpType::NotEqual => {
             let (a, b, c) = get_new_eq_table(my_temp_use_id, my_lval_decl, true);
             my_p4_control = my_p4_control + &a;
             my_p4_actions = my_p4_actions + &b;
             my_p4_commons = my_p4_commons + &c;
         }
         _ => {}
     }
     return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}


// This method is using p4 control blocks available in pipeline.
pub fn handle_condition_refval_v2<'a> (bin_op_type : &str, my_lval_decl : &VarDecl, prefix1 : &str,
 my_rval1_decl : &VarDecl, val : u64) -> (String, String, String, String) {
     let mut my_p4_control : String = String::new();
     let mut my_p4_actions : String = String::new();
     let mut my_p4_commons : String = String::new();
     let mut my_p4_metadecl : String = String::new();
     if prefix1.len() != 0 {
         my_p4_control = my_p4_control + &format!("{}if ({}.{} {} {}) {{\n{}", TAB, prefix1, my_rval1_decl.id, bin_op_type, val, TAB);
     } else {
         my_p4_control = my_p4_control + &format!("{}if ({} {} {}) {{\n{}", TAB, my_rval1_decl.id, bin_op_type, val, TAB);
     }
     let (a, b, c) = get_NEW_ACTION();
     my_p4_control = my_p4_control + &a;
     my_p4_actions = my_p4_actions + &b;
     my_p4_commons = my_p4_commons + &c;

     match my_lval_decl.type_qualifier {
        TypeQualifier::Field => {
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}, 1); \n}}\n", TAB, my_lval_decl.id);
        }
        _ => { my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, 1); \n}}\n", TAB, META_HEADER, my_lval_decl.id); }
     }

     my_p4_control = my_p4_control + &format!("{}}} else {{\n{}", TAB, TAB);
     let (a, b, c) = get_NEW_ACTION();
     my_p4_control = my_p4_control + &a;
     my_p4_actions = my_p4_actions + &b;
     my_p4_commons = my_p4_commons + &c;

     match my_lval_decl.type_qualifier {
        TypeQualifier::Field => {
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}, 0); \n}}\n", TAB, my_lval_decl.id);
        }
        _ => { my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, 0); \n}}\n", TAB, META_HEADER, my_lval_decl.id); }
     }

     my_p4_control = my_p4_control + &format!("{}}}\n", TAB);
     return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}



//  Packet { packet_id: Identifier { id_name: "n" }, packet_base: Identifier { id_name: "udp" },

// packet_fields: PacketFields { field_vector: [
//         PacketField { identifier: Identifier { id_name: "new_one" },
//                     var_type: VarType { var_info: BitArray(32, 1), type_qualifier: Field } }]
// },

// packet_parser_condition: ParserCondition(Identifier { id_name: "srcPort" }, Value { value: 1234 }) }
//   // packet_map    : HashMap<String, (String, u64)>,



pub fn handle_binop_refs_assignment<'a> (my_lval_decl : &VarDecl,  my_lval_index : u64, my_rval1_decl : &VarDecl, my_rval1_index : u64,
    bin_op_type : BinOpType, my_rval2_decl : &VarDecl, my_rval2_index : u64, decl_map : &'a  HashMap<String, VarDecl> ) -> (String, String, String, String) {
        let mut my_p4_control : String = String::new();
        let mut my_p4_actions : String = String::new();
        let mut my_p4_commons : String = String::new();
        let mut my_p4_metadecl : String = String::new();
        let mut prefix1 = "";
        let mut prefix2 = "";

        match my_rval1_decl.type_qualifier {
            TypeQualifier::Persistent => {
                // If register, then first need to read the register val to meta.
                let (a,b,c) = handle_read_register(my_rval1_decl, my_rval1_index);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                prefix1 = META_HEADER;
            }
            TypeQualifier::Transient => {
                prefix1 = META_HEADER;
            }
            _ => {
                // For others, nothing to be done.
            }
        }

        match my_rval2_decl.type_qualifier {
            TypeQualifier::Persistent => {
                // If register, then first need to read the register val to meta.
                let (a,b,c) = handle_read_register(my_rval2_decl, my_rval2_index);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                prefix2 = META_HEADER;
            }
            TypeQualifier::Transient => {
                prefix2 = META_HEADER;
            }
            _ => {
                // For others, nothing to be done.
            }
        }
        let mut p4_func = "";
        match bin_op_type {
            BinOpType::BooleanAnd => {
                p4_func = "bit_and";
            }
            BinOpType::BooleanOr => {
                p4_func = "bit_or";
            }
            BinOpType::Plus => {
                p4_func = "add";
            }
            BinOpType::Minus => {
                p4_func = "subtract";
            }
            BinOpType::Mul => {
                p4_func = "";
            }
            BinOpType::Div => {
                p4_func = "";
            }
            BinOpType::Modulo => {
                p4_func = "";
            } // Conditions from here
            BinOpType::Equal => {
                p4_func = "";
                // Eg. a == b
                let (a,b,c,d) = handle_condition_refs_v2("==", my_lval_decl, prefix1, my_rval1_decl, prefix2, my_rval2_decl);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                my_p4_metadecl = my_p4_metadecl + &d;
            }
            BinOpType::NotEqual => {
                p4_func = "";
                // Eg. a != b
                let (a,b,c,d) = handle_condition_refs_v2("!=", my_lval_decl, prefix1, my_rval1_decl, prefix2, my_rval2_decl);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                my_p4_metadecl = my_p4_metadecl + &d;
            }
            BinOpType::GreaterThan => {
                p4_func = "";
                // Eg. a != b
                let (a,b,c,d) = handle_condition_refs_v2(">", my_lval_decl, prefix1, my_rval1_decl, prefix2, my_rval2_decl);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                my_p4_metadecl = my_p4_metadecl + &d;
            }
            BinOpType::LessThan => {
                p4_func = "";
                // Eg. a != b
                let (a,b,c,d) = handle_condition_refs_v2("<", my_lval_decl, prefix1, my_rval1_decl, prefix2, my_rval2_decl);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                my_p4_metadecl = my_p4_metadecl + &d;
            }
            BinOpType::GTEQOp => {
                p4_func = "";
                // Eg. a != b
                let (a,b,c,d) = handle_condition_refs_v2(">=", my_lval_decl, prefix1, my_rval1_decl, prefix2, my_rval2_decl);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                my_p4_metadecl = my_p4_metadecl + &d;
            }
            BinOpType::LTEQOp => {
                p4_func = "";
                // Eg. a != b
                let (a,b,c,d) = handle_condition_refs_v2("<=", my_lval_decl, prefix1, my_rval1_decl, prefix2, my_rval2_decl);
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
                my_p4_metadecl = my_p4_metadecl + &d;
            }
            _ => {
                p4_func = "";
                // Eg. a >= b
                panic!("Comparison on references not supported.\n");
            }
        }
        match my_lval_decl.type_qualifier {
            TypeQualifier::Transient => {
                // Metadata
                if p4_func.len() != 0 {
                    if NEW_ACTION.load(Ordering::SeqCst) {
                        let (a, b, c) = get_NEW_ACTION();
                        my_p4_control = my_p4_control + &a;
                        my_p4_actions = my_p4_actions + &b;
                        my_p4_commons = my_p4_commons + &c;
                    }
                    my_p4_actions = my_p4_actions + &format!("{}{}({}.{},", TAB, p4_func, META_HEADER, my_lval_decl.id);
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{},", my_rval1_decl.id);

                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{},",prefix1, my_rval1_decl.id);
                        }
                    }
                    match prefix2.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{});\n", my_rval2_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{});\n",prefix2, my_rval2_decl.id);
                        }
                    }
                    if NEW_ACTION.load(Ordering::SeqCst) {
                        my_p4_actions = my_p4_actions + &format!("}}\n");
                    }
                }

            }

            TypeQualifier::Persistent => {
                // Register
                if p4_func.len() != 0 {
                    if NEW_ACTION.load(Ordering::SeqCst) {
                        let (a, b, c) = get_NEW_ACTION();
                        my_p4_control = my_p4_control + &a;
                        my_p4_actions = my_p4_actions + &b;
                        my_p4_commons = my_p4_commons + &c;
                    }
                    my_p4_actions = my_p4_actions + &format!("{}{}({}.{}, ", TAB, p4_func, META_HEADER, my_lval_decl.id);
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{}, ", my_rval1_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{}, ",prefix1, my_rval1_decl.id);
                        }
                    }
                    match prefix2.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{});\n", my_rval2_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{});\n",prefix2, my_rval2_decl.id);
                        }
                    }
                    my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {}.{});\n", TAB,
                            my_lval_decl.id, my_lval_index, META_HEADER, my_lval_decl.id);
                    if NEW_ACTION.load(Ordering::SeqCst) {
                        my_p4_actions = my_p4_actions + &format!("}}\n");
                    }
                }
            }

            TypeQualifier::Field => {
                // packet fields
                if p4_func.len() != 0 {
                    if NEW_ACTION.load(Ordering::SeqCst) {
                        let (a, b, c) = get_NEW_ACTION();
                        my_p4_control = my_p4_control + &a;
                        my_p4_actions = my_p4_actions + &b;
                        my_p4_commons = my_p4_commons + &c;
                    }

                    my_p4_actions = my_p4_actions + &format!("{}{}({},", TAB, p4_func, my_lval_decl.id);
                    my_p4_actions = my_p4_actions + &format!("{},{});\n", my_rval1_decl.id, my_rval2_decl.id);
                    if NEW_ACTION.load(Ordering::SeqCst) {
                        my_p4_actions = my_p4_actions + &format!("}}\n");
                    }
                }

            }
            // not handling input, output, const...
            _ => {}
        }

        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}

//Direction : true  for ref <op> val, false for val <op> ref
pub fn handle_binop_refval_assignment<'a> (my_lval_decl : &VarDecl,  my_lval_index : u64,
    my_rval_decl : &VarDecl,  my_rval_index : u64,bin_op_type : BinOpType, val2 : u64,
     decl_map : &'a  HashMap<String, VarDecl>, ordering : bool) -> (String, String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut my_p4_metadecl : String = String::new();

    let mut prefix1 = "";

    match my_rval_decl.type_qualifier {
        TypeQualifier::Persistent => {
            // If register, then first need to read the register val to meta.
            let (a,b,c) = handle_read_register(my_rval_decl, my_rval_index);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            prefix1 = META_HEADER;
        }
        TypeQualifier::Transient => {
            prefix1 = META_HEADER;
        }
        _ => {
            // For others, nothing to be done.
        }
    }

    let mut p4_func = "";
    match bin_op_type {
        BinOpType::BooleanAnd => {
            p4_func = "bit_and";
        }
        BinOpType::BooleanOr => {
            p4_func = "bit_or";
        }
        BinOpType::Plus => {
            p4_func = "add";
        }
        BinOpType::Minus => {
            p4_func = "subtract";
        }
        BinOpType::Mul => {
            p4_func = "";
        }
        BinOpType::Div => {
            p4_func = "";
        }
        BinOpType::Modulo => {
            p4_func = "";
        }
        BinOpType::Equal => {
            p4_func = "";
            // Eg. a == 10
            let (a,b,c,d) = handle_condition_refval_v2("==", my_lval_decl, prefix1, my_rval_decl, val2);
            println!("Handing Condition.\n");
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            my_p4_metadecl = my_p4_metadecl + &d;
            println!("{:?} .. {:?} .. {:?}\n", my_p4_control, my_p4_actions, my_p4_commons);
        }
        BinOpType::NotEqual => {
            p4_func = "";
            // Eg. a != 10
            let (a,b,c,d) = handle_condition_refval_v2("!=", my_lval_decl, prefix1, my_rval_decl, val2);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            my_p4_metadecl = my_p4_metadecl + &d;
        }
        BinOpType::GreaterThan => {
            p4_func = "";
            // Eg. a != 10
            let (a,b,c,d) = handle_condition_refval_v2(">", my_lval_decl, prefix1, my_rval_decl, val2);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            my_p4_metadecl = my_p4_metadecl + &d;
        }
        BinOpType::LessThan => {
            p4_func = "";
            // Eg. a != 10
            let (a,b,c,d) = handle_condition_refval_v2("<", my_lval_decl, prefix1, my_rval_decl, val2);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            my_p4_metadecl = my_p4_metadecl + &d;
        }
        BinOpType::GTEQOp => {
            p4_func = "";
            // Eg. a != 10
            let (a,b,c,d) = handle_condition_refval_v2(">=", my_lval_decl, prefix1, my_rval_decl, val2);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            my_p4_metadecl = my_p4_metadecl + &d;
        }
        BinOpType::LTEQOp => {
            p4_func = "";
            // Eg. a != 10
            let (a,b,c,d) = handle_condition_refval_v2("<=", my_lval_decl, prefix1, my_rval_decl, val2);
            my_p4_control = my_p4_control + &a;
            my_p4_actions = my_p4_actions + &b;
            my_p4_commons = my_p4_commons + &c;
            my_p4_metadecl = my_p4_metadecl + &d;
        }
        _ => {
            p4_func = "";
            // Eg. a >= b
            panic!("Comparison on references not supported.\n");
        }
    }
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if p4_func.len() != 0 {
                if NEW_ACTION.load(Ordering::SeqCst) {
                    let (a, b, c) = get_NEW_ACTION();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({}.{},", TAB, p4_func, META_HEADER, my_lval_decl.id);
                if ordering {
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{},", my_rval_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{},",prefix1, my_rval_decl.id);
                        }
                    }
                    my_p4_actions = my_p4_actions + &format!("{});\n", val2);
                } else {
                    my_p4_actions = my_p4_actions + &format!("{},\n", val2);
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{});", my_rval_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{});",prefix1, my_rval_decl.id);
                        }
                    }
                }
                if NEW_ACTION.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }

        }

        TypeQualifier::Persistent => {
            // Register
            if p4_func.len() != 0 {
                if NEW_ACTION.load(Ordering::SeqCst) {
                    let (a, b, c) = get_NEW_ACTION();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({}.{},", TAB, p4_func, META_HEADER, my_lval_decl.id);
                if ordering {
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{},", my_rval_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{},",prefix1, my_rval_decl.id);
                        }
                    }
                    my_p4_actions = my_p4_actions + &format!("{});\n", val2);
                } else {
                    my_p4_actions = my_p4_actions + &format!("{},\n", val2);
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{});", my_rval_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{});",prefix1, my_rval_decl.id);
                        }
                    }
                }
                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {}.{});\n", TAB,
                        my_lval_decl.id, my_lval_index, META_HEADER, my_lval_decl.id);
                if NEW_ACTION.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }
        }

        TypeQualifier::Field => {
            // Metadata
            if p4_func.len() != 0 {
                if NEW_ACTION.load(Ordering::SeqCst) {
                    let (a, b, c) = get_NEW_ACTION();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }

                my_p4_actions = my_p4_actions + &format!("{}{}({},", TAB, p4_func, my_lval_decl.id);
                if ordering {
                    my_p4_actions = my_p4_actions + &format!("{},{});\n", my_rval_decl.id, val2);
                } else {
                    my_p4_actions = my_p4_actions + &format!("{},\n", val2);
                    my_p4_actions = my_p4_actions + &format!("{});", my_rval_decl.id);
                }

                if NEW_ACTION.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }
        }

        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }
    }

    return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}


pub fn handle_binop_vals_assignment<'a> (my_lval_decl : &VarDecl, my_lval_index : u64,
 val1 : u64, bin_op_type : BinOpType, val2 : u64, decl_map : &'a  HashMap<String, VarDecl> ) -> (String, String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut my_p4_metadecl : String = String::new();

    let mut p4_func = "";
    match bin_op_type {
        BinOpType::BooleanAnd => {
            p4_func = "bit_and";
        }
        BinOpType::BooleanOr => {
            p4_func = "bit_or";
        }
        BinOpType::Plus => {
            p4_func = "add";
        }
        BinOpType::Minus => {
            p4_func = "subtract";
        }
        BinOpType::Mul => {
            p4_func = "";
        }
        BinOpType::Div => {
            p4_func = "";
        }
        BinOpType::Modulo => {
            p4_func = "";
        }
        _ => {
            panic!("Not supporting complete value based condition.");
            //Something like z = a < b. This could be a pre-condition. will be handled separately.
        }
    }
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if p4_func.len() != 0 {
                if NEW_ACTION.load(Ordering::SeqCst) {
                    let (a, b, c) = get_NEW_ACTION();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({}.{},{}, {});\n", TAB, p4_func, META_HEADER, my_lval_decl.id, val1, val2);

                if NEW_ACTION.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }

        }

        TypeQualifier::Persistent => {
            // Register
            if p4_func.len() != 0 {
                if NEW_ACTION.load(Ordering::SeqCst) {
                    let (a, b, c) = get_NEW_ACTION();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({}.{},{}, {});\n", TAB, p4_func, META_HEADER, my_lval_decl.id, val1, val2);

                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {}.{});\n", TAB,
                        my_lval_decl.id, my_lval_index, META_HEADER, my_lval_decl.id);
                if NEW_ACTION.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }
        }

        TypeQualifier::Field => {
            // Metadata
            if p4_func.len() != 0 {
                if NEW_ACTION.load(Ordering::SeqCst) {
                    let (a, b, c) = get_NEW_ACTION();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({},{}, {});\n", TAB, p4_func, my_lval_decl.id, val1, val2);

                if NEW_ACTION.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }

        }

        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }
    }
    return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}

// handle_action_operand(l, index, reg3, decl_map)
pub fn handle_action_operand<'a> (my_lval_decl : &VarDecl,  my_lval_index : u64, operand : &Operand<'a>, decl_map : &'a  HashMap<String, VarDecl>) -> (String, String, String, String) {
    let mut my_rval_decl;
    let mut my_rval_index = 0;
    match operand {
        Operand::LValue(ref lval) => {
            match lval {
                LValue::Scalar(ref my_id) => {
                    my_rval_decl = get_decl(my_id.id_name, decl_map);
                }
                LValue::Array(ref my_id, ref box_index_op) => {
                    my_rval_decl = get_decl(my_id.id_name, decl_map);
                }
                LValue::Field(ref p, ref f) => {
                    let my_id = format!("{}.{}", p.id_name, f.id_name);
                    my_rval_decl = get_decl(&my_id, decl_map);
                }
            }
            return handle_ref_assignment(&my_lval_decl, my_lval_index, &my_rval_decl, my_rval_index, &handle_read_register_v2);
        }
        Operand::Value(ref rval_val) => {
            return handle_value_assignment(&my_lval_decl, my_lval_index, rval_val.value);
        }
    }
}

// reg1 = if_block_tmp_2 ? tmp_0_if_2 : reg1; (test1.np)
pub fn handle_ternary_assignment<'a> (my_lval_decl : &VarDecl, my_lval_index : u64,
 my_rval_decl : &VarDecl<'a>, operand1 : &Operand<'a>, operand2 : &Operand<'a>, decl_map : &'a  HashMap<String, VarDecl>  ) -> (String, String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut my_p4_metadecl : String = String::new();


    my_p4_control = my_p4_control + &format!("{}apply(table{:?});\n", TAB, TABLE_COUNT);
    let action1 = &format!("action{:?}", ACTION_COUNT);
    ACTION_COUNT.fetch_add(1, Ordering::SeqCst);
    let action2 = &format!("action{:?}", ACTION_COUNT);
    ACTION_COUNT.fetch_add(1, Ordering::SeqCst);
    NEW_ACTION.store(false, Ordering::SeqCst);
    my_p4_actions = my_p4_actions + &format!("action {} () {{\n", action1.to_string());

    // handle_action_operand(l, index, reg3, decl_map) (see first1.np)
    let (a,b,c,d) = handle_action_operand(my_lval_decl, my_lval_index, operand1, decl_map);
    my_p4_control = my_p4_control + &a;
    my_p4_actions = my_p4_actions + &b;
    my_p4_commons = my_p4_commons + &c;
    my_p4_metadecl = my_p4_metadecl + &d;
    my_p4_actions = my_p4_actions + &format!("}}\n");
    my_p4_actions = my_p4_actions + &format!("action {} () {{\n", action2.to_string());
    let (a,b,c,d) = handle_action_operand(my_lval_decl, my_lval_index, operand2, decl_map);
    my_p4_control = my_p4_control + &a;
    my_p4_actions = my_p4_actions + &b;
    my_p4_commons = my_p4_commons + &c;
    my_p4_metadecl = my_p4_metadecl + &d;
    my_p4_actions = my_p4_actions + &format!("}}\n");
    my_p4_commons = my_p4_commons + &format!("table table{:?} {{\n", TABLE_COUNT);
    my_p4_commons = my_p4_commons + &format!("{}reads {{\n", TAB);


    // match my_lval_decl.type_qualifier {
    //     TypeQualifier::Field => {
    //         my_p4_commons = my_p4_commons + &format!("{}{}{} : exact;\n{}}}\n", TAB, TAB, my_rval_decl.id, TAB);
    //     }
    //     _ => {
    //         my_p4_commons = my_p4_commons + &format!("{}{}{}.{} : exact;\n{}}}\n", TAB, TAB, META_HEADER, my_rval_decl.id, TAB);
    //     }
    // }

    my_p4_commons = my_p4_commons + &format!("{}{}{}.{} : exact;\n{}}}\n", TAB, TAB, META_HEADER, my_rval_decl.id, TAB);
    my_p4_commons = my_p4_commons + &format!("{}actions {{\n", TAB);
    my_p4_commons = my_p4_commons + &format!("{}{}{};\n", TAB, TAB, action1.to_string());
    my_p4_commons = my_p4_commons + &format!("{}{}{};\n", TAB, TAB, action2.to_string());
    my_p4_commons = my_p4_commons + &format!("{}}}\n", TAB);
    my_p4_commons = my_p4_commons + &format!("}}\n");
    TABLE_COUNT.fetch_add(1, Ordering::SeqCst);
    NEW_ACTION.store(true, Ordering::SeqCst);

    return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
}

pub fn get_decl<'a> (my_id : &str,  decl_map : &'a  HashMap<String, VarDecl>) -> &'a VarDecl<'a> {
    let my_lval : String = String::from(my_id);
    let my_decl_option = decl_map.get(&my_lval);
    match my_decl_option {
        Some(my_decl) => {
            return my_decl;
        }
        None => {
            panic!("Error: {} not declared?\n",my_lval);
        }
    }
}

pub fn handle_statement<'a> (my_statement :  &Statement<'a>, node_type : &DagNodeType<'a>,
    pre_condition : &Option<Statement<'a>>, decl_map : &'a  HashMap<String, VarDecl>,
      import_map : &HashMap<String, String>, packet_map : &HashMap<String, String>) -> (String, String, String, String) {
        let mut my_p4_control : String = String::new();
        let mut my_p4_actions : String = String::new();
        let mut my_p4_commons : String = String::new();
        let mut my_p4_metadecl : String = String::new();
        let mut my_lval_1 : String;
        let empty_decl = VarDecl {id : String::new(), var_info : VarInfo::BitArray(0,0), type_qualifier: TypeQualifier::Input};
        let mut my_lval_decl;
        let mut my_lval_index = 0;
        let mut my_rval_decl1 = &empty_decl;
        let mut my_rval1_index = 0;
        let mut my_rval_decl2;
        let mut my_rval2_index = 0;
        let mut is_rval1_val = false;
        let mut rval1_val = 0;
        println!("Handling Statement\n");
        println!("{:?}\n", my_statement);
        //println!("decl_map: {:?}\n", decl_map);

        // checking that lvalue of statement is declared
        match my_statement.lvalue {
            LValue::Scalar(ref my_id) => {
                let my_lval : String = String::from(my_id.id_name);
                //println!("Checking for {:?}\n", my_lval);
                let my_decl_option = decl_map.get(&my_lval);
                match my_decl_option {
                    Some(my_decl) => {
                        my_lval_decl = my_decl;
                    }
                    None => {
                        println!("Error: {} not declared?\n",my_lval);
                        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                    }
                }
            }
            LValue::Array(ref my_id, ref box_index_op) => {
                let my_lval : String = String::from(my_id.id_name);
                let my_decl_option = decl_map.get(&my_lval);
                match my_decl_option {
                    Some(my_decl) => {
                        my_lval_decl = my_decl;
                    }
                    None => {
                        println!("Error: {} not declared?\n",my_lval);
                        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                    }
                }
                my_lval_index = 0;
            }

            LValue::Field(ref p, ref f) => {
                let field = format!("{}.{}", p.id_name, f.id_name);
                let my_lval = packet_map.get(&field).unwrap();
                let my_decl_option = decl_map.get(&my_lval.clone());
                match my_decl_option {
                    Some(my_decl) => {
                        my_lval_decl = my_decl;
                    }
                    None => {
                        //Could be an imported field?
                        let my_decl_import_option = decl_map.get(&field);
                        match my_decl_option {
                            Some(my_decl) => {
                                my_lval_decl = my_decl;
                            }
                            None => {
                                println!("Error: {} not declared?\n", field);
                                return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                            }
                        }
                    }
                }
                my_lval_index = 0;
            }
        }
        //println!("lval_done\n");

        match my_statement.expr.op1 {
        // checking that op1 of statement is declared if it is an lvalue
            Operand::LValue(ref lval) => {
                // Could be an assignment or operation. e.g a = b or  a = b + c
                match lval {
                    LValue::Scalar(ref my_id2) => {
                        let my_rval1 : String = String::from(my_id2.id_name);
                        let my_decl_option = decl_map.get(&my_rval1);
                        match my_decl_option {
                            Some(my_decl) => {
                                my_rval_decl1 = my_decl;
                                // expr_right to be looked into
                            }
                            None => {
                                println!("Error: {} not declared?\n",my_rval1);
                                return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                            }
                        }
                    }

                    LValue::Field(ref p, ref f) => {
                        let field = format!("{}.{}", p.id_name, f.id_name);
                        let my_rval1_option = packet_map.get(&field);
                        match my_rval1_option {
                            Some(my_rval1) => {
                                let my_decl_option = decl_map.get(&my_rval1.clone());
                                match my_decl_option {
                                    Some(my_decl) => {
                                        my_rval_decl1 = my_decl;
                                        // expr_right to be looked into
                                    }
                                    None => {
                                        //Could be an imported field?
                                        panic!("Error: {} not declared?\n",my_rval1);
                                    }
                                }
                            }
                            None => {
                                let my_decl_import_option = decl_map.get(&field);
                                match my_decl_import_option {
                                    Some(my_decl) => {
                                        my_rval_decl1 = my_decl;
                                    }
                                    None => {
                                        println!("Error: {} not declared?\n", field);
                                        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                                    }
                                }
                            }
                        }
                    }

                    _ => {
                        //TODO. Do this for Array
                        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                    }
                }

            }
            Operand::Value(ref val) => {
                // This is a value assignment . e.g a = 1 or
                is_rval1_val = true;
                rval1_val = val.value;
                //return handle_value_assignment(&my_lval_decl, val.value);
            }
        }

        match my_statement.expr.expr_right {
            ExprRight::BinOp(bin_op_type, ref operand) => {
                // Operations like a = b + c
                match operand {
                    Operand::LValue(ref lval) => {
                        match lval {
                            LValue::Scalar(ref my_id3) => {
                                let my_lval3 : String = String::from(my_id3.id_name);
                                let my_decl_option = decl_map.get(&my_lval3);
                                match my_decl_option {
                                    Some(my_decl) => {
                                        my_rval_decl2 = my_decl;
                                        // expr_right to be looked into
                                    }
                                    None => {
                                        println!("Error: {} not declared?\n",my_lval3);
                                        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                                    }
                                }
                            }

                            LValue::Field(ref p, ref f) => {
                                let field = format!("{}.{}", p.id_name, f.id_name);
                                let my_lval3 = packet_map.get(&field).unwrap();
                                let my_decl_option = decl_map.get(&my_lval3.clone());
                                match my_decl_option {
                                    Some(my_decl) => {
                                        my_rval_decl2 = my_decl;
                                        // expr_right to be looked into
                                    }
                                    None => {
                                        println!("Error: {} not declared?\n", my_lval3);
                                        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                                    }
                                }
                            }

                            _ => {
                                //TODO. Do this for Array
                                return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
                            }
                        }
                        if is_rval1_val {
                            return handle_binop_refval_assignment(&my_lval_decl, my_lval_index, &my_rval_decl1, my_rval1_index, bin_op_type, rval1_val, decl_map, false);
                        } else {
                            return handle_binop_refs_assignment(&my_lval_decl, my_lval_index, &my_rval_decl1, my_rval1_index,  bin_op_type, &my_rval_decl2, my_rval2_index, decl_map);
                        }
                    }

                    Operand::Value(ref val2) => {
                        if is_rval1_val {
                            return handle_binop_vals_assignment(&my_lval_decl, my_lval_index, rval1_val, bin_op_type, val2.value, decl_map);
                        } else {
                            return handle_binop_refval_assignment(&my_lval_decl, my_lval_index, &my_rval_decl1, my_rval1_index, bin_op_type, val2.value, decl_map, true);
                        }
                    }
                }
            }

            ExprRight::Cond(ref operand1, ref operand2) => {
                // Operations like m = z?A:B;
                // TODO
                if !is_rval1_val {
                    return handle_ternary_assignment(&my_lval_decl, my_lval_index, my_rval_decl1, operand1, operand2, decl_map);
                } else {
                    panic!("Static ternary not supported for now.\n");
                }
            }

            ExprRight::Empty() => {
                // This is an assignment of meta/register/packet . e.g. a = b or a = 1
                if is_rval1_val {
                    return handle_value_assignment(&my_lval_decl, my_lval_index, rval1_val);
                } else {
                    return handle_ref_assignment(&my_lval_decl, my_lval_index, &my_rval_decl1, my_rval1_index, &handle_read_register);
                }
            }
        }

        return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
    }


// Ideally to get both ingress and egress parts of conversion [0] for ingress and [1] for egress and [2] for actions
pub fn get_p4_body_trans<'a> (node_type : &DagNodeType<'a>, pre_condition : &Option<Statement<'a>>,
 decl_map : &'a HashMap<String, VarDecl>, import_map : &HashMap<String, String>, packet_map : &HashMap<String, String>) -> (String, String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut my_p4_metadecl : String = String::new();

    match &node_type {
        // DagNodeType::Cond(my_cond) => {
        //     // TODO : If Statements
        //     panic!("If Conditional not supported yet!");
        //     //return (my_p4_control, my_p4_actions, my_p4_commons);
        // }
        DagNodeType::Stmt(my_statement) => {
            return handle_statement(&my_statement, node_type, pre_condition, decl_map, import_map, packet_map);
        }
        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons, my_p4_metadecl);
        }
    }
}


pub fn fill_p4code<'a> (import_map : &HashMap<String, String>, packet_map : &HashMap<String, String>,
    my_dag :  &mut Dag<'a>, pkt_tree : &Packets<'a>, my_packets : &Packets<'a>) {

    let mut decl_map : HashMap<String, VarDecl>= HashMap::new();

    // adding all include_dir packet fields to decl_map
    for packet in &pkt_tree.packet_vector {
        for field in &packet.packet_fields.field_vector {
            let mut my_vardecl : VarDecl;
            let name = &format!("{}.{}", packet.packet_id.id_name, field.identifier.id_name);
            let mut my_varinfo : VarInfo<'a>;
            match field.var_type.var_info {
                VarInfo::BitArray(bit_width, var_size) => {
                    my_varinfo = VarInfo::BitArray(bit_width, var_size);
                }
                _ => {
                    my_varinfo = VarInfo::BitArray(0, 0);
                }
            }
            let my_typequalifier : TypeQualifier = field.var_type.type_qualifier;
            my_vardecl = VarDecl{id : name.to_string(), var_info : my_varinfo, type_qualifier : my_typequalifier};
            decl_map.insert(name.to_string(), my_vardecl);
        }
    }

    // adding all user-defined packet fields to decl_map
    for packet in &my_packets.packet_vector {
        for field in &packet.packet_fields.field_vector {
            let mut my_vardecl : VarDecl;
            let name = &format!("{}.{}", packet.packet_id.id_name, field.identifier.id_name);
            let mut my_varinfo : VarInfo<'a>;
            match field.var_type.var_info {
                VarInfo::BitArray(bit_width, var_size) => {
                    my_varinfo = VarInfo::BitArray(bit_width, var_size);
                }
                _ => {
                    my_varinfo = VarInfo::BitArray(0, 0);
                }
            }
            let my_typequalifier : TypeQualifier = field.var_type.type_qualifier;
            my_vardecl = VarDecl{id : name.to_string(), var_info : my_varinfo, type_qualifier : my_typequalifier};
            decl_map.insert(name.to_string(), my_vardecl);
        }
    }

    // adding all  device metadata to decl_map
    for (sluice_meta, device_meta) in import_map.iter() {
        let mut my_vardecl : VarDecl;
        let mut my_varinfo : VarInfo<'a>;
        let mut my_id = "standard_metadata";
        match device_meta.as_str() {
            "timestamp_rx" => {
                my_id = "intrinsic_metadata.ingress_global_timestamp";
                my_varinfo = VarInfo::BitArray(32, 1);
            }
            "timestamp_ingress" => {
                my_id = "intrinsic_metadata.ingress_global_timestamp";
                my_varinfo = VarInfo::BitArray(32, 1);
            }
            "timestamp_egress" => {
                my_id = "intrinsic_metadata.ingress_global_timestamp";
                my_varinfo = VarInfo::BitArray(32, 1);
            }
            "timestamp_tx" => {
                my_id = "intrinsic_metadata.ingress_global_timestamp";
                my_varinfo = VarInfo::BitArray(32, 1);
            }
            "ingress_port" => {
                my_id = "standard_metadata.ingress_port";
                my_varinfo = VarInfo::BitArray(9, 1);
            }
            "egress_port" => {
                my_id = "standard_metadata.egress_spec";
                my_varinfo = VarInfo::BitArray(9, 1);
            }
            "packet_length" => {
                my_id = "standard_metadata.packet_length";
                my_varinfo = VarInfo::BitArray(32, 1);
            }
            "enq_qdepth" => {
                my_id = "queueing_metadata.enq_qdepth";
                my_varinfo = VarInfo::BitArray(19, 1);
            }
            "deq_qdepth" => {
                my_id = "queueing_metadata.deq_qdepth";
                my_varinfo = VarInfo::BitArray(19, 1);
            }
            _ => {
                panic!("Currently not supported\n");
            }
        }
        let my_typequalifier : TypeQualifier = TypeQualifier::Field;
        my_vardecl = VarDecl{id : String::from(my_id), var_info : my_varinfo, type_qualifier : my_typequalifier};
        decl_map.insert(sluice_meta.to_string(), my_vardecl);
    }



    for mut my_dag_node in &mut my_dag.dag_vector {
        my_dag_node.p4_code.p4_header = get_p4_header_trans(&my_dag_node.node_type);
        // Insert nodes to decl_map
        match my_dag_node.node_type {
            DagNodeType::Decl(ref my_decl) => {
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
    }
    println!("decl_map : {:?}\n", decl_map);
    for mut my_dag_node in &mut my_dag.dag_vector {
        let (a, b, c, d) = get_p4_body_trans(&my_dag_node.node_type, &my_dag_node.pre_condition, &decl_map, import_map, &packet_map);
        //println!("meta header : {}\n", d);
        my_dag_node.p4_code.p4_control = a;
        my_dag_node.p4_code.p4_actions = b;
        my_dag_node.p4_code.p4_commons = c;
        my_dag_node.p4_code.p4_header.meta.push_str(d.as_str());
    }
}

fn gen_p4_includes<'a> ( p4_file : &mut File) {
    //p4_file.write(b"#include <core.p4>\n#include <v1model.p4>\n");
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

fn gen_p4_headers<'a> (my_dag : &Dag<'a>, my_packets : &Packets<'a>, p4_file : &mut File) {
    // TODO
    let mut contents : String = String::new();
    contents = contents + "#define ETHERTYPE_IPV4 0x0800\n";
    contents = contents + "#define IP_PROTOCOLS_TCP 6\n";
    contents = contents + "#define IP_PROTOCOLS_UDP 17\n";
    contents = contents + "#define IP_PROTOCOLS_TCP 6\n";


    contents = contents + &format!("header_type ethernet_t {{
    fields {{
        dstAddr : 48;
        srcAddr : 48;
        etherType : 16;
    }}\n}}\n");
    contents = contents + &format!("header_type ipv4_t {{
    fields {{
        version : 4;
        ihl : 4;
        diffserv : 8;
        totalLen : 16;
        identification : 16;
        flags : 3;
        fragOffset : 13;
        ttl : 8;
        protocol : 8;
        hdrChecksum : 16;
        srcAddr : 32;
        dstAddr: 32;
    }}\n}}\n");

    contents = contents + &format!("header_type tcp_t {{
    fields {{
        srcPort : 16;
        dstPort : 16;
        seqNo : 32;
        ackNo : 32;
        dataOffset : 4;
        res : 4;
        flags : 8;
        window : 16;
        checksum : 16;
        urgentPtr : 16;
    }}\n}}\n");
    contents = contents + &format!("header_type udp_t {{
    fields {{
        srcPort : 16;
        dstPort : 16;
        len : 16;
        checksum : 16;
    }}\n}}\n");
    //
    for my_packet in &my_packets.packet_vector {
        if my_packet.packet_fields.field_vector.len() != 0 {
            contents = contents + &format!("header_type {}_t {{\n", my_packet.packet_id.id_name);
            contents = contents + &format!("{}fields {{\n", TAB);
        }
        for my_field in &my_packet.packet_fields.field_vector {
            match my_field.var_type.var_info {
                VarInfo::BitArray(size, no) => {
                    contents = contents + &format!("{}{}{} : {};\n", TAB, TAB, my_field.identifier.id_name, size)
                }
                _ => {
                    println!("Un-supported entry in packet field!");
                }
            }
        }
        if my_packet.packet_fields.field_vector.len() != 0 {
            contents = contents + &format!("{}}}\n}}\n", TAB);
            contents = contents + &format!("header {}_t {};\n", my_packet.packet_id.id_name, my_packet.packet_id.id_name);
        }
    }

    // let my_option  = my_packets.packet_vector.get(0);
    // match my_option {
    //     Some(my_packet) => {
    //         for my_field in &my_packet.packet_fields.field_vector {
    //
    //         }
            // match my_field.var_type.var_info {
            //     VarInfo::BitArray(size, no) => {
            //          contents = contents + &format!("{} : {};\n", my_field.identifier.id_name, size)
            //     }
            //     _ => {
            //         println!("Un-supported entry in packet field!");
            //     }
            // }
    //     }
    //     _ => {}
    // }
    contents = contents + &format!("header ethernet_t ethernet;\n");
    contents = contents + &format!("header ipv4_t ipv4;\n");
    contents = contents + &format!("header tcp_t tcp;\n");
    contents = contents + &format!("header udp_t udp;\n");

    p4_file.write(contents.as_bytes());
}


fn gen_p4_routing_tables<'a> (p4_file : &mut File) {
    let mut contents : String = String::new();
    contents = contents + &format!("
field_list ipv4_checksum_list {{
        ipv4.version;
        ipv4.ihl;
        ipv4.diffserv;
        ipv4.totalLen;
        ipv4.identification;
        ipv4.flags;
        ipv4.fragOffset;
        ipv4.ttl;
        ipv4.protocol;
        ipv4.srcAddr;
        ipv4.dstAddr;
}}

field_list_calculation ipv4_checksum {{
    input {{
        ipv4_checksum_list;
    }}
    algorithm : csum16;
    output_width : 16;
}}
calculated_field ipv4.hdrChecksum  {{
    verify ipv4_checksum;
    update ipv4_checksum;
}}


action _drop() {{
    drop();
}}

action ipv4_forward(dstAddr, port) {{
    modify_field(udp.checksum, 0);
    modify_field(standard_metadata.egress_spec, port);
    modify_field(ethernet.srcAddr, ethernet.dstAddr);
    modify_field(ethernet.dstAddr, dstAddr);
    subtract_from_field(ipv4.ttl, 1);
}}

table ipv4_lpm {{
    reads {{
        ipv4.dstAddr : lpm;
    }}
    actions {{
        ipv4_forward;
        _drop;
    }}
    size: 1024;
}}\n\n", );

    p4_file.write(contents.as_bytes());
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
    p4_file.write(b"metadata metadata_t mdata;\n");

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

fn gen_p4_parser<'a> (my_dag : &Dag<'a>, my_packets : &Packets<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();
    let my_option  = my_packets.packet_vector.get(0);
    let mut parse_my_ethpacket : String = String::new();
    let mut parse_my_ipv4packet : String = String::new();
    let mut parse_my_udppacket : String = String::new();
    let mut parse_my_tcppacket : String = String::new();

    match my_option {
        Some(my_packet) => {
            println!("Header base : {}\n", my_packet.packet_base.id_name);
            let my_base = my_packet.packet_base.id_name;
            let my_condition = &my_packet.packet_parser_condition;
            match my_base {
                "ethernet" => {
                    match my_condition {
                        PacketParserCondition::ParserCondition(id, val) => {
                            match id.id_name {
                                "etherType" => {
                                    parse_my_ethpacket = parse_my_ethpacket + &format!("{}{} : parse_{};", TAB, val.value, my_packet.packet_id.id_name);
                                }
                                _ => {
                                    panic!("Conditional Parsing over Ethernet supported for only etherType\n");
                                }
                            }
                        }
                        Empty => {
                            panic!("Conditional Parsing necessary on Ethernet Header\n");
                        }
                    }
                }
                "ipv4" => {
                    match my_condition {
                        PacketParserCondition::ParserCondition(id, val) => {
                            match id.id_name {
                                "protocol" => {
                                    parse_my_ipv4packet = parse_my_ipv4packet + &format!("{:?} : parse_{};", val.value, my_packet.packet_id.id_name);
                                }
                                _ => {
                                    panic!("Conditional Parsing over IPV4 supported for only protocol type\n");
                                }
                            }
                        }
                        Empty => {
                            panic!("Conditional Parsing necessary on IPV4 Header\n");
                        }
                    }
                }
                "udp" => {

                    match my_condition {
                        PacketParserCondition::ParserCondition(id, val) => {
                            match id.id_name {
                                "srcPort" => {
                                    parse_my_udppacket = parse_my_udppacket + &format!("{:?} : parse_{};", val.value, my_packet.packet_id.id_name);
                                }
                                _ => {
                                    panic!("Conditional Parsing over UDP supported for only srcPort type\n");
                                }
                            }
                        }
                        Empty => {
                            panic!("Conditional Parsing necessary on UDP Header\n");
                        }
                    }
                }
                _ => {
                    panic!("Need to have a derivative!\n");
                }
            }
        }
        _ => {}
    }

    contents = contents + &format!("\nparser start {{
    return parse_ethernet;\n}}\n\nparser parse_ethernet {{
    extract(ethernet);
    return select(latest.etherType) {{
        ETHERTYPE_IPV4 : parse_ipv4;\n");
    if parse_my_ethpacket.len() == 0 {
        contents = contents + &format!("{}{}default: ingress;\n", TAB, TAB);
    } else {
        contents = contents + &format!("        {}\n", parse_my_ethpacket);
    }

    contents = contents + &format!("{}}}\n}}\n\nparser parse_ipv4 {{
    extract(ipv4);
    return select(latest.protocol) {{
        IP_PROTOCOLS_TCP : parse_tcp;
        IP_PROTOCOLS_UDP : parse_udp;", TAB);

    if parse_my_ipv4packet.len() == 0 {
        contents = contents + &format!("\n{}{}default: ingress;\n", TAB, TAB);
    } else {
        contents = contents + &format!("        {}\n\n", parse_my_ipv4packet);
    }

    contents = contents + &format!("{}}}\n}}\n\nparser parse_tcp {{
    extract(tcp);
    return ingress;\n}}\n", TAB);

    if parse_my_udppacket.len() == 0 {
        contents = contents + &format!("\nparser parse_udp {{
    extract(udp);
    return ingress;\n}}\n");
    } else {
        contents = contents + &format!("\nparser parse_udp {{
    extract(udp);
    return select(latest.srcPort) {{\n");
        contents = contents + &format!("{}{}{}\n", TAB, TAB, parse_my_udppacket);
        contents = contents + &format!("{}{}default: ingress;\n{}}}\n}}\n\n", TAB, TAB, TAB);
    }

    match my_option {
        Some(my_packet) => {
            contents = contents + &format!("parser parse_{} {{
    extract({});
    return ingress;\n}}\n\n", my_packet.packet_id.id_name, my_packet.packet_id.id_name);
        }
        _ => {}
    }

    p4_file.write(contents.as_bytes());
}


fn gen_p4_body<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    let mut contents : String = String::new();

    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_actions.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_actions;
        }
    }

    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_commons.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_commons;
        }
    }

    // TODO : Identify placement in ingress/egress
    contents = contents + &format!("control ingress {{\n");
    for my_dag_node in &my_dag.dag_vector {
        if (my_dag_node.p4_code.p4_control.len() != 0) {
            contents = contents + &my_dag_node.p4_code.p4_control;
        }
    }
    // calling ipv4_lpm for routing
    contents = contents + "
    if(valid(ipv4) and ipv4.ttl > 0) {
        apply(ipv4_lpm);
    }\n" ;
    contents = contents + &format!("}}\n");
    contents = contents + &format!("control egress {{\n");
    // for my_dag_node in &my_dag.dag_vector {
    //     if (my_dag_node.p4_code.p4_control.len() != 0) {
    //         contents = contents + &my_dag_node.p4_code.p4_control;
    //     }
    // }
    contents = contents + &format!("}}\n");
    p4_file.write(contents.as_bytes());
}

pub fn gen_p4_code<'a> (snippet_name : &str , my_packets : &Packets<'a>, snippet_dag : &Dag<'a>){
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
    gen_p4_headers(&snippet_dag, my_packets, &mut p4_file);
    gen_p4_parser(&snippet_dag, my_packets, &mut p4_file);
    gen_p4_routing_tables(&mut p4_file);
    gen_p4_metadata(&snippet_dag, &mut p4_file);
    gen_p4_registers(&snippet_dag, &mut p4_file);
    //gen_p4_actions(&snippet_dag, &mut p4_file);
    gen_p4_body(&snippet_dag, &mut p4_file);
}


// TODO : handle packet fields
pub fn gen_control_plane_commands<'a> (snippet_name : &str , my_packets : &Packets<'a>, snippet_dag : &Dag<'a>){

    let command_filename : String = format!("bmv2_sim/commands/{}.txt", snippet_name);
    let path = Path::new(command_filename.as_str());
    let display  = path.display();
    let mut command_file = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(command_file) => command_file,
    };

    let mut decl_map : HashMap<String, VariableDecl>= HashMap::new();
    let mut contents : String = String::new();

    for dagnode in &snippet_dag.dag_vector {

        match &dagnode.node_type {
            DagNodeType::Decl(var_decl) => {
                decl_map.insert(var_decl.identifier.id_name.to_string(), var_decl.clone());
            }

            DagNodeType::Stmt(my_statement) => {
                match &my_statement.expr.expr_right {
                    ExprRight::Cond(_,_) => {
                        match my_statement.expr.op1 {
                            Operand::LValue(ref lval) => {
                                match lval {
                                    LValue::Scalar(ref my_id) => {
                                        let table_index = decl_map.get(my_id.id_name.clone());
                                        match table_index {
                                            Some(my_decl) => {
                                                match my_decl.var_type.var_info {
                                                    VarInfo::BitArray(1, 1) => {
                                                        // parse out action and table names from p4_commons
                                                        let re1 = Regex::new(r"table\d+").unwrap();
                                                        let re2 = Regex::new(r"action\d+").unwrap();

                                                        let mut table_array = Vec::new();
                                                        for cap in re1.captures_iter(&dagnode.p4_code.p4_commons) {
                                                            let ref table_str = cap.get(0).unwrap().as_str();
                                                            table_array.push(table_str.clone());
                                                        }

                                                        let mut action_array = Vec::new();
                                                        for cap in re2.captures_iter(&dagnode.p4_code.p4_commons) {
                                                            let ref action_str = cap.get(0).unwrap().as_str();
                                                            action_array.push(action_str.clone());
                                                        }

                                                        contents = contents + &format!("table_add {} {} 1 => \n", table_array[0], action_array[0]);
                                                        contents = contents + &format!("table_add {} {} 0 => \n", table_array[0], action_array[1]);
                                                    }
                                                    //TODO : add support for 32 bit table indices and tables with multiple read vars
                                                    _ => {panic!("Unsupported table index type!");}

                                                }
                                            }
                                            None => {
                                                println!("Error: {} not declared?\n",my_id.id_name);
                                            }

                                        }
                                    }

                                    // TODO : handle tables for array, value and packet field operands
                                    LValue::Array(ref my_id, ref box_index_op) => {}

                                    _ => {
                                        panic!("Unsuppoted operation!");
                                    }
                                }
                            }

                            Operand::Value(ref rval_val) => {}
                        }
                    }

                    _ => {
                        // parse out action and table names from p4_commons
                        let re1 = Regex::new(r"table\d+").unwrap();
                        let re2 = Regex::new(r"action\d+").unwrap();

                        let mut table_array = Vec::new();
                        for cap in re1.captures_iter(&dagnode.p4_code.p4_commons) {
                            let ref table_str = cap.get(0).unwrap().as_str();
                            table_array.push(table_str.clone());
                        }

                        let mut action_array = Vec::new();
                        for cap in re2.captures_iter(&dagnode.p4_code.p4_commons) {
                            let ref action_str = cap.get(0).unwrap().as_str();
                            action_array.push(action_str.clone());
                        }

                        // check that exactly 1 action is run per table
                        if table_array.len() == action_array.len() {
                            for (x, action) in action_array.iter().enumerate() {
                                contents = contents + &format!("table_set_default {} {}\n", table_array[x], action);
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }

    command_file.write(contents.as_bytes());
}
