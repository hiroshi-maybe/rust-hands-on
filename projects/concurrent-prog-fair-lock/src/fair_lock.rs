use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{fence, AtomicBool, AtomicUsize, Ordering},
};

pub const NUM_LOCK: usize = 8;
const MASK: usize = NUM_LOCK - 1;

pub struct FairLock<T> {
    waiting: Vec<AtomicBool>,
    lock: AtomicBool,
    turn: AtomicUsize,
    data: UnsafeCell<T>,
}

pub struct FairLockGuard<'a, T> {
    fair_lock: &'a FairLock<T>,
    idx: usize,
}

impl<T> FairLock<T> {
    pub fn new(v: T) -> Self {
        let mut vec = Vec::new();
        for _ in 0..NUM_LOCK {
            vec.push(AtomicBool::new(false));
        }

        FairLock {
            waiting: vec,
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(v),
            turn: AtomicUsize::new(0),
        }
    }

    pub fn lock(&self, idx: usize) -> FairLockGuard<T> {
        assert!(idx < NUM_LOCK);

        self.waiting[idx].store(true, Ordering::Relaxed);
        loop {
            // Another thread let the current thread get the turn and take a lock.
            if !self.waiting[idx].load(Ordering::Relaxed) {
                break;
            }

            // It's not a turn of the current thread, but it attempts to take a lock. Beat other threads in the contention.
            if !self.lock.load(Ordering::Relaxed) {
                if let Ok(_) = self.lock.compare_exchange_weak(
                    false,
                    true,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    break;
                }
            }
        }
        fence(Ordering::Acquire);

        FairLockGuard {
            fair_lock: self,
            idx: idx,
        }
    }
}

impl<'a, T> Drop for FairLockGuard<'a, T> {
    fn drop(&mut self) {
        let fl = self.fair_lock;

        fl.waiting[self.idx].store(false, Ordering::Relaxed);

        let turn = fl.turn.load(Ordering::Relaxed);
        let next = if turn == self.idx {
            (turn + 1) & MASK
        } else {
            turn
        };

        if fl.waiting[next].load(Ordering::Relaxed) {
            // A thread for the next turn is waiting. Give the lock!
            fl.turn.store(next, Ordering::Relaxed);
            fl.waiting[next].store(false, Ordering::Release);
        } else {
            // The thread for the next turn is not waiting. Move the turn and make a contention happen.
            fl.turn.store((next + 1) & MASK, Ordering::Relaxed);
            fl.lock.store(false, Ordering::Release);
        }
    }
}

unsafe impl<T> Sync for FairLock<T> {}
unsafe impl<T> Send for FairLock<T> {}

impl<'a, T> Deref for FairLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.fair_lock.data.get() }
    }
}

impl<'a, T> DerefMut for FairLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.fair_lock.data.get() }
    }
}
