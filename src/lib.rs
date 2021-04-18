use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

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

static RWVT: RawWakerVTable =
    RawWakerVTable::new(fake_clone, fake_wake, fake_wake_by_ref, fake_drop);

pub struct CasMachine<'a, T>
where
    T: Future,
{
    thing: Pin<&'a mut T>,
    fake_wake: Waker,
    done: bool,
}

impl<'a, T> CasMachine<'a, T>
where
    T: Future,
{
    pub fn new(thing: Pin<&'a mut T>) -> Self {
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
            return None
        }

        let mut ctxt = Context::from_waker(&self.fake_wake);
        let y = self.thing.as_mut().poll(&mut ctxt);
        match y {
            Poll::Pending => None,
            Poll::Ready(yes) => {
                self.done = true;
                Some(yes)
            }
        }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
}
