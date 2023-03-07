use std::{collections::HashMap, sync::Arc, time};

use cncr_k_ltd_ring::CncrKLtdRing;
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, NetworkExt, System, SystemExt};

pub fn get_new_sys_info() -> System {
    let mut sys_info = System::new_all();
    sys_info.refresh_all();
    sys_info
}

pub fn sample_sys_info(
    cache: &Arc<MetricCache>,
    sys_info: &mut System,
    ethernet_interface_name: &str,
) {
    sys_info.refresh_cpu();
    sys_info.refresh_memory();
    sys_info.refresh_networks();

    let timestamp = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cpus_usage = sys_info.global_cpu_info().cpu_usage() / 100.0;
    cache.push(
        MetricName::CpusUsage,
        MetricPoint {
            timestamp,
            value: cpus_usage as f32,
        },
    );

    for (i, cpu) in sys_info.cpus().iter().enumerate() {
        let usage = cpu.cpu_usage() / 100.0;
        cache.push(
            MetricName::CpuUsage { id: i },
            MetricPoint {
                timestamp,
                value: usage as f32,
            },
        );
    }

    let mem_usage = sys_info.used_memory() as f64 / sys_info.total_memory() as f64;
    cache.push(
        MetricName::MemUsage,
        MetricPoint {
            timestamp,
            value: mem_usage as f32,
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
                value: tx_bytes as f32,
            },
        );

        let rx_bytes = data.received();
        cache.push(
            MetricName::NetRxUsage,
            MetricPoint {
                timestamp,
                value: rx_bytes as f32,
            },
        );
        break;
    }
}

pub fn get_machine_metrics_all(
    req: MetricsAllRequest,
    cache: Arc<MetricCache>,
) -> MetricsAllResponse {
    let mut resp = MetricsAllResponse {
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

    resp
}

pub fn get_machine_metrics(req: MetricsRequest, cache: Arc<MetricCache>) -> MetricsResponse {
    let mut data = Vec::new();

    for item in req.0 {
        let metrics = cache
            .clone_batch_last(&item.name, item.count)
            .map_or(vec![], |v| v);
        data.push(metrics);
    }

    MetricsResponse(data)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f32,
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricName {
    CpusUsage,
    CpuUsage { id: usize },
    MemUsage,
    NetTxUsage,
    NetRxUsage,
}

pub type MetricCache = CncrKLtdRing<MetricName, MetricPoint>;

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsAllRequest {
    pub each_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsAllResponse {
    pub cpus: Vec<MetricPoint>,
    pub cpu: HashMap<usize, Vec<MetricPoint>>,
    pub mem: Vec<MetricPoint>,
    pub net_tx: Vec<MetricPoint>,
    pub net_rx: Vec<MetricPoint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsRequest(pub Vec<MetricsRequestItem>);

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsRequestItem {
    pub count: usize,
    pub name: MetricName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsResponse(pub Vec<Vec<MetricPoint>>);
