#[cfg(feature = "tracing")]
pub(crate) use tracing::{debug, info};

#[cfg(not(feature = "tracing"))]
mod noop {
    macro_rules! debug {
        ($($a:tt)*) => {};
    }
    macro_rules! info {
        ($($a:tt)*) => {};
    }
    // macro_rules! error {
    //     ($($a:tt)*) => {};
    // }
    // macro_rules! warn {
    //     ($($a:tt)*) => {};
    // }
    pub(crate) use {debug, info};
}

#[cfg(not(feature = "tracing"))]
pub(crate) use noop::{debug, info};
