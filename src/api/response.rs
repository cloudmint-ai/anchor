use super::{Body, HttpResponse, IntoHttpResponse, Json, StatusCode};
use crate::*;

pub type Response<T> = std::result::Result<T, ErrorResponse>;

// TODO re define Response after std::ops::Try stable
// 限制必须使用Protocol 进行返回值
// use std::ops::Try;
// pub enum Response<T>
// where
//     T: ResponseProtocol,
// {
//     Ok(T),
//     Err(ErrorResponse),
// }
// pub trait ResponseProtocol {}
// impl ResponseProtocol for () {}
// impl ResponseProtocol for HttpResponse {}
// impl<T: Protocol> ResponseProtocol for Vec<T> {}
// impl<T: Protocol> ResponseProtocol for Json<T> {}
// impl ResponseProtocol for Pdf {}

impl IntoHttpResponse for Pdf {
    fn into_response(self) -> HttpResponse {
        match HttpResponse::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/pdf")
            .body(Body::from(self.0))
        {
            Ok(response) => response,
            Err(error) => ErrorResponse::from(error).into_response(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorResponse {
    InternalServerError(Error),
    BadRequest(Error),
    Unauthorized(Error),
    EngineError { code: i64 },
}

impl ErrorResponse {
    fn get_message(status_code: StatusCode, err: Error) -> String {
        if !config::is_production() {
            return err.to_string();
        }
        if let Some(reason) = status_code.canonical_reason() {
            return reason.to_string();
        }
        return "".to_owned();
    }
    pub fn map_err(error_response: Self) -> Error {
        match error_response {
            ErrorResponse::InternalServerError(error) => error,
            ErrorResponse::BadRequest(error) => error,
            ErrorResponse::Unauthorized(error) => error,
            ErrorResponse::EngineError { code } => Error::EngineError(code),
        }
    }
}

impl IntoHttpResponse for ErrorResponse {
    fn into_response(self) -> HttpResponse {
        let (status, body) = match self {
            ErrorResponse::InternalServerError(err) => {
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                (
                    status_code,
                    json::value!({ "message": ErrorResponse::get_message(status_code, err) }),
                )
            }
            ErrorResponse::Unauthorized(err) => {
                let status_code = StatusCode::UNAUTHORIZED;
                (
                    status_code,
                    json::value!({ "message": ErrorResponse::get_message(status_code, err) }),
                )
            }
            ErrorResponse::BadRequest(err) => {
                let status_code = StatusCode::BAD_REQUEST;
                (
                    status_code,
                    json::value!({ "message": ErrorResponse::get_message(status_code, err) }),
                )
            }
            ErrorResponse::EngineError { code } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json::value!({ "code": code }),
            ),
        };
        (status, Json(body)).into_response()
    }
}

impl From<Error> for ErrorResponse {
    fn from(error: Error) -> Self {
        match error {
            Error::EngineError(code) => ErrorResponse::EngineError { code },
            Error::UnexpectedError(_) => ErrorResponse::InternalServerError(error),
        }
    }
}

impl<E> From<E> for ErrorResponse
where
    E: std::error::Error,
{
    fn from(error: E) -> Self {
        ErrorResponse::InternalServerError(error.into())
    }
}

#[macro_export]
macro_rules! ApiInternalServerError {
    ($($arg:tt)*) => {
        Err(api::ErrorResponse::InternalServerError(Error::_unexpected(format!($($arg)*))))
    };
}

#[macro_export]
macro_rules! ApiUnauthorized {
    ($($arg:tt)*) => {
        Err(api::ErrorResponse::Unauthorized(Error::_unexpected(format!($($arg)*))))
    };
}

#[macro_export]
macro_rules! ApiUnauthorizedError {
    ($($arg:tt)*) => {
        Err(api::ErrorResponse::Unauthorized($($arg)*))
    };
}
