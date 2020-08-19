fn main() {
    let w = 30;
    let h = 50;
    {
        fn area(w: u32, h: u32) -> u64 { (w as u64) * (h as u64) }
        println!("area for {} x {}: {}", w, h, area(w, h));
    }

    {
        fn area(d: (u32, u32)) -> u64 { (d.0 as u64) * (d.1 as u64) }
        println!("area for {} x {}: {}", w, h, area((w, h)));
    }

    #[derive(Debug)]
    struct Rectangle {
        width: u32,
        height: u32,
    }

    impl Rectangle {
        fn area(&self) -> u64 {
            (self.width as u64) * (self.height as u64)
        }

        fn can_hold(&self, other: &Rectangle) -> bool {
            self.width >= other.width && self.height >= other.height
        }
    }

    {
        // Area
        let rect1 = Rectangle {
            width: w,
            height: h,
        };

        fn area(rectangle: &Rectangle) -> u64 {
            (rectangle.width as u64) * (rectangle.height as u64)
        }
        println!("area for {} x {}: {}", w, h, area(&rect1));

        println!("area for {:?}: {}", rect1, area(&rect1));
        println!("area for pretty {:#?}: {}", rect1, area(&rect1));
        println!("area calculated through a method: {}", rect1.area());
    }

    {
        // Hold
        let rect1 = Rectangle {
            width: 30,
            height: 50,
        };
        let rect2 = Rectangle {
            width: 10,
            height: 40,
        };
        let rect3 = Rectangle {
            width: 60,
            height: 45,
        };

        let rects = [rect1, rect2, rect3];
        for j in 0..rects.len() {
            for i in 0..rects.len() {
                if i != j {
                    println!("Can rect{} {:?} hold rect{} {:?}? {}",
                    i+1, rects[i], j+1, rects[j],
                    rects[i].can_hold(&rects[j]));
                }
            }
        }
    }

    {
        // Associated functions

        impl Rectangle {
           fn square(size: u32) -> Rectangle {
                Rectangle {
                    width: size,
                    height: size,
                }
            }
        }

        let rect1 = Rectangle {
            width: 30,
            height: 50,
        };

        let sq = Rectangle::square(20);
        println!("Rect {:?} can hold square {:?}? {}",
        rect1, sq, rect1.can_hold(&sq));
    }

    {
        // `impl` blocks are visible outside of the scope
        let x = Rectangle::square(10);
        println!("{:?}", x);
    }
}
