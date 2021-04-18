#![cfg_attr(not(test), no_std)]

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

unsafe fn fake_clone(_: *const ()) -> RawWaker {
    todo!()
}

unsafe fn fake_wake(_: *const ()) {
    todo!()
}

unsafe fn fake_wake_by_ref(_: *const ()) {
    todo!()
}

unsafe fn fake_drop(_: *const ()) {
    // Don't panic, this does happen
    // TODO: ???
}

static RWVT: RawWakerVTable =
    RawWakerVTable::new(fake_clone, fake_wake, fake_wake_by_ref, fake_drop);

pub struct CasMachine<T>
where
    T: Future + Unpin,
{
    thing: T,
    fake_wake: Waker,
    done: bool,
}

impl<T> CasMachine<T>
where
    T: Future + Unpin,
{
    pub fn new(thing: T) -> Self {
        let raw_waker = RawWaker::new(core::ptr::null(), &RWVT);
        let waker = unsafe { Waker::from_raw(raw_waker) };


        Self {
            thing,
            fake_wake: waker,
            done: false,
        }
    }

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

    pub fn block_on(mut self) -> <T as Future>::Output {
        // TODO
        assert!(
            !self.done,
            "Blocked on completed future"
        );

        loop {
            if let Some(val) = self.poll_on() {
                return val;
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
}
