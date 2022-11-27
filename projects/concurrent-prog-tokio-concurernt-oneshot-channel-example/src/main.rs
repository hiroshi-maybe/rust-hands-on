use tokio::sync::oneshot;

async fn set_val_later(tx: oneshot::Sender<i32>) {
    let three_secs = std::time::Duration::from_secs(3);
    tokio::time::sleep(three_secs).await;
    if let Err(_) = tx.send(100) {
        println!("failed to send");
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    println!("future spawned");
    tokio::spawn(set_val_later(tx));

    match rx.await {
        Ok(n) => {
            println!("n = {}", n);
        }
        Err(e) => {
            println!("failed to receive: {}", e);
            return;
        }
    }
}
