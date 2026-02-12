#[macro_export]
macro_rules! init {
    ($e:expr) => {
        config::init!($e);
        log::init()?
        // TODO id init
    };
}
