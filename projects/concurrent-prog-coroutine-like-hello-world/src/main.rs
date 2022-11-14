use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
    Future, FutureExt,
};
use std::{
    pin::Pin,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    task::{Context, Poll},
};

struct Task {
    future: Mutex<BoxFuture<'static, ()>>,
    sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        println!("[Task] waker (=Task itself) awoken");
        let self0 = arc_self.clone();
        println!("[Task] sent by waker (=Task itself) to the queue");
        arc_self.sender.send(self0).unwrap();
    }
}

struct Executor {
    sender: SyncSender<Arc<Task>>,
    receiver: Receiver<Arc<Task>>,
}

impl Executor {
    fn new() -> Self {
        let (sender, receiver) = sync_channel(1024);
        println!("[Executor] instantiated");
        Executor {
            sender: sender.clone(),
            receiver,
        }
    }

    fn get_spawner(&self) -> Spawner {
        println!("[Executor] created spawner");
        Spawner {
            sender: self.sender.clone(),
        }
    }

    fn run(&self) {
        println!("[Executor] started running");
        let mut cnt = 1;
        while let Ok(task) = self.receiver.recv() {
            println!("[Executor][{}] received task", cnt);
            let mut future = task.future.lock().unwrap();
            let waker = waker_ref(&task);
            println!("[Executor][{}] obtained waker from task", cnt);
            let mut ctx = Context::from_waker(&waker);
            println!("[Executor][{}] started polling", cnt);
            let _ = future.as_mut().poll(&mut ctx);
            println!("[Executor][{}] finished polling", cnt);
            cnt += 1;
        }
        println!("[Executor] finished running");
    }
}

struct Spawner {
    sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(future),
            sender: self.sender.clone(),
        });

        println!("[Spawner] sent task to the queue");
        self.sender.send(task).unwrap();
    }
}

fn main() {
    let executor = Executor::new();
    executor.get_spawner().spawn(Hello::new());
    executor.run();
}

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
        println!("[Hello] instantiated");
        Hello {
            state: StateHello::Hello,
        }
    }
}

impl Future for Hello {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        println!("[Hello] polled");
        match (*self).state {
            StateHello::Hello => {
                println!("** Hello");
                (*self).state = StateHello::World;
                println!("[Hello] awake waker");
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            StateHello::World => {
                println!("** World");
                (*self).state = StateHello::End;
                println!("[Hello] awake waker");
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            StateHello::End => Poll::Ready(()),
        }
    }
}
