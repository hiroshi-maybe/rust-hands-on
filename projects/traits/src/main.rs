use traits::{Tweet, Summary, MyDiary, NewsArticle, Display};

fn main() {
    {
        // Implementing a Trait on a Type

        let tweet = Tweet {
            username: String::from("horse_ebooks"),
            content: String::from("of course, as you probably know, people"),
            reply: false,
            retweet: false,
        };

        println!("1 new tweet: {}", tweet.summarize());
        let diary = MyDiary {};
        println!("My diary: {}", diary.summarize());

        let a = NewsArticle {
            headline: String::from("Breaking news!"),
            location: String::from("San Jose"),
            author: String::from("Mike"),
            content: String::from("abc"),
        };
        println!("News: {}", a.summarize());
    }

    {
        // Traits as Parameters
        fn notify(item: &impl Summary) {
            println!("Breaking news! {}", item.summarize());
        }

        let tweet = Tweet {
            username: String::from("horse_ebooks"),
            content: String::from("of course, as you probably know, people"),
            reply: false,
            retweet: false,
        };

        notify(&tweet);

        fn notify2(item1: &impl Summary, item2: &impl Summary) {
            println!("{}, {}", item1.summarize(), item2.summarize());
        }

        // Different concrete type works
        let diary = MyDiary {};
        notify2(&tweet, &diary);

        fn notify3<T: Summary>(item1: &T, item2: &T) {
            println!("{}, {}", item1.summarize(), item2.summarize());
        }

        // Parameters should have the same concrete types
        notify3(&tweet, &tweet);
        // This doesn't work
        // notify3(&tweet, &diary);

        fn notify_to_display(item: &(impl Summary + Display)) {
            println!("{}", item.summarize());
        }
        notify_to_display(&tweet);

        fn notify_to_display2<T>(item: &T)
            where T: Summary + Display
        {
            println!("{}", item.summarize());
        }
        notify_to_display2(&tweet);
    }

    {
        // Returning Types that Implement Traits

    }
}
