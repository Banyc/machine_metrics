use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
    time,
};

use axum::{middleware, routing::post, Json, Router};
use log::info;
use machine_metrics::{api, metrics, MetricCache};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();

    let config = get_config();

    let cache = MetricCache::new(config.shard_count, config.ring_size);
    let cache = Arc::new(cache);

    let api_tokens = HashSet::from_iter(config.api_tokens.iter().map(|x| x.token.clone()));
    let api_tokens = Arc::new(api_tokens);

    start_sampling_machine_metrics(&config, &cache);

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
    let addr: SocketAddr = config
        .listen_addr
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn get_config() -> Config {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/config.toml".to_string());
    let mut config_file = File::open(config_path).unwrap();
    let mut config = String::new();
    config_file.read_to_string(&mut config).unwrap();
    toml::from_str(&config).unwrap()
}

fn start_sampling_machine_metrics(config: &Config, cache: &Arc<MetricCache>) {
    let mut sys_info = metrics::get_new_sys_info();

    let ethernet_interface_name = config.ethernet_name.clone();
    let sample_interval_s = config.sample_interval_s;
    let cache = Arc::clone(&cache);

    tokio::spawn(async move {
        loop {
            metrics::sample_sys_info(&cache, &mut sys_info, &ethernet_interface_name);
            tokio::time::sleep(time::Duration::from_secs(sample_interval_s)).await;
        }
    });
}

#[derive(Debug, Deserialize)]
pub struct ApiToken {
    pub token: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: String,
    pub shard_count: usize,
    pub ring_size: usize,
    pub sample_interval_s: u64,
    pub ethernet_name: String,
    pub api_tokens: Vec<ApiToken>,
}
