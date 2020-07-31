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
}
