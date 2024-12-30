fn main() {
    {
        // Integer types
        let dec = 10_000_00;
        println!("decimal: {}", dec);

        let hex = 0xf;
        println!("hex: {}", hex);

        let octal = 0o10;
        println!("octal: {}", octal);

        let binary = 0b_1_1_1;
        println!("binary: {}", binary);

        let byte = b'A';
        println!("byte: {}", byte)
    }

    {
        // Floating-Point type
        let x = 2.0;
        let y: f32 = 3.0;
        let z = 2e5;
        println!("floating points: {}, {}, {}", x, y, z);

        let a = 1e-10;
        let b = 1.0 / 1000.0 * 1000.0;
        let c = 2.0 % 4.1;
        println!("floating points: {}, {}, {}", a, b, c);
    }

    {
        // Boolean type
        let x = true;
        let y = x ^ true;
        let z = x | y;
        println!("boolean types: {}, {}, {}", x, y, z);
    }

    {
        // Character type
        let a = 'a';
        let b: char = 'b';
        let c = '🍙';
        let d = '𠮷';
        println!("floating points: {}, {}, {}, {}", a, b, c, d);

        // https://emojipedia.org/emoji/
        let a = '🎌';
        //let b = '🤹🏿‍♀️';
        //let b = '🧟‍♀️';
        let b = '👋';
        //let c = '👋🏽';
        //let c = '☠️';
        let c = '㍻';
        //let d = '👨‍👨‍👧‍👧';
        let d = '';

        println!("floating points: {}, {}, {}, {}", a, b, c, d);
    }

    {
        // The tuple type
        let t: (i32, f64, u8) = (500, 6.4, 1);
        let (x, y, z) = t;
        println!("tuple: {:?}", t);
        println!("Destructured tuple elements: {}, {}, {}", x, y, z);
        println!("Tuple access with dot nottation: {}, {}, {}", t.0, t.1, t.2);
        // Tuple is immutable by default as well
        // t.0 = 1;

        let mut t: (i32, char) = (1, 'あ');
        t.1 = 'い';
        println!("Mutable tuple: {:?}", t);
    }

    {
        // The array type
        let a = [1, 2, 3, 4, 5];
        println!("Array: {:?}", a);
        // Array is immutable by default as well
        // a[1]=2;

        let mut a: [char; 4] = ['a', 'b', 'c', 'd'];
        // Index out of bounds access for an array is compile error in Rust
        // a[4] = 'e';
        // println!("Index out of bounds: {}", a[11]);
        a[3] = 'e';
        println!("Array: {:?}", a);

        let mut a = [1; 10];
        a[1] = 2;
        println!("Initialization with same values: {:?}", a);
    }
}
