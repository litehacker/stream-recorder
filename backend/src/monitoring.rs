/*
 * monitoring.rs
 * Purpose: Application monitoring and metrics collection
 * 
 * This file contains:
 * - MetricsStore for collecting application-wide metrics
 * - ResourceMonitor for tracking system resources (CPU, memory)
 * - ConnectionTracker for managing active WebSocket connections
 * - Performance metrics collection and reporting
 * - Garbage collection monitoring
 */

use std::{
    sync::atomic::{AtomicU64, AtomicI64, Ordering},
    time::{Duration, Instant},
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::{RwLock as TokioRwLock, Mutex};
use metrics::{counter, gauge, histogram};
use crate::{
    error::AppError,
    logging::log_performance_metrics,
};
use tracing::{info, warn};

// Performance metrics
static ACTIVE_CONNECTIONS: AtomicU64 = AtomicU64::new(0);
static BYTES_TRANSFERRED: AtomicU64 = AtomicU64::new(0);
static PEAK_MEMORY_USAGE: AtomicU64 = AtomicU64::new(0);
static LAST_GC_TIME: AtomicI64 = AtomicI64::new(0);

// Metrics store for collecting application metrics
#[derive(Clone)]
pub struct MetricsStore {
    requests: Arc<RwLock<HashMap<String, u64>>>,
    errors: Arc<RwLock<HashMap<String, u64>>>,
    latencies: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl MetricsStore {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            errors: Arc::new(RwLock::new(HashMap::new())),
            latencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_request(&self, endpoint: &str) {
        let mut requests = self.requests.write().unwrap();
        *requests.entry(endpoint.to_string()).or_insert(0) += 1;
    }

    pub async fn record_error(&self, endpoint: &str) {
        let mut errors = self.errors.write().unwrap();
        *errors.entry(endpoint.to_string()).or_insert(0) += 1;
    }

    pub async fn record_latency(&self, endpoint: &str, latency: f64) {
        let mut latencies = self.latencies.write().unwrap();
        latencies.entry(endpoint.to_string())
            .or_insert_with(Vec::new)
            .push(latency);
    }

    pub async fn get_total_requests(&self) -> u64 {
        let requests = self.requests.read().unwrap();
        requests.values().sum()
    }

    pub async fn get_error_rate(&self) -> f64 {
        let requests = self.requests.read().unwrap();
        let errors = self.errors.read().unwrap();
        
        let total_requests: u64 = requests.values().sum();
        let total_errors: u64 = errors.values().sum();
        
        if total_requests == 0 {
            0.0
        } else {
            (total_errors as f64 / total_requests as f64) * 100.0
        }
    }

    pub async fn get_avg_latency(&self) -> f64 {
        let latencies = self.latencies.read().unwrap();
        let mut total = 0.0;
        let mut count = 0;
        
        for values in latencies.values() {
            total += values.iter().sum::<f64>();
            count += values.len();
        }
        
        if count == 0 {
            0.0
        } else {
            total / count as f64
        }
    }

    pub fn record_bytes(&self, room_id: String, bytes: u64) {
        let mut requests = self.requests.write().unwrap();
        *requests.entry(format!("bytes_{}", room_id)).or_insert(0) += bytes;
    }

    pub fn record_frames(&self, room_id: String, frames: u64) {
        let mut requests = self.requests.write().unwrap();
        *requests.entry(format!("frames_{}", room_id)).or_insert(0) += frames;
    }

    pub fn record_errors(&self, room_id: String, count: u64) {
        let mut errors = self.errors.write().unwrap();
        *errors.entry(room_id).or_insert(0) += count;
    }

    pub async fn get_room_metrics(&self, room_id: &str) -> Result<RoomMetrics, AppError> {
        let requests = self.requests.read().unwrap();
        let errors = self.errors.read().unwrap();
        let latencies = self.latencies.read().unwrap();

        Ok(RoomMetrics {
            bytes_transferred: *requests.get(&format!("bytes_{}", room_id)).unwrap_or(&0),
            frames_processed: *requests.get(&format!("frames_{}", room_id)).unwrap_or(&0),
            error_rate: errors.get(room_id).copied().unwrap_or(0) as f64,
            avg_latency: latencies.get(room_id)
                .map(|v| v.iter().sum::<f64>() / v.len() as f64)
                .unwrap_or(0.0),
        })
    }

    pub async fn get_user_metrics(&self) -> Result<UserMetrics, AppError> {
        let requests = self.requests.read().unwrap();
        
        Ok(UserMetrics {
            total_rooms: requests.keys()
                .filter(|k| k.starts_with("frames_"))
                .count() as u64,
            total_storage: requests.iter()
                .filter(|(k, _)| k.starts_with("bytes_"))
                .map(|(_, &v)| v)
                .sum(),
            total_bandwidth: requests.iter()
                .filter(|(k, _)| k.starts_with("bytes_"))
                .map(|(_, &v)| v)
                .sum(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct RoomMetrics {
    pub bytes_transferred: u64,
    pub frames_processed: u64,
    pub error_rate: f64,
    pub avg_latency: f64,
}

#[derive(Debug, Clone)]
pub struct UserMetrics {
    pub total_rooms: u64,
    pub total_storage: u64,
    pub total_bandwidth: u64,
}

// Resource monitor for tracking system resources
#[derive(Clone)]
pub struct ResourceMonitor {
    memory_threshold: Arc<RwLock<u64>>,
    cpu_threshold: Arc<RwLock<f64>>,
    memory_usage: Arc<RwLock<u64>>,
    cpu_usage: Arc<RwLock<f64>>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            memory_threshold: Arc::new(RwLock::new(90)),  // 90% threshold
            cpu_threshold: Arc::new(RwLock::new(80.0)),   // 80% threshold
            memory_usage: Arc::new(RwLock::new(0)),
            cpu_usage: Arc::new(RwLock::new(0.0)),
        }
    }

    pub async fn update_metrics(&self) {
        // Update memory usage
        if let Ok(memory) = sys_info::mem_info() {
            let total = memory.total;
            let free = memory.free;
            let used = total - free;
            let usage_percent = (used as f64 / total as f64) * 100.0;
            *self.memory_usage.write().unwrap() = usage_percent as u64;
        }

        // Update CPU usage
        if let Ok(cpu) = sys_info::loadavg() {
            *self.cpu_usage.write().unwrap() = cpu.one;
        }
    }

    pub async fn is_memory_critical(&self) -> bool {
        let usage = *self.memory_usage.read().unwrap();
        let threshold = *self.memory_threshold.read().unwrap();
        usage > threshold
    }

    pub async fn is_cpu_critical(&self) -> bool {
        let usage = *self.cpu_usage.read().unwrap();
        let threshold = *self.cpu_threshold.read().unwrap();
        usage > threshold
    }

    pub async fn get_memory_usage(&self) -> u64 {
        *self.memory_usage.read().unwrap()
    }

    pub async fn get_cpu_usage(&self) -> f64 {
        *self.cpu_usage.read().unwrap()
    }
}

// Connection tracker for managing active connections
#[derive(Clone)]
pub struct ConnectionTracker {
    connections: Arc<RwLock<HashMap<String, u32>>>,
    max_connections: Arc<RwLock<u32>>,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections: Arc::new(RwLock::new(100)), // Default max connections
        }
    }

    pub fn with_max_connections(max: u32) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections: Arc::new(RwLock::new(max)),
        }
    }

    pub async fn check_limits(&self, room_id: &str) -> bool {
        let connections = self.connections.read().unwrap();
        let count = connections.get(room_id).copied().unwrap_or(0);
        let max = *self.max_connections.read().unwrap();
        
        count < max
    }

    pub async fn add_connection(&self, room_id: &str) {
        let mut connections = self.connections.write().unwrap();
        *connections.entry(room_id.to_string()).or_insert(0) += 1;
    }

    pub async fn remove_connection(&self, room_id: &str) {
        let mut connections = self.connections.write().unwrap();
        if let Some(count) = connections.get_mut(room_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub async fn get_connection_count(&self, room_id: &str) -> u32 {
        let connections = self.connections.read().unwrap();
        connections.get(room_id).copied().unwrap_or(0)
    }
}

// Garbage collection monitoring
pub fn monitor_gc() {
    use std::sync::atomic::AtomicBool;
    static GC_MONITORING: AtomicBool = AtomicBool::new(false);

    if !GC_MONITORING.swap(true, Ordering::Relaxed) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                let last_gc = LAST_GC_TIME.load(Ordering::Relaxed);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                if now - last_gc > 3600 { // 1 hour
                    // Trigger GC if needed
                    #[cfg(feature = "jemalloc")]
                    {
                        jemalloc_ctl::epoch::advance().unwrap();
                        jemalloc_ctl::purge::decay().unwrap();
                    }
                    
                    LAST_GC_TIME.store(now, Ordering::Relaxed);
                    log_performance_metrics("gc_triggered", 1.0, Some("memory"));
                }
            }
        });
    }
}

pub fn get_cpu_usage() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpu) = sys_info::cpu_num() {
            if let Ok(load) = sys_info::loadavg() {
                return load.one / cpu as f64 * 100.0;
            }
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux systems, return a dummy value
        return 50.0;
    }
    
    0.0
}