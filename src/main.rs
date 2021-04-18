#![allow(dead_code)]

// TODO: How could I make sure that context items used by
// a state are always initialized or in the right state?

struct Demo {
    lol: u32,
}

// Are these "non-leaf states"?
enum DemoTopStates {
    ZeroToTen,
    FiveToZero
}

// Are these "leaf states"?
enum DemoBottomStates {
    StartAtZero,
    StartAtFive,
    AddOneUntilTen,
    SubOneUntilZero,
}

trait Queue {
    type Item;
    fn peek(&self) -> Option<fn(&mut Self::Item) -> Result<bool, ()>>;
    fn pop(&mut self);
    fn resolve(self) -> Self;
}

struct SimpQueue<T: 'static> {
    queue: &'static [fn(&mut T) -> Result<bool, ()>],
    on_success: &'static [fn(&mut T) -> Result<bool, ()>],
}

trait Cassette<Q: Queue<Item = Self>> {
    type Item;
    fn set_queue(&mut self, q: Q);
    fn get_queue(&mut self) -> &mut Q;
    fn poll(&mut self) -> Result<(), ()>;
}

type BottomSequence = &'static [fn(&mut Demo) -> Result<bool, ()>];
type TopSequence = &'static [BottomSequence];

const ZERO_TO_TEN: BottomSequence = &[
    Demo::start_at_zero,
    Demo::add_one_until_ten,
];

const ZERO_TO_ZERO: BottomSequence = &[
    Demo::start_at_zero,
    Demo::add_one_until_ten,
    Demo::sub_one_until_zero,
];

impl Demo {
    fn zero_to_ten(&self) -> TopSequence {
        &[ZERO_TO_TEN]
    }

    fn zero_to_zero(&self) -> TopSequence {
        &[ZERO_TO_ZERO]
    }

    fn fancy(&self) -> TopSequence {
        &[
            ZERO_TO_TEN,
            ZERO_TO_ZERO,
            ZERO_TO_TEN,
        ]
    }
}

impl Demo {
    fn start_at_zero(&mut self) -> Result<bool, ()> {
        self.lol = 0;
        Ok(true)
    }

    fn start_at_five(&mut self) -> Result<bool, ()> {
        self.lol = 5;
        Ok(true)
    }

    fn add_one_until_ten(&mut self) -> Result<bool, ()> {
        self.lol += 1;
        Ok(self.lol >= 10)
    }

    fn sub_one_until_zero(&mut self) -> Result<bool, ()> {
        self.lol -= 1;
        Ok(self.lol == 0)
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
    let mut x = lol(&mut demo);

    let raw_waker = RawWaker::new(core::ptr::null(), &RWVT);
    let waker = unsafe { Waker::from_raw(raw_waker) };

    loop {
        let y = Future::poll(
            unsafe { Pin::new_unchecked(&mut x) },
            &mut Context::from_waker(&waker)
        );
        match y {
            Poll::Pending => { },
            Poll::Ready(yes) => {
                println!("Ready! {:?}", yes);
                break;
            }
        }
        // match lol(&mut demo).poll(()) {
        //     Poll::Pending => todo!(),
        //     Poll::Ready(stuff) => todo!()
        // }
    }
}

use std::sync::{
    atomic::AtomicU32,
    atomic::Ordering,
};

async fn lol(ctxt: &mut Demo) -> Result<bool, ()> {
    println!("boots!");
    cats(ctxt).await?;
    cats(ctxt).await?;
    cats(ctxt).await?;
    Ok(true)
}

static FAKE: AtomicU32 = AtomicU32::new(0);

async fn cats(ctxt: &mut Demo) -> Result<bool, ()> {
    let x = CountFuture;
    x.await;
    println!("and cats!");
    Ok(false)
}

struct CountFuture;

impl Future for CountFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let x = FAKE.fetch_add(1, Ordering::SeqCst);
        println!("    ...{}", x);
        if (x % 5) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
