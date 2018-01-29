fn main() {
  let mut a = vec!["sds"; 300000];
  let mut b = vec![]; 

  // Implementation 1: takes about 50 seconds
  loop {
    if a.is_empty() { break; }
    let item = a.remove(0);
    b.push(item);
  }

  // Implementation 2: takes about 0.05 seconds
//  let mut a_iter = a.iter().peekable();
//  loop {
//    if a_iter.peek().is_none() { break; }
//    let item = a_iter.next();
//    b.push(item);
//  }
}
