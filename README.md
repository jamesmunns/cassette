# Cassette

A simple, single-future, non-blocking executor intended for building state machines. Designed to be no-std and embedded friendly.

This executor TOTALLY IGNORES wakers and context, meaning that all async functions should expect to be polled repeatedly until completion.

## Inspiration

So, I'm really not good at async, but I like the idea of being able to use the ability to yield or await on tasks that will require some time to complete.

The idea here is that you would write one, top level `async` function that would either eventually resolve to some value, or that will run forever (to act as a state machine).

## How it works

1. You write some async functions
2. You call the "top level" async function
3. You poll on it until it resolves (or forever)

Note: This demo is available in the [`demo/` folder](./demo) of this repo.

### Step 1 - You write some async functions

Here's the "context" of our state machine, describing a couple of high level behaviors, as well as individual substeps.

```rust
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
```

We can also make simple little futures for code that needs to be polled until ready:

```rust
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
```

### Step 2 - You call the "top level" async function

```rust
fn main() {
    // Make a new struct
    let mut demo = Demo { lol: 100 };

    // Call the entry point future, and pin it
    let x = demo.entry();
    pin_mut!(x);

    // Give the pinned future to Cassette
    // for execution
    let mut cm = Cassette::new(x);

    /* ... */
}
```

### Step 3 - You poll on it until it resolves (or forever)

```rust
fn main() {
    /* ... */

    loop {
        if let Some(x) = cm.poll_on() {
            println!("Done!: `{:?}`", x);
            break;
        }
    }
}
```

## A larger demo

If you'd like to see a larger demo, I used Cassette to implement an I2C peripheral bootloader state machine for a `thumbv6m` target. You can check out [that PR](https://github.com/sprocket-board/sprocket-boot/pull/1) for more context.

## License

This crate is licensed under the [MPL v2.0](./LICENSE).

The `futures` module includes code licensed under the MIT+Apache2.0 dual
licenses from the `futures-rs` crate. Please see the upstream
repository at https://github.com/rust-lang/futures-rs, and details
of the license here: https://github.com/rust-lang/futures-rs#license
