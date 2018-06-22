use futures::{Poll, Stream};

use {Watch, WatchError};

/// Maps borrowed references to `T` into an `Item`.
pub trait Then<T> {
    /// The output type.
    type Output;

    /// What you get when Map fails.
    type Error;

    /// Produces a new Output value.
    fn then(&mut self, t: &T) -> Result<Self::Output, Self::Error>;
}

/// Each time the underlying `Watch<T>` is updated, the stream maps over the most-recent
/// value.
#[derive(Debug)]
pub struct ThenStream<T, M: Then<T>> {
    watch: Watch<T>,
    then: M,
}

impl<T, M: Then<T>> ThenStream<T, M> {
    pub(crate) fn new(watch: Watch<T>, then: M) -> Self {
        Self { watch, then }
    }
}

impl<T, M: Then<T>> Stream for ThenStream<T, M> {
    type Item = <M as Then<T>>::Output;
    type Error = Error<<M as Then<T>>::Error>;

    fn poll(&mut self) -> Poll<Option<M::Output>, Self::Error> {
        try_ready!(self.watch.poll().map_err(Error::WatchError));

        let item = self.then.then(&*self.watch.borrow())
            .map_err(Error::Then)?;

        Ok(Some(item).into())
    }
}

impl<T, M: Clone + Then<T>> Clone for ThenStream<T, M> {
    fn clone(&self) -> Self {
        Self {
            watch: self.watch.clone(),
            then: self.then.clone(),
        }
    }
}

/// Errors produced by `MapStream::poll`.
#[derive(Debug)]
pub enum Error<E> {
    /// An error mapping to a new value may be transient.
    Then(E),
    /// An error polling the underlying `Watch`. Probably fatal.
    WatchError(WatchError),
}
