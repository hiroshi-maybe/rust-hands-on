use std::time;

#[tokio::main]
async fn main() {
    tokio::join!(async move {
        let ten_secs = time::Duration::from_secs(10);
        println!("sleep started");
        // sleep that blocks the current thread
        // thread::sleep(ten_secs);
        tokio::time::sleep(ten_secs).await;
        println!("sleep ended");
    });
}
