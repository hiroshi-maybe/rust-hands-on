use std::fmt;
use std::ops::Add;

pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
}

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Add for Point {
    type Output = Point;
    // No need to specify `type Rhs = Point` because `Add` trait has
    // a default type parameter `trait Add<Rhs=Self> {`

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

fn main() {
    assert_eq!(
        Point { x: 1, y: 0 } + Point { x: 2, y: 3 },
        Point { x: 3, y: 3 }
    );

    {
        // Default Generic Type Parameters and Operator Overloading

        #[derive(Debug, PartialEq)]
        struct Millimeters(u32);
        struct Meters(u32);

        // Multiple implementations for type parameter
        impl Add<Meters> for Millimeters {
            type Output = Millimeters;

            fn add(self, other: Meters) -> Millimeters {
                Millimeters(self.0 + (other.0 * 1000))
            }
        }
        impl Add<u32> for Millimeters {
            type Output = Millimeters;

            fn add(self, other: u32) -> Millimeters {
                Millimeters(self.0 + other)
            }
        }
        // Leveraging default parameter `impl Add<Self> for Millimeters {`
        impl Add for Millimeters {
            type Output = Millimeters;

            fn add(self, other: Millimeters) -> Millimeters {
                Millimeters(self.0 + other.0)
            }
        }
        /*
        // Conflict with the definition above
        impl Add<Millimeters> for Millimeters {
            type Output = u32;

            fn add(self, other: Millimeters) -> u32 {
                self.0 + other.0
            }
        }*/

        assert_eq!(Millimeters(1) + Meters(2), Millimeters(2001));
        assert_eq!(Millimeters(1) + 2, Millimeters(3));
        assert_eq!(Millimeters(1) + Millimeters(2), Millimeters(3));
        //assert_eq!(Millimeters(1) + Millimeters(2), 3);
    }

    {
        // Fully Qualified Syntax for Disambiguation: Calling Methods with the Same Name

        trait Pilot {
            fn fly(&self);
        }

        trait Wizard {
            fn fly(&self);
        }

        struct Human;

        impl Pilot for Human {
            fn fly(&self) {
                println!("This is your captain speaking.");
            }
        }

        impl Wizard for Human {
            fn fly(&self) {
                println!("Up!");
            }
        }

        impl Human {
            fn fly(&self) {
                println!("*waving arms furiously*");
            }
        }

        let person = Human;
        person.fly();
        Human::fly(&person);
        Wizard::fly(&person);
        Pilot::fly(&person);

        trait Animal {
            fn baby_name() -> String;
        }

        struct Dog;

        impl Dog {
            fn baby_name() -> String {
                String::from("Spot")
            }
        }

        impl Animal for Dog {
            fn baby_name() -> String {
                String::from("puppy")
            }
        }

        println!("A baby dog is called a {}", Dog::baby_name());
        println!("A baby dog is called a {}", <Dog as Animal>::baby_name());
    }

    {
        // Using Supertraits to Require One Trait’s Functionality Within Another Trait

        trait OutlinePrint: fmt::Display {
            fn outline_print(&self) {
                let output = self.to_string();
                let len = output.len();

                println!("{}", "*".repeat(len + 4));
                println!("*{}*", " ".repeat(len + 2));
                println!("* {} *", output);
                println!("*{}*", " ".repeat(len + 2));
                println!("{}", "*".repeat(len + 4));
            }
        }

        impl fmt::Display for Point {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "({}, {})", self.x, self.y)
            }
        }
        impl OutlinePrint for Point {}

        let p = Point { x: 1, y: 2 };
        println!("{:?}", p.outline_print());
    }

    {
        // Using the Newtype Pattern to Implement External Traits on External Types

        struct Wrapper(Vec<String>);

        impl fmt::Display for Wrapper {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "[{}]", self.0.join(", "))
            }
        }

        let w = Wrapper(vec![String::from("hello"), String::from("world")]);
        println!("w = {}", w);
    }
}
