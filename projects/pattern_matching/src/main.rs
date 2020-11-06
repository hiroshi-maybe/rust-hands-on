fn main() {
    // Ch 18-1 All the Places Patterns Can Be Used

    {
        // Conditional if let Expressions

        let fav_color: Option<&str> = None;
        let is_tuesday = false;
        let age: Result<u8, _> = "34".parse();

        if let Some(color) = fav_color {
            println!("Using your favorite color, {}, as the background", color);
        } else if is_tuesday {
            println!("Tuesday is green day!");
        } else if let Ok(age) = age{
            if age > 30 {
                println!("Using purple as the background color");
            } else {
                println!("Using orange as the background color");
            }
        } else {
            println!("Using blue as the background color");
        }
    }

    {
        // while let Conditional Loops

        let mut stack = vec![1, 2, 3];
        while let Some(top) = stack.pop() {
            println!("{}", top);
        }
    }

    {
        // for Loops

        let v = vec!['a', 'b', 'c'];
        for (i, v) in v.iter().enumerate() {
            println!("{} is at index {}", v, i);
        }
    }

    {
        // let Statements

        let (x,y, ..) = (1,2,3,4);

        println!("let ({},{},..) = (1,2,3,4)", x, y);
    }

    {
        // Function Parameters

        fn print_coordinates(&(x, y): &(i32, i32)) {
            println!("Current location: ({}, {})", x, y);
        }

        print_coordinates(&(3,5));
    }
}
