fn main() {
    {
        // Repeating with `loop`
        let mut n = 0;
        loop {
            n += 1;
            println!("again!: {}", n);
            if n >= 10 {
                break;
            }
        }
    }

    {
        // Returning Values from Loops
        let mut counter = 0;
        let result = loop {
            counter += 1;

            if counter == 10 {
                break counter * 2;
            }
        };

        println!("The result is {}", result);
    }

    {
        // Conditional Loops with `while`
        let mut number = 3;
        while number != 0 {
            println!("{}!", number);
            number -= 1;
        }

        println!("LIFTOFF!!!");
    }

    {
        // `for` loop through a collection

        let a = [1, 2, 3, 4, 5];
        for element in a.iter() {
            println!("the value is: {}", element);
        }

        for number in (1..4).rev() {
            println!("{}!", number);
        }
        println!("LIFTOFF!!!");
    }

    {
        // Fibonacci

        let n = 6;
        let f = {
            let mut counter = 2;
            let mut a = 1;
            let mut b = 1;

            loop {
                counter += 1;
                let c = a + b;
                a = b;
                b = c;

                if counter == n {
                    break b;
                }
            }
        };

        println!("{}-th fibonacci is: {}", n, f);
    }
}
