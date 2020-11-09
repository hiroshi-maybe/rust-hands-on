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

    // Ch 18-2 Refutability: Whether a Pattern Might Fail to Match

    {
        let xs = vec![Some(1), None, Some(2)];

        for /*let .Some(x)*/ x in xs {
            println!("{:?}", x);
        }
    }

    // Ch 18-3 Pattern Syntax

    {
        // Matching literals

        let x = 1;
        match x {
            1 => println!("one"),
            2 => println!("two"),
            _ => println!("anything else"),
        }
    }

    {
        // Matching Named Variables

        let x = Some(5);
        let y = 10;

        match x {
            Some(50) => println!("Got 50"),
            Some(y) => println!("Matched, y = {:?}", y),
            _ => println!("Default case, x = {:?}", x),
        }

        // at the end: x = Some(5), y = 10
        println!("at the end: x = {:?}, y = {:?}", x, y);
    }

    {
        // Multiple Patterns

        fn pr(x: i32) {
            match x {
                1 | 2 => println!("{}, one or two", x),
                3..=5 => println!("{}, three through five", x),
                // `exclusive range pattern syntax is experimental`
                //6..7 => println!("six"),
                _ => println!("{}, anything else", x),
            }
        }

        for i in 1..7 {
            pr(i);
        }
    }

    {
        // Destructuring to Break Apart Values

        struct Point { x: i32, y: i32, }
        let p = Point { x: 0, y: 7};
        //let Point { x: x, y: y } = p;
        let Point { x, y } = p;
        assert_eq!(x, 0);
        assert_eq!(y, 7);

        match p {
            Point { x, y: 0 } => println!("On the x axis at {}", x),
            Point { x: 0, y } => println!("On the y axis at {}", y),
            Point { x, y } => println!("On neither axis: ({}, {})", x, y),
        }

        #[derive(Debug)]
        enum Color {
            Rgb(i32, i32, i32),
            Hsv(i32, i32, i32),
        }

        enum Message {
            Quit, Move { x: i32, y: i32 }, Write(String), ChangeColor(Color),
        }

        let m = Message::ChangeColor(Color::Rgb(0, 160, 255));

        fn pr(m: Message) {
            match m {
                Message::Quit => println!("Quit variant"),
                Message::Move { x, y } => {
                    println!("Move in the x direction {} and y direction {}", x, y);
                },
                Message::Write(text) => println!("Text message: {}", text),
                Message::ChangeColor(color) => println!("Change the color to {:?}", color),
            }
        }

        pr(m);

        let mm = Message::ChangeColor(Color::Hsv(0, 1, 2));
        match mm {
            Message::ChangeColor(Color::Rgb(r, g, b)) => println!("To red {}, green {}, blue {}", r, g, b),
            Message::ChangeColor(Color::Hsv(h, s, v)) => println!("To hue {}, saturation {}, value {}", h, s, v),
            _ => (),
        }

        let ((_, _), Point { x: _x, y: _y }) = ((3, 10), Point { x: 3, y: -10 });
    }

    {
        // Ignoring Values in a Pattern

        let mut setting_value = Some(5);
        let new_setting_value = Some(10);

        match (setting_value, new_setting_value) {
            (Some(_), Some(_)) => {
                println!("Can' overwrite and existing value");
            },
            _ => {
                setting_value = new_setting_value;
            }
        }

        println!("setting is {:?}", setting_value);

        let s = Some(String::from("Hello!"));

        if let Some(_) = s {
        // ownership error
        // if let Some(_s) = s {
            println!("found a string");
        }
        println!("{:?}", s);

        struct Point { x: i32, y: i32, z: i32 }
        let o = Point { x: 0, y: 0, z: 0 };
        match o {
            Point { x, .. } => println!("x is {}", x),
            // `..` works only for the last part
            // Point { .., y, .. } => println!("x is {}", y),
        }

        let ns = (1, 2, 3, 4, 5);
        match ns {
            (2, ..) => println!("{:?}: prefix two", ns),
            (first, .., last) => println!("{}, .. ,{}", first, last),
            _ => println!("anything else"),
        }
    }

    {
        // Extra Conditionals with Match Guards
        let n = Some(4);
        match n {
            Some(x) if x < 5 => println!("less than five: {}", x),
            Some(x) => println!("more than or equal to five: {}", x),
            None => (),
        }

        let x = 4;
        let y = false;

        match x {
            4 | 5 | 6 if y => println!("yes"),
            _ => println!("no"),
        }
    }

    {
        // @ Bindings
        enum Message {
            Hello { id: i32 },
        }
        let m = Message::Hello { id: 5 };
        match m {
            Message::Hello { id: id @ 3..=7} => {
            // above is shorter with @ binding
            //Message::Hello { id } if 3<=id && id<=7 => {
                println!("found id {}", id);
            },
            Message::Hello { id: 10..=12 } => println!("found in another range"),
            Message::Hello { id } => println!("Some other id: {}", id),
        }
    }
}
