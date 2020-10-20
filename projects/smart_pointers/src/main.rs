use std::ops::Deref;

// Ch 15-1 box

#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}
use crate::List::{Cons, Nil};

// Ch 15-2 deref

struct MyBox<T>(T);
impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

// Ch 15-3 drop

#[derive(Debug)]
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    fn drop(&mut self) {
        println!("Dropping CustomSmartPointer with data `{}`!", self.data);
    }
}

// Ch 15-4 reference counting

use std::rc::Rc;

#[derive(Debug)]
enum RcList {
    Cons(i32, Rc<RcList>),
    Nil,
}

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

    {
        let mut x = 5;
        let y = &x;

        println!("x: {}, y: {}, *y: {}", x, y, *y);
        assert_eq!(5, x);
        assert_eq!(5, *y); // syntax sugar of *(y.deref())
        assert_eq!(5, *(y.deref()));
        assert_eq!(x, *y);
        // Throws a compile time error: no implementation for `{integer} == &{integer}`
        // assert_eq!(5, y);
        // assert_eq!(x, y);

        let bx = Box::new(x);
        println!("x: {}, bx: {}, *bx: {}", x, bx, *bx);
        assert_eq!(5, *bx);
        assert_eq!(5, *(bx.deref()));
        assert_eq!(x, *bx);
        // Error: no implementation for `{integer} == std::boxed::Box<{integer}>`
        // assert_eq!(5, bx);
        // assert_eq!(x, bx);

        let z = &x;
        assert_eq!(*z, *y);
        assert_eq!(z, y);
        let zz = &z;
        assert_eq!(**zz, *z);
        assert_eq!(*zz, z);
        assert_eq!(*zz.deref(), 5);
        assert_eq!(*zz.deref().deref(), 5);

        x = 10;
        assert_eq!(10, x);
        //assert_eq!(5, *y);
    }

    {
        // Defining Our Own Smart Pointer
        let x = 5;
        let y = MyBox::new(x);

        assert_eq!(5, x);
        assert_eq!(5, *y);
    }

    {
        // Implicit Deref Coercions with Functions and Methods

        fn hello(name: &str) {
            println!("Hello, {}!", name);
        }

        let m = MyBox::new(String::from("Rust"));
        hello(&m);
        hello(m.deref());
        hello(&(*m)[..]);
    }

    {
        let c = CustomSmartPointer { data: String::from("my stuff") };
        let d = CustomSmartPointer { data: String::from("other stuff") };
        println!("CustomSmartPointers created: {:?}, {:?}", c, d);
        // Error: `explicit destructor calls not allowed`
        // c.drop();
        drop(c);
        // Error: `value used here after move`
        // drop(c);
        println!("CustomSmartPointers dropped before the end of main.");
    }

    {
        // Using Rc<T> to Share Data

        let a = Cons(5, Box::new(Cons(10, Box::new(Nil))));
        let b = Cons(3, Box::new(a));
        // Error: `value used here after move`
        // let c = Cons(4, Box::new(a));

        let a = Rc::new(RcList::Cons(5, Rc::new(RcList::Cons(10, Rc::new(RcList::Nil)))));
        println!("count after creating a = {}", Rc::strong_count(&a));
        let b = RcList::Cons(3, Rc::clone(&a));
        println!("count after creating a = {}", Rc::strong_count(&a));
        {
            let c = RcList::Cons(4, Rc::clone(&a));
            println!("count after creating c = {}", Rc::strong_count(&a));
        }
        println!("count after c is dropped = {}", Rc::strong_count(&a));
    }
}
