struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

fn build_user(email: String, username: String) -> User {
    User {
        email,
        username,
        active: true,
        sign_in_count: 1,
    }
}

fn main() {
    {
        // Structs

        let mut user1 = User {
            email: String::from("a@example.com"),
            username: String::from("hiroshi maybe"),
            active: true,
            sign_in_count: 1,
        };
        user1.email = String::from("b@example.com");

        let e = String::from("c@example.com");
        let u = String::from("hiroshi probably");
        let user2 = build_user(e, u);

        let user3 = User {
            email: String::from("d@example.com"),
            username: String::from("h maybe"),
            ..user1
        };
    }

    {
        // Tuple structs

        struct Color(i32, i32, i32);
        struct Point(i32, i32, i32);

        let black = Color(0, 0, 0);
        let origin = Point(0, 0, 0);
    }

    {
        // Borrowed fields

        // Compiler throws errors without `lifetime`
        /*
        struct User {
            username: &str,
            email: &str,
            sign_in_count: u64,
            active: bool,
        }*/
    }
}
