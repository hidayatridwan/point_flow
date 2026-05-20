use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tower_http::request_id::RequestId;
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

tokio::task_local! {
    static CURRENT_REQUEST_ID: String;
}

pub async fn bind_request_id(mut request: Request, next: Next) -> Response {
    let request_id = request_id_from_extension(&request)
        .or_else(|| request_id_from_header(&request))
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Ok(value) = HeaderValue::from_str(&request_id) {
        request.headers_mut().insert(REQUEST_ID_HEADER, value);
    }

    CURRENT_REQUEST_ID
        .scope(request_id.clone(), async move {
            let mut response = next.run(request).await;

            if let Ok(value) = HeaderValue::from_str(&request_id) {
                response.headers_mut().insert(REQUEST_ID_HEADER, value);
            }

            response
        })
        .await
}

pub fn current_request_id() -> String {
    CURRENT_REQUEST_ID
        .try_with(Clone::clone)
        .unwrap_or_else(|_| Uuid::new_v4().to_string())
}

fn request_id_from_extension(request: &Request) -> Option<String> {
    request
        .extensions()
        .get::<RequestId>()
        .and_then(|request_id| request_id.header_value().to_str().ok())
        .map(str::to_owned)
}

fn request_id_from_header(request: &Request) -> Option<String> {
    request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}
