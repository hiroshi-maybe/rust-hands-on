fn main() {
    #[derive(Debug)]
    enum IpAddrKind {
        V4,
        V6,
    }

    {
        let four = IpAddrKind::V4;
        let six = IpAddrKind::V6;

        println!("IP {:?} and IP {:?}", four, six);
    }

    #[derive(Debug)]
    struct IpAddr {
        kind: IpAddrKind,
        address: String,
    }

    {
        let home = IpAddr {
            kind: IpAddrKind::V4,
            address: String::from("127.0.0.1"),
        };
        let loopback = IpAddr {
            kind: IpAddrKind::V6,
            address: String::from("::1"),
        };

        println!("home: {:?}, loopback: {:?}", home, loopback);
    }

    {
        #[derive(Debug)]
        enum IpAddr {
            V4(String),
            V6(String),
        }

        let home = IpAddr::V4(String::from("127.0.0.1"));
        let loopback = IpAddr::V6(String::from("::1"));
        println!("home: {:?}, loopback: {:?}", home, loopback);
    }

    {
        #[derive(Debug)]
        enum IpAddr {
            V4(u8, u8, u8, u8),
            V6(String),
        }

        let home = IpAddr::V4(127, 0, 0, 1);
        let loopback = IpAddr::V6(String::from("::1"));
        println!("home: {:?}, loopback: {:?}", home, loopback);
    }

    #[derive(Debug)]
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(i32, i32, i32),
    }

    {
        let q = Message::Quit;
        let m = Message::Move { x: 1, y: 2 };
        let w = Message::Write(String::from("abc"));
        let c = Message::ChangeColor(1, 2, 3);

        println!("{:?}, {:?}, {:?}, {:?}", q, m, w, c);
    }

    // The Option Enum and Its Advantages Over Null Values

    {
        let n = Some(5);
        let s = Some("a string");
        let none: Option<i32> = None;

        println!("{:?}, {:?}, {:?}", n, s, none);
    }
}
