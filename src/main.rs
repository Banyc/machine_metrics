use std::{collections::HashMap, net::SocketAddr, sync::Arc, time};

use axum::{response::IntoResponse, routing::post, Json, Router};
use log::info;
use machine_metrics::{Cache, MetricName, MetricPoint, MetricsRequest, MetricsResponse};
use sysinfo::{CpuExt, NetworkExt, System, SystemExt};

#[tokio::main]
async fn main() {
    env_logger::try_init().unwrap();

    let shard_count = 4;
    let ring_size = 1024;
    let cache = Cache::new(shard_count, ring_size);
    let cache = Arc::new(cache);

    let mut sys_info = get_new_sys_info();

    let ethernet_name = "en0";

    {
        let cache = Arc::clone(&cache);
        tokio::spawn(async move {
            loop {
                sample_sys_info(&cache, &mut sys_info, &ethernet_name);
                tokio::time::sleep(time::Duration::from_secs(5)).await;
            }
        });
    }

    // build our application with a route
    let app = Router::new().route(
        "/api/v1/machine_metrics",
        post({
            let cache = Arc::clone(&cache);
            move |Json(req)| get_machine_metrics(req, Arc::clone(&cache))
        }),
    );

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_machine_metrics(req: MetricsRequest, cache: Arc<Cache>) -> impl IntoResponse {
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

fn get_new_sys_info() -> System {
    let mut sys_info = System::new_all();
    sys_info.refresh_all();
    sys_info
}

fn sample_sys_info(cache: &Arc<Cache>, sys_info: &mut System, ethernet_interface_name: &str) {
    sys_info.refresh_cpu();
    sys_info.refresh_memory();
    sys_info.refresh_networks();

    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cpus_usage = sys_info.global_cpu_info().cpu_usage();
    cache.push(
        MetricName::CpusUsage,
        MetricPoint {
            timestamp,
            value: cpus_usage as f64,
        },
    );

    for (i, cpu) in sys_info.cpus().iter().enumerate() {
        let usage = cpu.cpu_usage();
        cache.push(
            MetricName::CpuUsage { id: i },
            MetricPoint {
                timestamp,
                value: usage as f64,
            },
        );
    }

    let mem_usage = sys_info.used_memory() as f64 / sys_info.total_memory() as f64;
    cache.push(
        MetricName::MemUsage,
        MetricPoint {
            timestamp,
            value: mem_usage,
        },
    );

    for (interface_name, data) in sys_info.networks() {
        if interface_name != ethernet_interface_name {
            continue;
        }
        let tx_bytes = data.transmitted();
        cache.push(
            MetricName::NetTxUsage,
            MetricPoint {
                timestamp,
                value: tx_bytes as f64,
            },
        );

        let rx_bytes = data.received();
        cache.push(
            MetricName::NetRxUsage,
            MetricPoint {
                timestamp,
                value: rx_bytes as f64,
            },
        );
        break;
    }
}
