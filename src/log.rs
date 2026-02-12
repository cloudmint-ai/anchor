use crate::*;
pub use tracing::Level;

#[cfg(any(feature = "cloud", feature = "wasm"))]
fn get_level() -> Result<Level> {
    let level = config!().log.level;
    if level == Level::DEBUG && config::is_production() {
        return Unexpected!("debugging release");
    }
    Ok(level)
}

pub fn init() -> Result<()> {
    // TODO cloud 和 比如 desktop 产物啥的作为默认选项
    #[cfg(feature = "cloud")]
    init_cloud()?;

    #[cfg(feature = "wasm")]
    init_wasm()?;

    Ok(())
}

#[cfg(feature = "cloud")]
use tracing_subscriber::{
    FmtSubscriber,
    fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields, format},
    registry::LookupSpan,
};

#[cfg(feature = "cloud")]
struct EventFormatter;

#[cfg(feature = "cloud")]
impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let utc = time::utc();
        let mut trace_id = Context::BACKGROUND_TRACE_ID;

        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                let ext = span.extensions();
                let fields = &ext.get::<FormattedFields<N>>().expect("get fields fail");
                if !fields.is_empty() {
                    let items: Vec<_> = fields.split("=").collect();
                    if items.len() >= 2 {
                        if let Ok(num) = items[1].parse::<i64>() {
                            trace_id = Id::from(num)
                        }
                    }
                }
            }
        }

        let level = event.metadata().level();
        write!(writer, "{} [{}] {} : ", utc, trace_id, level)?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer, " [{:0>2?}]", std::thread::current().id())
    }
}

#[cfg(feature = "cloud")]
pub fn init_cloud() -> Result<()> {
    let level = get_level()?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .event_format(EventFormatter {})
        .finish();

    Ok(tracing::subscriber::set_global_default(subscriber)?)
}

#[cfg(feature = "wasm")]
use tracing_subscriber::{Registry, layer::SubscriberExt};

#[cfg(feature = "wasm")]
use tracing_wasm::{WASMLayer, WASMLayerConfigBuilder};

#[cfg(feature = "wasm")]
pub fn init_wasm() -> Result<()> {
    let level = get_level()?;
    let wasm_layer = WASMLayer::new(WASMLayerConfigBuilder::new().set_max_level(level).build());
    let subscriber = Registry::default().with(wasm_layer);
    Ok(tracing::subscriber::set_global_default(subscriber)?)
}
