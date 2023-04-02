pub mod api;

use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    net::{SocketAddr, ToSocketAddrs},
    sync::Arc,
};

use axum::{middleware, routing::post, Router};
use log::info;
use machine_metrics::{MachineMetrics, MachineMetricsConfig};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();

    let config = get_config();

    let api_tokens = HashSet::from_iter(config.api_tokens.iter().map(|x| x.token.clone()));
    let api_tokens = Arc::new(api_tokens);

    let machine_metrics = MachineMetrics::spawn_metrics(&config.machine_metrics);
    let machine_metrics = Arc::new(machine_metrics);

    let api_token_required = Router::new()
        .route(
            "/get_machine_metrics_all/v1",
            post({
                let machine_metrics = Arc::clone(&machine_metrics);
                move |req| api::get_machine_metrics_all(req, machine_metrics)
            }),
        )
        .route(
            "/get_machine_metrics/v1",
            post({
                let machine_metrics = Arc::clone(&machine_metrics);
                move |req| api::get_machine_metrics(req, machine_metrics)
            }),
        )
        .layer(middleware::from_fn(move |req, next| {
            api::api_token_auth(req, next, Arc::clone(&api_tokens))
        }))
        .layer(
            CorsLayer::new()
                .allow_headers(Any)
                .allow_methods(Any)
                .allow_origin(Any),
        );

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

#[derive(Debug, Deserialize)]
pub struct ApiToken {
    pub token: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: String,
    pub machine_metrics: MachineMetricsConfig,
    pub api_tokens: Vec<ApiToken>,
}
