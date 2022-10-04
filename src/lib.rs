use std::collections::HashMap;

use cncr_k_ltd_ring::CncrKLtdRing;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum MetricName {
    CpusUsage,
    CpuUsage { id: usize },
    MemUsage,
    NetTxUsage,
    NetRxUsage,
}

pub type Cache = CncrKLtdRing<MetricName, MetricPoint>;

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
