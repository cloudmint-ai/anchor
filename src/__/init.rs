#[macro_export]
macro_rules! __init_for_runtime {
    ($e:expr) => {
        config::init!($e);
        log::init()?
        // TODO id init
    };
}
