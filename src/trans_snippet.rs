// Translation from Sluice to p4 for each snippet.
// This works by first constructing a DAG.
use grammar::*;
use std::collections::HashMap;

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
    pub next_nodes : Vec<&'a DagNode<'a>>,
    pub prev_nodes : Vec<&'a DagNode<'a>>
}

// For now, using a simplistic DAG dc using vectors.
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Dag<'a> {
    pub dag_vector : Vec<DagNode<'a>>
}

pub fn getIdentifiers<'a> (my_operand : &'a Operand<'a>) -> Vec<&'a str> {
    match &my_operand {
        Operand::LValue(ref lval) => {
            let mut nex_vec = lval.get_string_vec();
            return nex_vec;
        },
        _ =>  { return Vec::new(); }
    }

}

// pub fn computeDagRelationLval<'a> (my_dag : &mut Dag<'a>, decl_map : &mut HashMap<&str, &'a mut DagNode<'a>>,
//  lval : &LValue<'a>, my_dag_node : &'a mut DagNode<'a>) {
//     let my_vec_ids = lval.get_string_vec();
//     for my_id in my_vec_ids {
//         println!("id :{:?}--", my_id);
//         if decl_map.contains_key(my_id) {
//             let my_option = decl_map.get_mut(my_id);
//             match my_option {
//                 Some(my_parent_dag_node) => {
//                     println!("dag_node {:?}", my_dag_node);
//                     if !&my_parent_dag_node.next_nodes.contains(&my_dag_node) {
//                         println!("Adding to dag..");
//                         &my_parent_dag_node.next_nodes.push(my_dag_node);
//                     }
//                 }
//                 None => {}
//             }
//
//         }
//     }
//
// }

// pub fn computeDagRelationOp<'a> (my_dag : &mut Dag<'a>, decl_map : &mut HashMap<&str, &mut DagNode<'a>>,
//  op : &Operand<'a>, my_dag_node : &mut DagNode<'a>) {
//     match &op {
//     Operand::LValue(ref lval) => {
//         computeDagRelationLval(my_dag, decl_map, lval, my_dag_node);
//     },
//         _ =>  {  }
//     }
// }
// Construct a single DAG
pub fn trans_my_snippet<'a> (my_snippet: &'a Snippet<'a>, my_dag : &Dag<'a>) {
    let mut my_dag_start_node : DagNode<'a>;
    let mut decl_map : HashMap<&str, DagNode>= HashMap::new();
    //First, process variable decls
    for my_variable_decl in &my_snippet.variable_decls.decl_vector {
        println!("Variable declarations: {:?}\n", my_variable_decl);
        //let my_dag_start_node = DagStartNode {variable_decl : my_variable_decl,  next_nodes : Vec::new()};
        my_dag_start_node = DagNode {node_type : DagNodeType::Decl(my_variable_decl),
              next_nodes : Vec::new(), prev_nodes : Vec::new()};
        decl_map.insert(&my_variable_decl.identifier.id_name, my_dag_start_node);

    }

    // Next, process statements, for now ignoring if block.
    for my_if_block in &my_snippet.ifblocks.ifblock_vector {
        for my_statement in &my_if_block.statements.stmt_vector {
            let mut my_dag_node = DagNode {node_type: DagNodeType::Stmt(&my_statement),
                 next_nodes: Vec::new(), prev_nodes: Vec::new()};
            //computeDagRelationLval(my_dag, &mut decl_map, &my_statement.lvalue, &mut my_dag_node);
            // let op1 = &my_statement.expr.op1;
            // computeDagRelationOp(my_dag, &mut decl_map, &op1, &mut my_dag_node);
            // match &my_statement.expr.expr_right {
            //     ExprRight::BinOp(btype, op2) => {
            //         println!("Statement : {:?}\n", &my_statement);
            //         computeDagRelationOp(my_dag, &mut decl_map, &op2, &mut my_dag_node);
            //     }
            //     ExprRight::Cond(op_true, op_false) => {
            //
            //     }
            //     ExprRight::Empty() => {
            //
            //     }
            // }
        }


    }
}

pub fn trans_snippets<'a> (my_snippets : &'a Snippets<'a>) {
    // TODO : Deal with mutability of my_dag
    let my_dag : Dag<'a> = Dag { dag_vector : Vec::new()};
    for my_snippet in &my_snippets.snippet_vector {
        trans_my_snippet(&my_snippet, &my_dag);
    }
}
