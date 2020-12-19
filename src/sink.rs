/// An `IterSink` is a value into which other values can be sent.
pub trait IterSink<Item> {
    /// The type of value produced by the sink when an error occurs.
    type Error;

    /// Attempts to prepare the `IterSink` to receive a value.
    ///
    /// This method must be called prior to each call to `start_send`.
    fn ready(&mut self) -> Result<(), Self::Error>;

    /// Begin the process of sending a value to the sink.
    /// Each call to this function must be preceded by a successful call to
    /// `ready`.
    fn start_send(&mut self, item: Item) -> Result<(), Self::Error>;

    /// Flush any remaining output from this sink.
    fn flush(&mut self) -> Result<(), Self::Error>;
}

/// An extension trait for `IterSink`s that provides a few convenient combinator functions.
pub trait IterSinkExt<Item>: IterSink<Item> {
    /// Fully processed an item into the sink, including flushing.
    ///
    /// Note that, because of the flushing requirement, it is usually better to batch together items to send via send_all, rather than flushing between each item.
    fn send(&mut self, item: Item) -> Result<(), Self::Error> {
        self.send_all(std::iter::once(item))
    }

    /// Fully process the iterator of items into the sink, including flushing.
    ///
    /// This future will drive the iterator to keep producing items until it is exhausted, sending each item to the sink. It will complete once both the stream is exhausted, the sink has received all items, and the sink has been flushed.
    fn send_all(&mut self, items: impl IntoIterator<Item = Item>) -> Result<(), Self::Error> {
        for item in items {
            self.ready()?;
            self.start_send(item)?;
        }
        self.flush()
    }
}

impl<T: ?Sized, Item> IterSinkExt<Item> for T where T: IterSink<Item> {}
