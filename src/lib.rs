#![cfg_attr(not(test), no_std)]

//! A simple, single-future, non-blocking executor intended for building state machines. Designed to be no-std and embedded friendly.
//!
//! This executor TOTALLY IGNORES wakers and context, meaning that all async functions should expect to be polled repeatedly until completion.
//!
//! ## Inspiration
//!
//! So, I'm really not good at async, but I like the idea of being able to use the ability to yield or await on tasks that will require some time to complete.
//!
//! The idea here is that you would write one, top level `async` function that would either eventually resolve to some value, or that will run forever (to act as a state machine).
//!
//! ## How it works
//!
//! 1. You write some async functions
//! 2. You call the "top level" async function
//! 3. You poll on it until it resolves (or forever)
//!
//! Note: This demo is available in the [`demo/` folder](./../demo) of this repo.
//!
//! ### Step 1 - You write some async functions
//!
//! Here's the "context" of our state machine, describing a couple of high level behaviors, as well as individual substeps.
//!
//! ```rust
//! struct Demo {
//!     lol: u32,
//! }
//!
//! impl Demo {
//!     async fn entry(&mut self) {
//!         for _ in 0..10 {
//!             self.entry_1().await;
//!             self.entry_2().await;
//!         }
//!     }
//!
//!     async fn entry_1(&mut self) {
//!         self.start_at_zero().await;
//!         self.add_one_until_ten().await;
//!         self.sub_one_until_zero().await;
//!     }
//!
//!     async fn entry_2(&mut self) {
//!         self.start_at_five().await;
//!         self.sub_one_until_zero().await;
//!         self.add_one_until_ten().await;
//!     }
//!
//!     async fn start_at_zero(&mut self) {
//!         self.lol = 0;
//!     }
//!
//!     async fn start_at_five(&mut self) {
//!         self.lol = 5;
//!     }
//!
//!     async fn add_one_until_ten(&mut self) {
//!         loop {
//!             delay(self).await; // simulate fake delays/not ready state
//!             self.lol += 1;
//!             if self.lol >= 10 {
//!                 return;
//!             }
//!         }
//!     }
//!
//!     async fn sub_one_until_zero(&mut self) {
//!         loop {
//!             delay(self).await; // simulate fake delays/not ready state
//!             self.lol -= 1;
//!             if self.lol == 0 {
//!                 return;
//!             }
//!         }
//!     }
//! }
//!
//! # use core::{
//! #     future::Future,
//! #     pin::Pin,
//! #     sync::atomic::{AtomicU32, Ordering},
//! #     task::{Context, Poll},
//! # };
//! # static FAKE: AtomicU32 = AtomicU32::new(0);
//! # struct CountFuture;
//! # impl Future for CountFuture {
//! #     type Output = ();
//! #     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//! #         let x = FAKE.fetch_add(1, Ordering::SeqCst);
//! #         print!("{}, ", x);
//! #         if (x % 5) == 0 {
//! #             Poll::Ready(())
//! #         } else {
//! #             cx.waker().wake_by_ref();
//! #             Poll::Pending
//! #         }
//! #     }
//! # }
//! #
//! # async fn delay(ctxt: &mut Demo) {
//! #     println!("delay says lol: {}", ctxt.lol);
//! #     let x = CountFuture;
//! #     x.await;
//! #     println!("and delay!");
//! # }
//! ```
//!
//! We can also make simple little futures for code that needs to be polled until ready:
//!
//! ```rust
//! # use core::{
//! #     future::Future,
//! #     pin::Pin,
//! #     sync::atomic::{AtomicU32, Ordering},
//! #     task::{Context, Poll},
//! # };
//! static FAKE: AtomicU32 = AtomicU32::new(0);
//! struct CountFuture;
//! impl Future for CountFuture {
//!     type Output = ();
//!     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//!         let x = FAKE.fetch_add(1, Ordering::SeqCst);
//!         print!("{}, ", x);
//!         if (x % 5) == 0 {
//!             Poll::Ready(())
//!         } else {
//!             cx.waker().wake_by_ref();
//!             Poll::Pending
//!         }
//!     }
//! }
//!
//! async fn delay(ctxt: &mut Demo) {
//!     println!("delay says lol: {}", ctxt.lol);
//!     let x = CountFuture;
//!     x.await;
//!     println!("and delay!");
//! }
//! #
//! # struct Demo {
//! #     lol: u32,
//! # }
//! ```
//!
//! ### Step 2 - You call the "top level" async function
//!
//! ```rust
//! # struct Demo {
//! #     lol: u32,
//! # }
//! #
//! # impl Demo {
//! #     async fn entry(&mut self) {
//! #         panic!()
//! #     }
//! # }
//! #
//!
//! use cassette::Cassette;
//!
//! fn main() {
//!     // Make a new struct
//!     let mut demo = Demo { lol: 100 };
//!
//!     // Call the entry point future, and pin it
//!     let x = core::pin::pin!(demo.entry());
//!
//!     // Give the pinned future to Cassette
//!     // for execution
//!     let mut cm = Cassette::new(x);
//!
//!     /* ... */
//! }
//! ```
//!
//! ### Step 3 - You poll on it until it resolves (or forever)
//!
//! ```rust
//! # use cassette::Cassette;
//!
//! # struct Demo {
//! #     lol: u32,
//! # }
//! #
//! # impl Demo {
//! #     async fn entry(&mut self) {
//! #     }
//! # }
//! #
//! fn main() {
//! #    // Make a new struct
//! #    let mut demo = Demo { lol: 100 };
//! #
//! #    // Call the entry point future, and pin it
//! #    let x = core::pin::pin!(demo.entry());
//! #
//! #    // Give the pinned future to Cassette
//! #    // for execution
//! #    let mut cm = Cassette::new(x);
//!     /* ... */
//!
//!     loop {
//!         if let Some(x) = cm.poll_on() {
//!             println!("Done!: `{:?}`", x);
//!             break;
//!         }
//!     }
//! }
//! ```
//!
//! ## A larger demo
//!
//! If you'd like to see a larger demo, I used Cassette to implement an I2C peripheral bootloader state machine for a `thumbv6m` target. You can check out [that PR](https://github.com/sprocket-board/sprocket-boot/pull/1) for more context.
//!
//! ## License
//!
//! [MPL v2.0](./../LICENSE)

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

fn no_op(_: *const ()) {}
fn no_op_clone(_: *const ()) -> RawWaker {
    noop_raw_waker()
}

static RWVT: RawWakerVTable = RawWakerVTable::new(no_op_clone, no_op, no_op, no_op);

#[inline]
fn noop_raw_waker() -> RawWaker {
    RawWaker::new(core::ptr::null(), &RWVT)
}

/// A single-future non-blocking executor
pub struct Cassette<T>
where
    T: Future + Unpin,
{
    thing: T,
    fake_wake: Waker,
    done: bool,
}

impl<T> Cassette<T>
where
    T: Future + Unpin,
{
    /// Create a new Cassette containing a single pinned future
    ///
    /// # Example
    ///
    /// ```
    /// use cassette::Cassette;
    ///
    /// struct Demo {
    ///     lol: u32,
    /// }
    ///
    /// impl Demo {
    ///     async fn entry(&mut self) {
    ///         /* Huzzah! */
    ///     }
    /// }
    ///
    /// // Make a new struct
    /// let mut demo = Demo { lol: 100 };
    ///
    /// // Call the entry point future, and pin it
    /// let x = core::pin::pin!(demo.entry());
    ///
    /// // Give the pinned future to Cassette
    /// // for execution
    /// let mut cm = Cassette::new(x);
    /// ```
    pub fn new(thing: T) -> Self {
        let raw_waker = noop_raw_waker();
        let waker = unsafe { Waker::from_raw(raw_waker) };

        Self {
            thing,
            fake_wake: waker,
            done: false,
        }
    }

    /// Perform a "single step" of the future contained by this
    /// Cassette.
    ///
    /// This is intended to be "polled to completion", which
    /// might be for forever.
    ///
    /// # Example
    ///
    /// ```
    /// use cassette::Cassette;
    ///
    /// struct Demo {
    ///     lol: u32,
    /// }
    ///
    /// impl Demo {
    ///     async fn entry(&mut self) {
    ///         /* Huzzah! */
    ///     }
    /// }
    ///
    /// // Make a new struct
    /// let mut demo = Demo { lol: 100 };
    ///
    /// // Call the entry point future, and pin it
    /// let x = core::pin::pin!(demo.entry());
    ///
    /// // Give the pinned future to Cassette
    /// // for execution
    /// let mut cm = Cassette::new(x);
    ///
    /// while cm.poll_on().is_none() { }
    /// println!("Future done!");
    /// ```
    //
    // TODO: try_poll_on where an error is returned
    // if `self.done == true`?
    pub fn poll_on(&mut self) -> Option<<T as Future>::Output> {
        if self.done {
            todo!("Polled a done future");
        }

        let mut ctxt = Context::from_waker(&self.fake_wake);
        let y = Pin::new(&mut self.thing).poll(&mut ctxt);
        match y {
            Poll::Pending => None,
            Poll::Ready(yes) => {
                self.done = true;
                Some(yes)
            }
        }
    }

    /// Block on the contained future forever
    ///
    /// ## Panics
    ///
    /// This method will panic if the contained future has already
    /// been completed as `Poll::Ready(_)`.
    pub fn block_on(mut self) -> <T as Future>::Output {
        // TODO
        assert!(!self.done, "Blocked on completed future");

        loop {
            if let Some(val) = self.poll_on() {
                return val;
            }
        }
    }

    /// Has the contained future resolved to `Poll::Ready(_)` yet?
    pub fn is_done(&self) -> bool {
        self.done
    }
}

/// Cooperatively gives up a timeslice to the task scheduler.
#[inline]
pub async fn yield_now() {
    YieldNow(false).await
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    // inspired by async-std v1.9.0
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;

            // Wake immediately, so we get polled again
            // in the next timeslice.
            //
            // Not necessary if our executor is `Cassette`, but
            // on other executors we might produce a deadlock
            // otherwise.
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

/// Runs a future to completion on the current thread.
///
/// This function will block the caller until the given future has completed.
pub fn block_on<F: Future>(f: F) -> <F as Future>::Output {
    let f = core::pin::pin!(f);
    Cassette::new(f).block_on()
}
