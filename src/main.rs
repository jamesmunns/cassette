#![allow(dead_code, unused_imports, unused_variables)]

use futures::{
    pin_mut,
    future::poll_fn,
};

struct Demo {
    lol: u32,
}

impl Demo {
    async fn entry(&mut self) -> Result<bool, ()> {
        self.start_at_zero().await;
        self.add_one_until_ten().await;
        self.sub_one_until_zero().await;
        Ok(true)
    }


    async fn start_at_zero(&mut self) {
        self.lol = 0;
    }

    async fn start_at_five(&mut self) {
        self.lol = 5;
    }

    async fn add_one_until_ten(&mut self) {
        loop {
            cats(self).await; // simulate fake delays/not ready state
            self.lol += 1;
            if self.lol >= 10 {
                return
            }
        }
    }

    async fn sub_one_until_zero(&mut self) {
        loop {
            cats(self).await; // simulate fake delays/not ready state
            self.lol -= 1;
            if self.lol == 0 {
                return
            }
        }
    }
}

use core::future::Future;
use core::task::{
    Poll,
    RawWakerVTable,
    RawWaker,
    Context,
    Waker,
};
use core::pin::Pin;

unsafe fn fake_clone(_: *const ()) -> RawWaker {
    println!("!!! - fake_clone!");
    todo!()
}

unsafe fn fake_wake(_: *const ()) {
    println!("!!! - fake_wake!")
}

unsafe fn fake_wake_by_ref(_: *const ()) {
    println!("!!! - fake_wake_by_ref!")
}

unsafe fn fake_drop(_: *const ()) {
    println!("!!! - fake_drop!")
}

static RWVT: RawWakerVTable = RawWakerVTable::new(
    fake_clone,
    fake_wake,
    fake_wake_by_ref,
    fake_drop
);

fn main() {
    let mut demo = Demo { lol: 100 };
    let x = demo.entry();
    pin_mut!(x);

    let raw_waker = RawWaker::new(core::ptr::null(), &RWVT);
    let waker = unsafe { Waker::from_raw(raw_waker) };

    loop {
        let y = x.as_mut().poll(&mut Context::from_waker(&waker));
        match y {
            Poll::Pending => { },
            Poll::Ready(yes) => {
                println!("Ready! {:?}", yes);
                break;
            }
        }
    }
}

use std::sync::{
    atomic::AtomicU32,
    atomic::Ordering,
};

static FAKE: AtomicU32 = AtomicU32::new(0);

async fn cats(ctxt: &mut Demo) {
    println!("cats says lol: {}", ctxt.lol);
    let x = CountFuture;
    x.await;
    println!("and cats!");
}

struct CountFuture;

impl Future for CountFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let x = FAKE.fetch_add(1, Ordering::SeqCst);
        print!("{}, ", x);
        if (x % 5) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
