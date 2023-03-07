use std::{sync::Arc, time};

use metrics::{
    MetricCache, MetricsAllRequest, MetricsAllResponse, MetricsRequest, MetricsResponse,
};
use serde::Deserialize;

// pub mod api;
pub mod metrics;

pub struct MachineMetrics {
    cache: Arc<MetricCache>,
}

impl MachineMetrics {
    pub fn spawn_metrics(config: &MachineMetricsConfig) -> Self {
        let cache = MetricCache::new(config.shard_count, config.ring_size);
        let cache = Arc::new(cache);

        start_sampling_machine_metrics(&config, &cache);

        Self { cache }
    }

    pub fn get_machine_metrics_all(&self, req: MetricsAllRequest) -> MetricsAllResponse {
        metrics::get_machine_metrics_all(req, self.cache.clone())
    }

    pub fn get_machine_metrics(&self, req: MetricsRequest) -> MetricsResponse {
        metrics::get_machine_metrics(req, self.cache.clone())
    }
}

#[derive(Debug, Deserialize)]
pub struct MachineMetricsConfig {
    pub shard_count: usize,
    pub ring_size: usize,
    pub sample_interval_s: u64,
    pub ethernet_name: String,
}

fn start_sampling_machine_metrics(config: &MachineMetricsConfig, cache: &Arc<MetricCache>) {
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
