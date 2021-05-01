use static_alloc::Bump;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use unsize::{Coercion, CoerceUnsize};
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU32, Ordering},
    task::{Context, Poll},
};

use cassette::Cassette;

static A: Bump<[u8; 4096]> = Bump::uninit();
static OML: OnceCell<Mutex<Yolo>> = OnceCell::new();

// shh bb no
unsafe impl Send for Yolo { }

struct Yolo {
    ip: Cassette<Pin<&'static mut dyn Future<Output = ()>>>
}

fn main() {
    println!("Hello, world!");

    // Setup...

    // Leak the object...
    let leaked_demo = A.leak(Demo { lol: 100 }).map_err(drop).unwrap();

    // Leak the future...
    let leaked = A.leak(leaked_demo.entry()).map_err(drop).unwrap();

    // Coerce it to a dyn Future...
    let deu = leaked.unsize(Coercion!(to dyn Future<Output=()>));

    // Pin it...
    let pdeu = unsafe { core::pin::Pin::new_unchecked(deu) };

    // Slam that future in the Cassette...
    let cas = Cassette::new(pdeu);

    // Stick the Cassette in a mutex oncecell to simulate
    // RTIC's `Resource`s
    OML.set(Mutex::new(Yolo { ip: cas })).map_err(drop).unwrap();

    // Run...
    loop {
        // Look! An interrupt occurred!
        fake_rtic_outer();
    }
}

fn fake_rtic_outer() {
    // This is simulating the guts of RTIC that gets the resource
    // and passes it into a task
    let mut ctxt = OML.get().unwrap().lock().unwrap();
    fake_rtic_inner(&mut ctxt);
}

fn fake_rtic_inner(ctxt: &mut Yolo) {
    // This is simulating an RTIC task
    match ctxt.ip.poll_on() {
        Some(data) => {
            panic!("Failed successfully: {:?}", data);
        }
        None => {
            println!("werk.");
        }
    }
}

///////////////////////////////////////////
// This is all the async state machine code
// I used when I wrote Cassette...

struct Demo {
    lol: u32,
}

impl Demo {
    async fn entry(&mut self) {
        for _ in 0..10 {
            self.entry_1().await;
            self.entry_2().await;
        }
    }

    async fn entry_1(&mut self) {
        self.start_at_zero().await;
        self.add_one_until_ten().await;
        self.sub_one_until_zero().await;
    }

    async fn entry_2(&mut self) {
        self.start_at_five().await;
        self.sub_one_until_zero().await;
        self.add_one_until_ten().await;
    }

    async fn start_at_zero(&mut self) {
        self.lol = 0;
    }

    async fn start_at_five(&mut self) {
        self.lol = 5;
    }

    async fn add_one_until_ten(&mut self) {
        loop {
            delay(self).await; // simulate fake delays/not ready state
            self.lol += 1;
            if self.lol >= 10 {
                return;
            }
        }
    }

    async fn sub_one_until_zero(&mut self) {
        loop {
            delay(self).await; // simulate fake delays/not ready state
            self.lol -= 1;
            if self.lol == 0 {
                return;
            }
        }
    }
}


static FAKE: AtomicU32 = AtomicU32::new(0);
struct CountFuture;
impl Future for CountFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let x = FAKE.fetch_add(1, Ordering::SeqCst);
        print!("{}, ", x);
        if (x % 5) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

async fn delay(ctxt: &mut Demo) {
    println!("delay says lol: {}", ctxt.lol);
    let x = CountFuture;
    x.await;
    println!("and delay!");
}
