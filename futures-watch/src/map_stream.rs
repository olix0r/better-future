use futures::{Poll, Stream};

/// Maps borrowed references to `T` into an `Item`.
pub trait Map<T> {
    /// The output type.
    type Output;


    /// What you get when Map fails.
    type Error;

    /// Produces a new Output value.
    fn map(&mut self, t: &T) -> Result<Self::Output, Self::Error>;
}

/// Each time the underlying `Watch<T>` is updated, the stream maps over the most-recent
/// value.
#[derive(Debug)]
pub struct MapStream<T, M: Map<T>> {
    watch: super::Watch<T>,
    map: M,
}

impl<T, M: Map<T>> MapStream<T, M> {
    pub(crate) fn new(watch: super::Watch<T>, map: M) -> Self {
        Self { watch, map }
    }
}

impl<T, M: Map<T>> Stream for MapStream<T, M> {
    type Item = <M as Map<T>>::Output;
    type Error = Error<<M as Map<T>>::Error>;

    fn poll(&mut self) -> Poll<Option<M::Output>, Self::Error> {
        try_ready!(self.watch.poll().map_err(Error::WatchError));

        let item = self.map.map(&*self.watch.borrow())
            .map_err(Error::Map)?;

        Ok(Some(item).into())
    }
}

impl<T, M: Clone + Map<T>> Clone for MapStream<T, M> {
    fn clone(&self) -> Self {
        Self {
            watch: self.watch.clone(),
            map: self.map.clone(),
        }
    }
}

/// Errors produced by `MapStream::poll`.
#[derive(Debug)]
pub enum Error<E> {
    /// An error mapping to a new value may be transient.
    Map(E),
    /// An error polling the underlying `Watch`. Probably fatal.
    WatchError(super::WatchError),
}
