fn main() {
    {
        // Ch 17-2 Using Trait Objects That Allow for Values of Different Types
        use oop::{Button, Screen, SelectBox};

        let screen = Screen {
            components: vec![
                Box::new(SelectBox {
                    width: 75,
                    height: 10,
                    options: vec![
                        String::from("Yes"),
                        String::from("Maybe"),
                        String::from("No"),
                    ],
                }),
                Box::new(Button {
                    width: 50,
                    height: 10,
                    label: String::from("OK"),
                }),
            ],
        };

        screen.run();
    }

    {
        // Ch 17-3 Implementing an Object-Oriented Design Pattern

        // State pattern
        use oop::Post;

        let mut post = Post::new();

        post.add_text("I ate a salad for lunch today");
        assert_eq!("", post.content());

        post.request_review();
        assert_eq!("", post.content());

        post.approve();
        assert_eq!("I ate a salad for lunch today", post.content());

        // Type safe encoding
        use oop::SafePost;

        let mut post = SafePost::new();
        post.add_text("I ate a salad for lunch today");
        let post = post.request_review();
        let post = post.approve();
        assert_eq!("I ate a salad for lunch today", post.content());
    }
}
