use std::{collections::HashSet, sync::Arc};

use crate::{MetricCache, MetricsRequest};
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};

pub async fn api_token_auth<B>(
    req: Request<B>,
    next: Next<B>,
    api_tokens: Arc<HashSet<String>>,
) -> impl IntoResponse {
    let auth_header = req.headers().get("Authorization");
    if let Some(auth_header) = auth_header {
        if let Ok(auth_header) = auth_header.to_str() {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                if api_tokens.contains(token) {
                    return next.run(req).await;
                }
            }
        }
    }
    (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

pub async fn get_machine_metrics(
    req: MetricsRequest,
    cache: Arc<MetricCache>,
) -> impl IntoResponse {
    let resp = crate::metrics::get_machine_metrics(req, cache);
    Json(resp)
}
