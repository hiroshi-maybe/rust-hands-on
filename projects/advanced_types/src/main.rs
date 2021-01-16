use std::fmt;

fn main() {
    {
        // Creating Type Synonyms with Type Aliases
        type Kilometers = i32;
        let x: i32 = 1;
        let y: Kilometers = 2;
        assert_eq!(x + y, 3);

        type Result<T> = std::result::Result<T, std::io::Error>;

        pub trait Write {
            fn write(&mut self, buf: &[u8]) -> Result<usize>;
            fn flush(&mut self) -> Result<()>;

            fn write_all(&mut self, buf: &[u8]) -> Result<()>;
            fn write_fmt(&mut self, fmt: fmt::Arguments) -> Result<()>;
        }
    }

    {
        // The Never Type that Never Returns

        fn bar() -> ! {
            panic!("never is returned");
        }
    }

    {
        // Dynamically Sized Types and the Sized Trait

        // Size is known at compile time
        fn sized_generic<T /* :Sized */>(t: T) -> T {
            t
        }

        // Size may or may not be known at compile time
        fn unsized_generic<T: ?Sized>(t: &T) -> &T {
            t
        }
    }
}
