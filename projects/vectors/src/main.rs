fn main() {
    {
        {
            let v: Vec<i32> = Vec::new();
            println!("{:?}", v);
        }

        {
            let v = vec![1,2,3];
            println!("{:?}", v);
        }

        {
            let mut v = Vec::new();
            for i in 0..5 {
                v.push(i);
            }
            println!("{:?}", v);
        }

        {
            let v = vec![1,2,3,4,5];
            let third: &i32 = &v[2];
            println!("The third element is {}", third);

            if let Some(a) = v.get(2) {
                println!("The third element is {}", a);
            }
            println!("The 100-th element is {:?}", v.get(99));
        }

        {
            let mut v = vec![1,2,3,4,5];
            let first = &v[0];
            // Error is thrown because immutable borrow `first` is still alive
            // v.push(6);
            println!("The first elemetn is: {}", first);
        }

        {
            let v = vec![1,2,3];
            for a in &v {
                println!("{}", a);
            }

            let mut v = vec![1,2,3];
            for i in &mut v {
                *i += 100;
            }
            println!("{:?}", v);
        }
    }
}
