use super::Router;
use crate::*;
use axum::serve as axum_serve;
use runtime::TcpListener;

pub async fn serve(root: Router, host: config::Host) -> Result<()> {
    let listener = TcpListener::bind(host.for_service()?).await?;
    Ok(axum_serve(listener, root).await?)
}

#[async_trait]
pub trait VerifiableService {
    // TODO make it static LazyLock, make it return DurationMs
    fn time_window_seconds(&self) -> i64;
    async fn verify(&self, message: &[u8], signature_base64: &str) -> Result<()>;
}
