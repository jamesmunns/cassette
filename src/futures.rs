//! Items pulled from the `futures` crate
//!
//! This module is necessary due to a lack of ability to disable
//! the use of atomic CAS operations in the futures crate. Eventually
//! this module should go away.
//
// This module includes code licensed under the MIT+Apache2.0 dual
// licenses from the `futures-rs` crate. Please see the upstream
// repository at https://github.com/rust-lang/futures-rs, and details
// of the license here: https://github.com/rust-lang/futures-rs#license

use core::fmt;
use core::pin::Pin;
use core::future::Future;
use core::task::{Context, Poll};

/// Pins a value on the stack.
///
/// NOTE: Taken from `futures::pin_mut!()`
///
/// # Example
///
/// ```rust
/// # use cassette::pin_mut;
/// # use core::pin::Pin;
/// # struct Foo {}
/// let foo = Foo { /* ... */ };
/// pin_mut!(foo);
/// let _: Pin<&mut Foo> = foo;
/// ```
#[macro_export]
#[deprecated(since = "0.2.4", note = "use `core::pin::pin` instead")]
macro_rules! pin_mut {
    ($($x:ident),* $(,)?) => { $(
        // Move the value to ensure that it is owned
        let mut $x = $x;
        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $x = unsafe {
            core::pin::Pin::new_unchecked(&mut $x)
        };
    )* }
}


// Just a helper function to ensure the futures we're returning all have the
// right implementations.
pub(crate) fn assert_future<T, F>(future: F) -> F
where
    F: Future<Output = T>,
{
    future
}

/// Future for the [`poll_fn`] function.
#[deprecated(since = "0.2.4", note = "use `core::future::PollFn` instead")]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PollFn<F> {
    f: F,
}

impl<F> Unpin for PollFn<F> {}

/// Creates a new future wrapping around a function returning [`Poll`].
///
/// Polling the returned future delegates to the wrapped function.
///
/// # Examples
///
/// ```no_run
/// use cassette::futures::poll_fn;
/// use core::task::{Context, Poll};
///
/// fn read_line(_cx: &mut Context<'_>) -> Poll<String> {
///     Poll::Ready("Hello, World!".into())
/// }
///
/// # async fn func() {
/// let read_future = poll_fn(read_line);
/// assert_eq!(read_future.await, "Hello, World!".to_owned());
/// # }
/// ```
#[deprecated(since = "0.2.4", note = "use `core::future::poll_fn` instead")]
pub fn poll_fn<T, F>(f: F) -> PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>
{
    assert_future::<T, _>(PollFn { f })
}

impl<F> fmt::Debug for PollFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PollFn").finish()
    }
}

impl<T, F> Future for PollFn<F>
    where F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        (&mut self.f)(cx)
    }
}
