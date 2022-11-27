use core::time;
use std::sync::{Arc, Mutex};
use tokio::{join, sync::Mutex as TokioMutex};

const NUM_TASKS: usize = 8;
const NUM_LOOP: usize = 100_000;

async fn std_lock_only_task() -> Result<(), tokio::task::JoinError> {
    let val = Arc::new(Mutex::new(0));
    let mut ts_std_lock = vec![];

    for _ in 0..NUM_TASKS {
        // locking with std Mutex
        let n = val.clone();
        let t = tokio::spawn(async move {
            for _ in 0..NUM_LOOP {
                let mut n = n.lock().unwrap();
                *n += 1;
            }
        });
        ts_std_lock.push(t);
    }

    for t in ts_std_lock {
        t.await?;
    }

    println!(
        "[std] COUNT = {} (expected = {})",
        *val.lock().unwrap(),
        NUM_LOOP * NUM_TASKS
    );

    Ok(())
}

async fn tokio_lock_and_sleep_task() -> Result<(), tokio::task::JoinError> {
    let mut ts_tokio_lock = vec![];

    let tval = Arc::new(TokioMutex::new(0));
    ts_tokio_lock.push(tokio::spawn(lock_sleep(tval.clone())));

    for _ in 0..NUM_TASKS {
        // locking with tokio Mutex
        let n = tval.clone();
        ts_tokio_lock.push(tokio::spawn(lock_only(n)));
    }

    for t in ts_tokio_lock {
        t.await?;
    }

    println!(
        "[tokio] COUNT = {} (expected = {})",
        *tval.lock().await,
        NUM_TASKS + 1
    );

    Ok(())
}

async fn lock_only(v: Arc<TokioMutex<u64>>) {
    let mut n = v.lock().await;
    *n += 1;
}

async fn lock_sleep(v: Arc<TokioMutex<u64>>) {
    let mut n = v.lock().await;
    let ten_secs = time::Duration::from_secs(3);
    tokio::time::sleep(ten_secs).await;
    *n += 1;
}

#[tokio::main]
async fn main() -> Result<(), tokio::task::JoinError> {
    let t1 = std_lock_only_task();
    let t2 = tokio_lock_and_sleep_task();

    let res = join!(t1, t2);
    res.0.or(res.1)
}
