fn main() {
    {

        // Error due to missing type constraint to `T`
        /*
        fn largest<T>(list: &[T]) -> &T {
            let mut largest = &list[0];
            for item in list {
                if item > largest { largest = item; }
            }
            largest
        }*/

        fn largest_i32(list: &[i32]) -> &i32 {
            let mut largest = &list[0];
            for item in list {
                if item > largest { largest = item; }
            }
            largest
        }

        let nums = vec![1,2,3,4,5];
        let res = largest_i32(&nums);
        println!("The largest number is {}", res);
    }

    {
        #[derive(Debug)]
        struct Point<T, U> {
            x: T, y: U,
        }

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 1, y: 2.0 };
        println!("{:?}, {:?}", a, b);
    }
}
