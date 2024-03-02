use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    task::{Context, Poll, Wake, Waker},
};

pub struct Pollable<F> {
    future: F,
    waker: Arc<PollableWaker>,
}

impl<F: Future> Pollable<F> {
    pub fn new(future: F) -> Self {
        Self {
            future,
            waker: Arc::new(PollableWaker::new()),
        }
    }

    pub fn poll(&mut self) -> Poll<F::Output> {
        if !self.waker.awake.load(Ordering::Relaxed) {
            return Poll::Pending;
        }

        self.waker.awake.store(false, Ordering::Relaxed);

        let future = unsafe { Pin::new_unchecked(&mut self.future) };

        let waker = Waker::from(Arc::clone(&self.waker));

        let mut context = Context::from_waker(&waker);

        future.poll(&mut context)
    }
}

struct PollableWaker {
    awake: AtomicBool,
}

impl PollableWaker {
    fn new() -> Self {
        Self {
            awake: AtomicBool::new(true),
        }
    }
}

impl Wake for PollableWaker {
    fn wake(self: Arc<Self>) {
        self.awake.store(true, Ordering::Relaxed);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.awake.store(true, Ordering::Relaxed);
    }
}

pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    let block_on_waker = Arc::new(BlockOnWaker::new());

    let waker = Waker::from(Arc::clone(&block_on_waker));

    let mut context = Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Pending => {
                let mut awake = block_on_waker.awake.lock().unwrap();

                while !*awake {
                    awake = block_on_waker.cond.wait(awake).unwrap();
                }
            }
            Poll::Ready(output) => return output,
        }
    }
}

struct BlockOnWaker {
    awake: Mutex<bool>,
    cond: Condvar,
}

impl BlockOnWaker {
    fn new() -> Self {
        Self {
            awake: Mutex::new(false),
            cond: Condvar::new(),
        }
    }
}

impl Wake for BlockOnWaker {
    fn wake(self: Arc<Self>) {
        self.cond.notify_one()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.cond.notify_one()
    }
}
