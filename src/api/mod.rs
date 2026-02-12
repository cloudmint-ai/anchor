mod request;
pub use request::*;

mod response;
pub use response::*;

mod point;
pub use point::*;

mod header;
pub use header::*;

mod client;
pub use client::*;

mod service;
pub use service::*;

pub mod middlewares;

mod route;
pub use route::*;

pub use macros::Protocol;
pub mod protocol;
pub use protocol::Protocol;

pub use crate::ApiInternalServerError as InternalServerError;
pub use crate::ApiUnauthorized as Unauthorized;
pub use crate::ApiUnauthorizedError as UnauthorizedError;

pub(crate) use axum::{
    extract::Request as HttpRequest,
    handler::Handler,
    http::header::HeaderValue,
    response::{IntoResponse as IntoHttpResponse, Response as HttpResponse},
    routing::{get, post},
    // TODO 支持正常登陆和内部鉴权
    // TODO recover it routing::{get, get_service, post},
};

pub use axum::{
    Router,
    body::{Body, Bytes},
    extract::{Extension, Json, Path, Query, State},
    http::{Method, StatusCode},
    middleware,
};

mod utils;
use utils::*;
