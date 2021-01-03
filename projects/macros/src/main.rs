use macros::myvec;

fn main() {
    {
        // Declarative Macros with macro_rules! for General Metaprogramming

        let v = myvec![1,2,3];
        println!("{:?}", v);
    }
}
