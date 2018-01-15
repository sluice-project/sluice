// Make sure the grammar is LL(1)
enum Program {
  Program {
    Vec<Snippets>,
    Datapath
  }
}

enum Snippet {
  Snippet {
    snippet_id : String,
    argument_list : Vec<String>,
    initializers : Vec<Initializer>,
    body : Vec<Statement>
  }
}

enum Datapath {
  Connections(Vec<Connection>) 
}

enum Connection {
  Connection {
    from_id : String,
    to_id   : String
  }
}

enum Initializer {


}
