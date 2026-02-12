use crate::*;
use api::{
    AUTHORIZATION, Body, HttpRequest, HttpResponse, InternalServerError, Request, Response, State,
    Unauthorized, UnauthorizedError, VerifiableService, middleware::Next, parse_signature,
    to_bytes,
};

pub async fn authorize<S>(
    State(service): State<Arc<S>>,
    request: HttpRequest,
    next: Next,
) -> Response<HttpResponse>
where
    S: VerifiableService,
{
    let (head, body) = request.into_parts();
    let signature = match head.headers.get(AUTHORIZATION) {
        Some(header_value) => parse_signature(header_value.to_str()?.to_string())?,
        None => return Unauthorized!("header not found"),
    };

    let body_bytes = to_bytes(body).await?;

    let request: Request = match json::from_slice(&body_bytes) {
        Ok(data) => data,
        Err(_) => return Unauthorized!("request body parsed fail"),
    };

    if let Err(err) = service.verify(&body_bytes, &signature).await {
        return UnauthorizedError!(err);
    }

    let time_window_milliseconds = time::TimeDelta::from(service.time_window_seconds() * 1000);
    if time_window_milliseconds == time::TimeDelta::ZERO {
        return InternalServerError!("unexpected zero");
    }
    let now = time::now();

    if request.timestamp < now - time_window_milliseconds
        || request.timestamp > now + time_window_milliseconds
    {
        return Unauthorized!("unexpected request time");
    }

    let request = HttpRequest::from_parts(head, Body::from(body_bytes));
    Ok(next.run(request).await)
}
