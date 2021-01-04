use hello_macro::myvec;
use hello_macro::HelloMacro;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct Pancakes;

fn main() {
    {
        // Declarative Macros with macro_rules! for General Metaprogramming
        let v = myvec![1,2,3];
        println!("{:?}", v);
    }

    {
        // How to Write a Custom derive Macro
        Pancakes::hello_macro();
    }
}
