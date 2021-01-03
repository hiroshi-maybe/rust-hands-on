// Read https://danielkeep.github.io/tlborm/book/index.html for more details

// Declarative Macros with macro_rules! for General Metaprogramming

#[macro_export]
macro_rules! myvec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

// Procedural Macros for Generating Code from Attributes

