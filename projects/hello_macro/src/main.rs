use hello_macro::myvec;
use hello_macro::HelloMacro;
use hello_macro_attribute::show_streams;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct Pancakes;

#[show_streams("attr message")]
fn hello_attr_like_macro() {
    println!("`hello_attr_like_macro` function called!");
}

fn main() {
    {
        // Declarative Macros with macro_rules! for General Metaprogramming
        let v = myvec![1, 2, 3];
        println!("{:?}", v);
    }

    {
        // How to Write a Custom derive Macro
        Pancakes::hello_macro();
    }

    {
        // Attribute-like macros
        hello_attr_like_macro();
    }
}
