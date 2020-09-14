use traits::{Tweet, Summary, MyDiary, NewsArticle};

fn main() {
    {
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
}
