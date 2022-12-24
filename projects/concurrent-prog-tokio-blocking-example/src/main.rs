fn do_block(n: u64) -> u64 {
    let ten_secs = std::time::Duration::from_secs(10);
    std::thread::sleep(ten_secs);
    println!("wake up - do_block {}", n);
    n
}

async fn do_print() {
    let sec = std::time::Duration::from_secs(1);
    for n in 0..20 {
        tokio::time::sleep(sec).await;
        println!("wake up - do_print {}", n);
    }
}

#[tokio::main]
pub async fn main() {
    // do_block() runs concurrently by blocking its OWN thread.
    let mut v = vec![];
    for n in 0..32 {
        let t = tokio::task::spawn_blocking(move || do_block(n));
        v.push(t);
    }

    // do_print() keeps running concurrently with do_block()
    let p = tokio::spawn(do_print());

    for t in v {
        let n = t.await.unwrap();
        println!("finished: {}", n);
    }
    p.await.unwrap();
}
