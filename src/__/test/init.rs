use crate::*;

// 这种运行时的判断变量是没用的，因为测试期间都是符合条件的，会隐藏问题
// pub(super) static TESTING: OnceLock<bool> = OnceLock::new();
// 同时 在macro_export 里的 cfg 也是没用的，是先cfg 再macro展开的。
// 因此 只有quote! 内的cfg test标记，才是有意义的

#[cfg(feature = "async")]
use tracing_subscriber::{EnvFilter, FmtSubscriber, filter::LevelFilter, fmt::format};

// DO NOT invoke it manually, it is done by test::case
pub fn init() {
    #[cfg(feature = "async")]
    init_log();
}

#[cfg(feature = "async")]
static INIT: Once = Once::new();

#[cfg(feature = "async")]
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

tests! {
    fn test_log() {
        #[cfg(feature = "async")]
        init_log();
        info!("test info");
        warn!("test warn");
        error!("test error");
    }
}
