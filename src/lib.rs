use std::collections::HashMap;

use cncr_k_ltd_ring::CncrKLtdRing;
use serde::{Deserialize, Serialize};

pub mod api;
pub mod metrics;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f32,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum MetricName {
    CpusUsage,
    CpuUsage { id: usize },
    MemUsage,
    NetTxUsage,
    NetRxUsage,
}

pub type MetricCache = CncrKLtdRing<MetricName, MetricPoint>;

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsRequest {
    pub each_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsResponse {
    pub cpus: Vec<MetricPoint>,
    pub cpu: HashMap<usize, Vec<MetricPoint>>,
    pub mem: Vec<MetricPoint>,
    pub net_tx: Vec<MetricPoint>,
    pub net_rx: Vec<MetricPoint>,
}
