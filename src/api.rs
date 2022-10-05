use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{MetricCache, MetricName, MetricsRequest, MetricsResponse};
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
    let mut resp = MetricsResponse {
        cpu: HashMap::new(),
        cpus: cache
            .clone_batch_last(&MetricName::CpusUsage, req.each_count)
            .map_or(vec![], |v| v),
        mem: cache
            .clone_batch_last(&MetricName::MemUsage, req.each_count)
            .map_or(vec![], |v| v),
        net_tx: cache
            .clone_batch_last(&MetricName::NetTxUsage, req.each_count)
            .map_or(vec![], |v| v),
        net_rx: cache
            .clone_batch_last(&MetricName::NetRxUsage, req.each_count)
            .map_or(vec![], |v| v),
    };

    let mut id = 0;
    loop {
        let metrics = match cache.clone_batch_last(&MetricName::CpuUsage { id }, req.each_count) {
            Some(metrics) => metrics,
            None => break,
        };
        resp.cpu.insert(id, metrics);
        id = id + 1;
    }

    Json(resp)
}
