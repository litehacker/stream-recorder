use std::{
    sync::atomic::{AtomicU64, AtomicI64, Ordering},
    time::{Duration, Instant},
    collections::HashMap,
};
use tokio::sync::RwLock;
use metrics::{counter, gauge, histogram};
use crate::logging::log_performance_metrics;

// Performance metrics
static ACTIVE_CONNECTIONS: AtomicU64 = AtomicU64::new(0);
static BYTES_TRANSFERRED: AtomicU64 = AtomicU64::new(0);
static PEAK_MEMORY_USAGE: AtomicU64 = AtomicU64::new(0);
static LAST_GC_TIME: AtomicI64 = AtomicI64::new(0);

// Real-time metrics storage
pub struct MetricsStore {
    latencies: RwLock<HashMap<String, Vec<f64>>>,
    error_rates: RwLock<HashMap<String, (u64, u64)>>, // (errors, total)
    throughput: RwLock<HashMap<String, Vec<(i64, u64)>>>, // (timestamp, count)
}

impl MetricsStore {
    pub fn new() -> Self {
        Self {
            latencies: RwLock::new(HashMap::new()),
            error_rates: RwLock::new(HashMap::new()),
            throughput: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record_latency(&self, endpoint: &str, duration: f64) {
        let mut latencies = self.latencies.write().await;
        latencies
            .entry(endpoint.to_string())
            .or_insert_with(Vec::new)
            .push(duration);

        // Record to metrics system
        histogram!("request_duration_seconds", duration, "endpoint" => endpoint.to_string());
    }

    pub async fn record_error(&self, endpoint: &str) {
        let mut error_rates = self.error_rates.write().await;
        let (errors, total) = error_rates
            .entry(endpoint.to_string())
            .or_insert((0, 0));
        *errors += 1;
        *total += 1;

        // Record to metrics system
        counter!("request_errors_total", 1, "endpoint" => endpoint.to_string());
    }

    pub async fn record_request(&self, endpoint: &str) {
        let timestamp = Instant::now()
            .duration_since(Instant::from_std(Duration::ZERO))
            .as_secs() as i64;

        let mut throughput = self.throughput.write().await;
        let entries = throughput
            .entry(endpoint.to_string())
            .or_insert_with(Vec::new);
        
        entries.push((timestamp, 1));
        
        // Cleanup old entries (keep last hour)
        let cutoff = timestamp - 3600;
        entries.retain(|(ts, _)| *ts > cutoff);

        // Record to metrics system
        counter!("requests_total", 1, "endpoint" => endpoint.to_string());
    }

    pub async fn get_error_rate(&self, endpoint: &str) -> f64 {
        let error_rates = self.error_rates.read().await;
        if let Some((errors, total)) = error_rates.get(endpoint) {
            if *total > 0 {
                return *errors as f64 / *total as f64;
            }
        }
        0.0
    }

    pub async fn get_average_latency(&self, endpoint: &str) -> f64 {
        let latencies = self.latencies.read().await;
        if let Some(values) = latencies.get(endpoint) {
            if !values.is_empty() {
                return values.iter().sum::<f64>() / values.len() as f64;
            }
        }
        0.0
    }

    pub async fn get_requests_per_second(&self, endpoint: &str) -> f64 {
        let throughput = self.throughput.read().await;
        if let Some(entries) = throughput.get(endpoint) {
            if entries.len() < 2 {
                return 0.0;
            }
            
            let now = Instant::now()
                .duration_since(Instant::from_std(Duration::ZERO))
                .as_secs() as i64;
            let window = 60; // 1 minute window
            let cutoff = now - window;
            
            let recent_requests: u64 = entries
                .iter()
                .filter(|(ts, _)| *ts > cutoff)
                .map(|(_, count)| count)
                .sum();
            
            return recent_requests as f64 / window as f64;
        }
        0.0
    }
}

// Resource monitoring
pub struct ResourceMonitor {
    memory_threshold: u64,
    cpu_threshold: f64,
}

impl ResourceMonitor {
    pub fn new(memory_threshold_mb: u64, cpu_threshold_percent: f64) -> Self {
        Self {
            memory_threshold: memory_threshold_mb * 1024 * 1024,
            cpu_threshold: cpu_threshold_percent,
        }
    }

    pub fn check_resources(&self) -> bool {
        let memory_ok = self.check_memory();
        let cpu_ok = self.check_cpu();

        // Record metrics
        gauge!("memory_usage_bytes", self.get_memory_usage() as f64);
        gauge!("cpu_usage_percent", self.get_cpu_usage());

        memory_ok && cpu_ok
    }

    fn check_memory(&self) -> bool {
        let usage = self.get_memory_usage();
        PEAK_MEMORY_USAGE.fetch_max(usage, Ordering::Relaxed);
        
        if usage > self.memory_threshold {
            log_performance_metrics(
                "memory_threshold_exceeded",
                usage as f64 / (1024.0 * 1024.0),
                Some("memory")
            );
            false
        } else {
            true
        }
    }

    fn check_cpu(&self) -> bool {
        let usage = self.get_cpu_usage();
        if usage > self.cpu_threshold {
            log_performance_metrics(
                "cpu_threshold_exceeded",
                usage,
                Some("cpu")
            );
            false
        } else {
            true
        }
    }

    fn get_memory_usage(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::Read;
            let mut status = String::new();
            if File::open("/proc/self/status")
                .and_then(|mut f| f.read_to_string(&mut status))
                .is_ok()
            {
                if let Some(line) = status.lines().find(|l| l.starts_with("VmRSS:")) {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb.parse::<u64>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }
        0
    }

    fn get_cpu_usage(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::Read;
            let mut stat = String::new();
            if File::open("/proc/self/stat")
                .and_then(|mut f| f.read_to_string(&mut stat))
                .is_ok()
            {
                let fields: Vec<&str> = stat.split_whitespace().collect();
                if fields.len() > 13 {
                    if let (Ok(utime), Ok(stime)) = (
                        fields[13].parse::<u64>(),
                        fields[14].parse::<u64>()
                    ) {
                        let total_time = utime + stime;
                        let seconds = total_time as f64 / 100.0; // Convert jiffies to seconds
                        return seconds * 100.0; // Convert to percentage
                    }
                }
            }
        }
        0.0
    }
}

// Connection tracking
pub struct ConnectionTracker {
    connection_limit: u64,
}

impl ConnectionTracker {
    pub fn new(connection_limit: u64) -> Self {
        Self { connection_limit }
    }

    pub fn track_connection(&self) -> bool {
        let current = ACTIVE_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
        
        // Record metrics
        gauge!("active_connections", current as f64 + 1.0);
        
        if current >= self.connection_limit {
            ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
            false
        } else {
            true
        }
    }

    pub fn release_connection(&self) {
        ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
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
                let now = Instant::now()
                    .duration_since(Instant::from_std(Duration::ZERO))
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