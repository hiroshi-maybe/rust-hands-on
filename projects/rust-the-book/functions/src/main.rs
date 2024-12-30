fn main() {
    println!("Hello, world!");

    another_function(5);

    {
        // Block is expression
        let y = {
            let x = 3;
            x + 1
        };

        println!("The value from a block expression: {}", y);
    }

    {
        // Function returns a value
        let x = five();
        println!("The value from a function: {}", x);

        let parity1 = parity(1);
        let parity2 = parity(2);
        println!("Function demo of early return: {}, {}", parity1, parity2);

        let fact5 = fact(5);
        println!("5! by recursive function: {}", fact5);
    }
}

fn another_function(x: i32) {
    println!("The value passed to another function is : {}", x);
}

fn five() -> i32 {
    5
}

fn parity(n: i32) -> i32 {
    if n % 2 != 0 {
        return 1;
    }

    0
}

fn fact(n: u32) -> u32 {
    if n == 1 {
        1
    } else {
        n * fact(n - 1)
    }
}
