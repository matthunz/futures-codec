mod framed_read;

struct Fuse<T, U>(T, U);

pub trait Decoder {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
