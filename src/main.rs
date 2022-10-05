use std::{collections::HashSet, net::SocketAddr, sync::Arc, time};

use axum::{middleware, routing::post, Json, Router};
use log::info;
use machine_metrics::{api, metrics, MetricCache};

#[tokio::main]
async fn main() {
    env_logger::try_init().unwrap();

    let shard_count = 4;
    let ring_size = 1024;
    let cache = MetricCache::new(shard_count, ring_size);
    let cache = Arc::new(cache);

    let ethernet_name = "en0";
    let api_tokens = vec!["unsafe".to_string()];

    let api_tokens = HashSet::from_iter(api_tokens);
    let api_tokens = Arc::new(api_tokens);

    let mut sys_info = metrics::get_new_sys_info();
    {
        let cache = Arc::clone(&cache);
        tokio::spawn(async move {
            loop {
                metrics::sample_sys_info(&cache, &mut sys_info, &ethernet_name);
                tokio::time::sleep(time::Duration::from_secs(5)).await;
            }
        });
    }

    let api_token_required = Router::new()
        .route(
            "/get_machine_metrics/v1",
            post(move |Json(req)| api::get_machine_metrics(req, Arc::clone(&cache))),
        )
        .layer(middleware::from_fn(move |req, next| {
            api::api_token_auth(req, next, Arc::clone(&api_tokens))
        }));

    // build our application
    let app = Router::new().nest("/api_token", api_token_required);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
