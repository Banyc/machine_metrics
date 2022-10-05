use std::{sync::Arc, time};

use crate::{MetricCache, MetricName, MetricPoint};
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
