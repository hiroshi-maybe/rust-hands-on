fn main() {
    {
        // In Function Definitions

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
        // In Struct Definitions

        #[derive(Debug)]
        struct Point<T, U> {
            x: T, y: U,
        }

        let a = Point { x: 1, y: 2 };
        let b = Point { x: 1, y: 2.0 };
        println!("{:?}, {:?}", a, b);
    }

    {
        // In Enum Definitions
        #[derive(Debug)]
        enum TreeNode<T> {
            Node(T),
            Branch(Box<TreeNode<T>>, Box<TreeNode<T>>)
        }

        let tree = TreeNode::Branch(
            Box::new(TreeNode::Node(1)),
            Box::new(TreeNode::Branch(
                Box::new(TreeNode::Branch(
                    Box::new(TreeNode::Node(2)),
                    Box::new(TreeNode::Node(3)),
                )),
                Box::new(TreeNode::Node(4)),
            ))
        );

        println!("{:?}", tree);
    }

    {
        // In Method Definitions

        #[derive(Debug)]
        struct Point<T> {
            x: T, y: T,
        }
        impl<T> Point<T> {
            fn x(&self) -> &T {
                &self.x
            }
        }

        impl Point<f32> {
            fn distance_from_origin(&self) -> f32 {
                (self.x().powi(2) + self.y.powi(2)).sqrt()
            }
        }

        let p = Point { x: 1.0, y: 2.0 };
        println!("{:?}", p.distance_from_origin());
    }
}
