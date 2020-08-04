fn main() {
    {
        // if expression
        let number = 7;

        if number < 5 {
            println!("condition was true");
        } else {
            println!("condition was false");
        }
    }

    {
        // Using if in a let Statement

        let condition = true;
        let n = if condition { 5 } else { 6 };

        println!("The value of number is: {}", n);

        let a = if condition {
            println!("condition was true")
        } else {
            println!("condition was false")
        };

        // "()" is printed
        println!("The value of void is: {:?}", a);

        // 5 is error
        //let number: &str = if condition { 5 } else { "six" };
    }
}
