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

    {
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
    }
}
