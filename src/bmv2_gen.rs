
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

static action_count : AtomicUsize = AtomicUsize::new(1);
static table_count : AtomicUsize = AtomicUsize::new(1);
static operation_count : AtomicUsize = AtomicUsize::new(1);
static new_action : AtomicBool = AtomicBool::new(true);

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

pub fn get_new_action () -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    my_p4_control = my_p4_control + &format!("{}apply(table{:?});\n", TAB, table_count);
    my_p4_actions = my_p4_actions + &format!("action action{:?} () {{\n", action_count);
    my_p4_commons = my_p4_commons + &format!("table table{:?} {{\n", table_count);
    my_p4_commons = my_p4_commons + &format!("{}actions {{\n", TAB);
    my_p4_commons = my_p4_commons + &format!("{}{}action{:?};\n", TAB, TAB, table_count);
    my_p4_commons = my_p4_commons + &format!("{}}}\n", TAB);
    my_p4_commons = my_p4_commons + &format!("}}\n");
    action_count.fetch_add(1, Ordering::SeqCst);
    table_count.fetch_add(1, Ordering::SeqCst);
    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_read_register (my_decl : &VarDecl, my_index : u64) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let (a,b,c) = get_new_action();
    my_p4_control = a; my_p4_actions = b; my_p4_commons = c;
    my_p4_actions = my_p4_actions + &format!("{}register_read({}.{}, {}, {});\n", TAB,
        META_HEADER, my_decl.id, my_decl.id, my_index);
    my_p4_actions = my_p4_actions + &format!("}}\n");

    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_value_assignment<'a> ( my_lval_decl : &VarDecl, my_lval_index : u64,  val : u64) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();

    println!("handling value assignment for  :{:?}\n", my_lval_decl);
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if new_action.load(Ordering::SeqCst) {
                let (a, b, c) = get_new_action();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, {});\n", TAB, META_HEADER, my_lval_decl.id, val);
            if new_action.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
        TypeQualifier::Persistent => {
            // Register
            if new_action.load(Ordering::SeqCst) {
                let (a, b, c) = get_new_action();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {});\n", TAB, my_lval_decl.id, val, my_lval_index);
            if new_action.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
    }
}

pub fn handle_ref_assignment<'a> (my_lval_decl : &VarDecl, my_lval_index : u64, my_rval_decl : &VarDecl, my_rval_index : u64) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();
    let mut prefix = "";
    match my_rval_decl.type_qualifier {
        TypeQualifier::Persistent => {
            // If register, then first need to read the register val to meta.
            let (a,b,c) = handle_read_register(my_rval_decl, my_rval_index);
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
            if new_action.load(Ordering::SeqCst) {
                let (a, b, c) = get_new_action();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;            }
            if prefix.len()!= 0 {
                my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, {}.{});\n", TAB,
                    META_HEADER, my_lval_decl.id, prefix, my_rval_decl.id);
            } else {
                my_p4_actions = my_p4_actions + &format!("{}modify_field({}.{}, {});\n", TAB,
                    META_HEADER, my_lval_decl.id, my_rval_decl.id);
            }
            if new_action.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }

            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
        TypeQualifier::Persistent => {
            // Register
            if new_action.load(Ordering::SeqCst) {
                let (a, b, c) = get_new_action();
                my_p4_control = my_p4_control + &a;
                my_p4_actions = my_p4_actions + &b;
                my_p4_commons = my_p4_commons + &c;
            }
            if prefix.len()!= 0 {
                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}.{}, {});\n", TAB,
                    my_lval_decl.id, prefix, my_rval_decl.id, my_rval_index);
            } else {
                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {});\n", TAB,
                    my_lval_decl.id, my_rval_decl.id, my_rval_index);
            }
            if new_action.load(Ordering::SeqCst) {
                my_p4_actions = my_p4_actions + &format!("}}\n");
            }
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
    }
}

pub fn handle_binop_refs_assignment<'a> (my_lval_decl : &VarDecl,  my_lval_index : u64, my_rval1_decl : &VarDecl, my_rval1_index : u64,
    bin_op_type : BinOpType, my_rval2_decl : &VarDecl, my_rval2_index : u64, decl_map : &'a  HashMap<String, VarDecl> ) -> (String, String, String) {
        let mut my_p4_control : String = String::new();
        let mut my_p4_actions : String = String::new();
        let mut my_p4_commons : String = String::new();
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
            }
            _ => {
                //Something like z = a < b. This could be a pre-condition. will be handled separately.
            }
        }
        match my_lval_decl.type_qualifier {
            TypeQualifier::Transient => {
                // Metadata
                if p4_func.len() != 0 {
                    if new_action.load(Ordering::SeqCst) {
                        let (a, b, c) = get_new_action();
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
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{});\n", my_rval2_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{});\n",prefix2, my_rval2_decl.id);
                        }
                    }
                    if new_action.load(Ordering::SeqCst) {
                        my_p4_actions = my_p4_actions + &format!("}}\n");
                    }
                }

            }
            TypeQualifier::Persistent => {
                // Register
                if p4_func.len() != 0 {
                    if new_action.load(Ordering::SeqCst) {
                        let (a, b, c) = get_new_action();
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
                    match prefix1.len() {
                        0 => {
                            my_p4_actions = my_p4_actions + &format!("{});\n", my_rval2_decl.id);
                        }
                        _ => {
                            my_p4_actions = my_p4_actions + &format!("{}.{});\n",prefix2, my_rval2_decl.id);
                        }
                    }
                    my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}.{}, {});\n", TAB,
                            my_lval_decl.id, META_HEADER, my_lval_decl.id, my_lval_index);
                    if new_action.load(Ordering::SeqCst) {
                        my_p4_actions = my_p4_actions + &format!("}}\n");
                    }
                }
            }
            _ => {
                return (my_p4_control, my_p4_actions, my_p4_commons);
            }
        }

        return (my_p4_control, my_p4_actions, my_p4_commons);
}

//Direction : true  for ref <op> val, false for val <op> ref
pub fn handle_binop_refval_assignment<'a> (my_lval_decl : &VarDecl,  my_lval_index : u64,
    my_rval_decl : &VarDecl,  my_rval_index : u64,bin_op_type : BinOpType, val2 : u64,
     decl_map : &'a  HashMap<String, VarDecl>, ordering : bool) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();

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
        _ => {
            //Something like z = a < b. This could be a pre-condition. will be handled separately.
        }
    }
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if p4_func.len() != 0 {
                if new_action.load(Ordering::SeqCst) {
                    let (a, b, c) = get_new_action();
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
                if new_action.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }

        }
        TypeQualifier::Persistent => {
            // Register
            if p4_func.len() != 0 {
                if new_action.load(Ordering::SeqCst) {
                    let (a, b, c) = get_new_action();
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
                if new_action.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }
        }
        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
    }

    return (my_p4_control, my_p4_actions, my_p4_commons);
}


pub fn handle_binop_vals_assignment<'a> (my_lval_decl : &VarDecl, my_lval_index : u64,
 val1 : u64, bin_op_type : BinOpType, val2 : u64, decl_map : &'a  HashMap<String, VarDecl> ) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();

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
            //Something like z = a < b. This could be a pre-condition. will be handled separately.
        }
    }
    match my_lval_decl.type_qualifier {
        TypeQualifier::Transient => {
            // Metadata
            if p4_func.len() != 0 {
                if new_action.load(Ordering::SeqCst) {
                    let (a, b, c) = get_new_action();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({}.{},{}, {});\n", TAB, p4_func, META_HEADER, my_lval_decl.id, val1, val2);

                if new_action.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }

        }
        TypeQualifier::Persistent => {
            // Register
            if p4_func.len() != 0 {
                if new_action.load(Ordering::SeqCst) {
                    let (a, b, c) = get_new_action();
                    my_p4_control = my_p4_control + &a;
                    my_p4_actions = my_p4_actions + &b;
                    my_p4_commons = my_p4_commons + &c;
                }
                my_p4_actions = my_p4_actions + &format!("{}{}({}.{},{}, {});\n", TAB, p4_func, META_HEADER, my_lval_decl.id, val1, val2);

                my_p4_actions = my_p4_actions + &format!("{}register_write({}, {}, {}.{});\n", TAB,
                        my_lval_decl.id, my_lval_index, META_HEADER, my_lval_decl.id);
                if new_action.load(Ordering::SeqCst) {
                    my_p4_actions = my_p4_actions + &format!("}}\n");
                }
            }
        }
        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
    }
    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_ternary_assignment<'a> (my_lval_decl : &VarDecl, my_lval_index : u64,
 pre_condition : &Option<Statement<'a>>, operand1 : &Operand<'a>, operand2 : &Operand<'a>) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();

    return (my_p4_control, my_p4_actions, my_p4_commons);
}

pub fn handle_statement<'a> (my_statement :  &Statement<'a>, node_type : &DagNodeType<'a>,
    pre_condition : &Option<Statement<'a>>, decl_map : &'a  HashMap<String, VarDecl> ) -> (String, String, String) {
        let mut my_p4_control : String = String::new();
        let mut my_p4_actions : String = String::new();
        let mut my_p4_commons : String = String::new();
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
        //println!("Handling Statement\n");
        //println!("{:?}\n", my_statement);
        //println!("decl_map: {:?}\n", decl_map);
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
                        return (my_p4_control, my_p4_actions, my_p4_commons);
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
                        return (my_p4_control, my_p4_actions, my_p4_commons);
                    }
                }
                my_lval_index = 0;
            }
            _ => {
                return (my_p4_control, my_p4_actions, my_p4_commons);
            }
        }

        match my_statement.expr.op1 {
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
                                return (my_p4_control, my_p4_actions, my_p4_commons);
                            }
                        }
                    }
                    _ => {
                        //TODO. Do this for Array
                        return (my_p4_control, my_p4_actions, my_p4_commons);
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
                                        return (my_p4_control, my_p4_actions, my_p4_commons);
                                    }
                                }
                            }
                            _ => {
                                //TODO. Do this for Array
                                return (my_p4_control, my_p4_actions, my_p4_commons);
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
                return handle_ternary_assignment(&my_lval_decl, my_lval_index, pre_condition, operand1, operand2);
            }
            ExprRight::Empty() => {
                // This is an assignment of meta/register/packet . e.g. a = b or a = 1
                if is_rval1_val {
                    return handle_value_assignment(&my_lval_decl, my_lval_index, rval1_val);
                } else {
                    return handle_ref_assignment(&my_lval_decl, my_lval_index, &my_rval_decl1, my_rval1_index);
                }
            }
        }

        return (my_p4_control, my_p4_actions, my_p4_commons);
    }


// Ideally to get both ingress and egress parts of conversion [0] for ingress and [1] for egress and [2] for actions
pub fn get_p4_body_trans<'a> (node_type : &DagNodeType<'a>, pre_condition : &Option<Statement<'a>>,
 decl_map : &'a HashMap<String, VarDecl>) -> (String, String, String) {
    let mut my_p4_control : String = String::new();
    let mut my_p4_actions : String = String::new();
    let mut my_p4_commons : String = String::new();

    match &node_type {
        DagNodeType::Cond(my_cond) => {
            // TODO : If Statements
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
        DagNodeType::Stmt(my_statement) => {
            return handle_statement(&my_statement, node_type, pre_condition, decl_map);
        }
        _ => {
            return (my_p4_control, my_p4_actions, my_p4_commons);
        }
    }
}

pub fn fill_p4code<'a> (my_dag :  &mut Dag<'a>) {
    let mut decl_map : HashMap<String, VarDecl>= HashMap::new();
    for mut my_dag_node in &mut my_dag.dag_vector {
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
        println!("declMap : {:?}\n", decl_map);
    }
    for mut my_dag_node in &mut my_dag.dag_vector {
        let (a, b, c) = get_p4_body_trans(&my_dag_node.node_type, &my_dag_node.pre_condition, &decl_map);
        my_dag_node.p4_code.p4_control = a;
        my_dag_node.p4_code.p4_actions = b;
        my_dag_node.p4_code.p4_commons = c;
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
fn gen_p4_headers<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    // TODO
    let mut contents : String = String::new();
    contents = contents + &format!("header_type ethernet_t {{\n{}fields {{\n{}{}dstAddr : 48;\n{}{}srcAddr : 48;\n{}{}etherType : 16;\n{}}}\n}}\n",
    TAB,TAB,TAB,TAB,TAB,TAB,TAB,TAB);
    contents = contents + &format!("header ethernet_t ethernet;\n");
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

fn gen_p4_parser<'a> (my_dag : &Dag<'a>, p4_file : &mut File) {
    // TODO
    let mut contents : String = String::new();
    contents = contents + &format!("parser start {{\n{}return parse_ethernet;\n }}\nparser parse_ethernet {{\n{}extract(ethernet);\n{}return ingress;\n}}\n",
     TAB, TAB, TAB);
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
