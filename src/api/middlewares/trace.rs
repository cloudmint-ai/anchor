use crate::*;
use api::{HeaderValue, HttpRequest, HttpResponse, Response, TRACE, middleware::Next};
use tracing::{Instrument, info, info_span};

pub async fn trace(mut request: HttpRequest, next: Next) -> Response<HttpResponse> {
    let request_time = time::now();
    let request_method = request.method().to_string();
    let request_path = request.uri().path().to_string();
    let request_user_agent = match request.headers().get("User-Agent") {
        Some(user_agent) => user_agent.to_str()?,
        None => "",
    };

    let trace_id = match request.headers().get(TRACE) {
        Some(trace_id) => match trace_id.to_str()?.parse::<i64>() {
            Ok(trace_id) => Id::from(trace_id),
            Err(_) => Id::generate().await,
        },
        None => Id::generate().await,
    };
    let trace_id_i64: i64 = trace_id.into();
    let trace_span = info_span!("", trace_id_i64);
    trace_span.in_scope(|| {
        info!(
            "request {} {} [UserAgent {}]",
            request_method, request_path, request_user_agent,
        );
    });

    request.extensions_mut().insert(Context::new(trace_id));

    let mut response = next.run(request).instrument(trace_span.clone()).await;

    trace_span.in_scope(|| {
        info!(
            "request {} {} [Status {}] latency {} ms",
            request_method,
            request_path,
            response.status(),
            time::now() - request_time,
        );
    });

    response
        .headers_mut()
        .insert(TRACE, HeaderValue::from_str(&trace_id.to_string())?);

    Ok(response)
}
