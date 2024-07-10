use anyhow::Result;
use psutil::cpu::CpuPercentCollector;
use psutil::disk::{disk_usage, DiskUsage};
use psutil::memory::virtual_memory;
use psutil::host;
use psutil::network::NetIoCountersCollector;
use std::time::Duration;

pub fn get_uptime() -> Result<u64> {
    Ok(host::boot_time()?.elapsed()?.as_secs())
}
pub fn get_load() -> Result<f64> {
    let load = host::loadavg()?;
    Ok(load.one)
}

pub async fn get_cpu() -> Result<f64> {
    let mut collector = CpuPercentCollector::new()?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(collector.cpu_percent()? as f64)
}

pub fn get_memory() -> Result<(u64, u64, u64, u64)> {
    let memory = virtual_memory()?;
    Ok((
        memory.total()/1024,
        memory.used()/1024,
        memory.total()/1024, // Assuming SwapTotal is the same as total memory
        memory.total()/1024 - memory.available()/1024, // SwapUsed
    ))
}

pub fn get_hdd() -> Result<(u64, u64)> {
    let usage: DiskUsage = disk_usage("/")?;
    Ok((usage.total()/1024/1024, usage.used()/1024/1024))
}

pub struct Network {
    collector: NetIoCountersCollector,
}

impl Network {
    pub fn new() -> Result<Self> {
        Ok(Self {
            collector: NetIoCountersCollector::default(),
        })
    }

    pub fn get_speed(&mut self) -> Result<(u64, u64)> {
        let before = self.collector.net_io_counters()?;
        std::thread::sleep(Duration::from_secs(1));
        let after = self.collector.net_io_counters()?;

        let rx = after.bytes_recv().saturating_sub(before.bytes_recv());
        let tx = after.bytes_sent().saturating_sub(before.bytes_sent());

        Ok((rx, tx))
    }

    pub fn get_traffic(&mut self) -> Result<(u64, u64)> {
        let io = self.collector.net_io_counters()?;
        Ok((io.bytes_recv(), io.bytes_sent()))
    }
}

pub async fn get_network(ip_version: u8) -> Result<bool> {
    let host = match ip_version {
        4 => "ipv4.google.com",
        6 => "ipv6.google.com",
        _ => return Ok(false),
    };

    let client = reqwest::Client::new();
    let response = client.get(format!("http://{}", host))
        .timeout(Duration::from_secs(2))
        .send()
        .await;

    Ok(response.is_ok())
}