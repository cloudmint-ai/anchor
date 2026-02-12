#[cfg(feature = "runtime")]
use crate::runtime::Once;

#[cfg(feature = "runtime")]
use tracing_subscriber::{EnvFilter, FmtSubscriber, filter::LevelFilter, fmt::format};

// DO NOT invoke it manually, it is done by test::case
pub fn init() {
    #[cfg(feature = "runtime")]
    init_log();
}

#[cfg(feature = "runtime")]
static INIT: Once = Once::new();

#[cfg(feature = "runtime")]
fn init_log() {
    // 默认 INFO，若需要调整Level 进行单个测试追查，使用终端任务脚本并结合环境变量 RUST_LOG=DEBUG cargo test ....
    INIT.call_once(|| {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .with_thread_ids(true)
            .event_format(
                format()
                    .compact()
                    .with_target(false)
                    .with_thread_ids(true)
                    .with_level(true),
            )
            .finish();

        tracing::subscriber::set_global_default(subscriber).unwrap()
    })
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "runtime")]
    use super::*;
    use crate::*;

    #[test::case]
    fn test_log() {
        #[cfg(feature = "runtime")]
        init_log();
        info!("test info");
        warn!("test warn");
        error!("test error");
    }
}
