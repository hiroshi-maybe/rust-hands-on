use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;

use futures::future::BoxFuture;
use futures::task::waker_ref;
use futures::task::ArcWake;
use futures::FutureExt;

struct Hello {
    state: StateHello,
}

enum StateHello {
    Hello,
    World,
    End,
}

impl Hello {
    fn new() -> Self {
        Hello {
            state: StateHello::Hello,
        }
    }
}

impl Future for Hello {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        match (*self).state {
            StateHello::Hello => {
                print!("Hello, ");
                (*self).state = StateHello::World;
                Poll::Pending
            }
            StateHello::World => {
                println!("World");
                (*self).state = StateHello::End;
                Poll::Pending
            }
            StateHello::End => Poll::Ready(()),
        }
    }
}

struct Task {
    hello: Mutex<BoxFuture<'static, ()>>,
}

impl Task {
    fn new() -> Self {
        let hello = Hello::new();
        Task {
            hello: Mutex::new(hello.boxed()),
        }
    }
}

impl ArcWake for Task {
    fn wake_by_ref(_arc_self: &Arc<Self>) {}
}

fn main() {
    let task = Arc::new(Task::new());
    let waker = waker_ref(&task);
    let mut ctx = Context::from_waker(&waker);
    let mut hello = task.hello.lock().unwrap();

    for _ in 0..3 {
        let _ = hello.as_mut().poll(&mut ctx);
    }
}
