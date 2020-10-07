#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use crate::List::{Cons, Nil};

fn main() {
    {
        // Using a Box<T> to Store Data on the Heap
        let b = Box::new(5);
        println!("b = {}", b);
    }

    {
        // Enabling Recursive Types with Boxes
        let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
        println!("{:?}", list);
    }
}
