use super::Body;
use crate::*;

pub async fn to_bytes(body: Body) -> Result<Vec<u8>> {
    Ok(axum::body::to_bytes(body, usize::MAX).await?.to_vec())
}
