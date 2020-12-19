// https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/cfg.rs

macro_rules! cfg_async {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "async")]
            #[cfg_attr(docsrs, doc(cfg(feature = "async")))]
            $item
        )*
    }
}

macro_rules! cfg_blocking {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "blocking")]
            #[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
            $item
        )*
    }
}
