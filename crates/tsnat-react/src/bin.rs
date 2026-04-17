use yoga::{Node};
fn main() {
    let mut parent = Node::new();
    let mut child = Node::new();
    parent.insert_child(&mut child, 0);
}
