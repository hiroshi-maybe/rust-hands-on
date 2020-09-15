use traits::{Tweet, Summary, MyDiary, NewsArticle, Display};

fn main() {
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

    let news = NewsArticle {
        headline: String::from("Breaking news!"),
        location: String::from("San Jose"),
        author: String::from("Mike"),
        content: String::from("abc"),
    };
    println!("News: {}", news.summarize());


    {
        // Traits as Parameters
        fn notify(item: &impl Summary) {
            println!("Breaking news! {}", item.summarize());
        }

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

        fn returns_summarizable() -> impl Summary {
            MyDiary {}
        }
        println!("Returns summarizabe: {:?}", returns_summarizable().summarize());
    }


    {
        // Back to largest()

        fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
            let mut res = list[0];
            for &item in list {
                if item < res {
                    res = item;
                }
            }

            res
        }

        fn largest_borrow<T: PartialOrd>(list: &[T]) -> &T {
            let mut res = &list[0];
            for item in list {
                if item < res {
                    res = item;
                }
            }

            res
        }

        let nums = vec![1,2,3,4,5];
        let res = largest(&nums);
        println!("The largest number is {}", res);

        let chars = vec!['a','b','c','d'];
        let res = largest_borrow(&chars);
        println!("The largest char is {}", res);
    }
}
