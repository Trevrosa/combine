#[cfg(not(feature = "tracing"))]
mod noop {
    #[macro_export]
    macro_rules! debug {
        ($($a:tt)*) => {};
    }
    #[macro_export]
    macro_rules! error {
        ($($a:tt)*) => {};
    }
    #[macro_export]
    macro_rules! info {
        ($($a:tt)*) => {};
    }
    #[macro_export]
    macro_rules! warn {
        ($($a:tt)*) => {};
    }
}
