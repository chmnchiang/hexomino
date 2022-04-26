use std::{
    cell::{Cell, RefCell, UnsafeCell},
    collections::VecDeque,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker}, fmt::{Display, Debug},
};

use futures::FutureExt as _;
use log::error;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

pub type Shared<T> = Rc<RefCell<T>>;

pub struct Mutex<T> {
    value: UnsafeCell<T>,
    is_locked: Cell<bool>,
    queue: RefCell<VecDeque<Waker>>,
}

pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            is_locked: Cell::new(false),
            queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn lock(&self) -> LockFuture<'_, T> {
        LockFuture { lock: self }
    }

    pub fn raw_lock(&self) -> Option<()> {
        let already_locked = self.is_locked.replace(true);
        if already_locked {
            None
        } else {
            Some(())
        }
    }

    pub fn raw_unlock(&self) {
        self.is_locked.set(false);
        if let Some(waker) = self.queue.borrow_mut().pop_front() {
            waker.wake();
        }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.raw_unlock();
    }
}

pub struct LockFuture<'a, T> {
    lock: &'a Mutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let lock = &self.lock;
        if lock.raw_lock().is_none() {
            lock.queue.borrow_mut().push_back(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(MutexGuard { lock })
        }
    }
}

pub trait FutureExt: Future + Sized {
    fn spawn_with_handler(self, handler: impl FnOnce(<Self as Future>::Output) + 'static)
    where
        Self: 'static,
    {
        spawn_local(self.map(handler))
    }

    fn spawn_with_callback(self, callback: Callback<<Self as Future>::Output>)
    where
        Self: 'static,
    {
        self.spawn_with_handler(move |out| callback.emit(out))
    }
}

impl<T: Future> FutureExt for T {}

pub trait OptionExt<T> {
    fn log_none(self, msg: &str) -> Self;
    fn map_cb(self, callback: Callback<T>) -> Option<()>;
}

impl<T> OptionExt<T> for Option<T> {
    fn log_none(self, msg: &str) -> Self {
        if self.is_none() {
            error!("{}", msg);
        }
        self
    }
    fn map_cb(self, callback: Callback<T>) -> Option<()> {
        self.map(|x| callback.emit(x))
    }
}

pub trait ResultExt<T, E> {
    fn log_err(self) -> Self
    where
        E: Display;
    fn map_cb(self, callback: Callback<T>) -> Result<(), E>;
    fn map_err_cb(self, callback: Callback<E>) -> Result<T, ()>;
    fn ignore_err(self) -> Result<T, ()>;
    fn anyhow(self) -> Result<T, anyhow::Error>
    where
        E: Sync + Send + Debug + Display + 'static;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn log_err(self) -> Self
    where
        E: Display,
    {
        match &self {
            Err(err) => error!("{}", err),
            _ => (),
        }
        self
    }
    fn map_cb(self, callback: Callback<T>) -> Result<(), E> {
        self.map(|x| callback.emit(x))
    }
    fn map_err_cb(self, callback: Callback<E>) -> Result<T, ()> {
        self.map_err(|x| callback.emit(x))
    }
    fn ignore_err(self) -> Result<T, ()> {
        self.map_err(|_| ())
    }
    fn anyhow(self) -> Result<T, anyhow::Error>
    where
        E: Sync + Send + Debug + Display + 'static,
    {
        self.map_err(|err| anyhow::anyhow!(err))
    }
}
