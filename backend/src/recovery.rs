use std::{
    sync::atomic::{AtomicU32, AtomicI64, Ordering},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use crate::{
    error::{AppError, AppResult},
    logging::{log_performance_metrics, log_error_with_context},
};

// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

// Circuit breaker for external services
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: AtomicI64,
    state: std::sync::atomic::AtomicU8,
    threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout_secs: u64) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            last_failure: AtomicI64::new(0),
            state: std::sync::atomic::AtomicU8::new(CircuitState::Closed as u8),
            threshold,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub async fn call<F, T>(&self, operation: F, context: &str) -> AppResult<T>
    where
        F: Future<Output = AppResult<T>>,
    {
        let start = Instant::now();
        let state = CircuitState::from(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Open => {
                let last = self.last_failure.load(Ordering::Relaxed);
                let elapsed = Instant::now()
                    .duration_since(Instant::from_std(Duration::from_secs(last as u64)));
                
                if elapsed < self.timeout {
                    return Err(AppError::ServiceUnavailable(
                        format!("Circuit breaker open for {}", context)
                    ));
                }
                self.state.store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // Allow only one request to test the service
                if self.failure_count.load(Ordering::Relaxed) > 0 {
                    return Err(AppError::ServiceUnavailable(
                        format!("Circuit breaker half-open for {}", context)
                    ));
                }
            }
            CircuitState::Closed => {}
        }

        match operation.await {
            Ok(result) => {
                // Reset on success
                if state == CircuitState::HalfOpen {
                    self.state.store(CircuitState::Closed as u8, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                }
                
                // Log success metrics
                log_performance_metrics(
                    &format!("{}_success_time", context),
                    start.elapsed().as_secs_f64(),
                    None
                );
                
                Ok(result)
            }
            Err(error) => {
                self.record_failure(context);
                Err(error)
            }
        }
    }

    fn record_failure(&self, context: &str) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.last_failure.store(
            Instant::now().duration_since(Instant::from_std(Duration::ZERO)).as_secs() as i64,
            Ordering::Relaxed
        );

        if count >= self.threshold {
            self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
            log_error_with_context(
                "Circuit breaker opened",
                context,
                Some(&serde_json::json!({
                    "failures": count,
                    "threshold": self.threshold
                }))
            );
        }
    }
}

// Retry strategy with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    operation: F,
    max_retries: u32,
    initial_delay: Duration,
    context: &str,
) -> Result<T, E>
where
    F: Fn() -> Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut retries = 0;
    let mut delay = initial_delay;

    loop {
        match operation().await {
            Ok(result) => {
                if retries > 0 {
                    log_performance_metrics(
                        &format!("{}_retry_success", context),
                        retries as f64,
                        None
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                retries += 1;
                if retries >= max_retries {
                    log_error_with_context(
                        "Max retries exceeded",
                        context,
                        Some(&serde_json::json!({
                            "retries": retries,
                            "max_retries": max_retries,
                            "error": format!("{:?}", error)
                        }))
                    );
                    return Err(error);
                }

                log_warning!(
                    &format!("Retry attempt {} for {}", retries, context),
                    &format!("{:?}", error)
                );

                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

// Health check with recovery
pub struct HealthCheck {
    last_check: AtomicI64,
    healthy: std::sync::atomic::AtomicBool,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self {
            last_check: AtomicI64::new(0),
            healthy: std::sync::atomic::AtomicBool::new(true),
        }
    }

    pub async fn check_health<F>(&self, check: F, service: &str) -> bool
    where
        F: Future<Output = bool>,
    {
        let now = Instant::now().duration_since(Instant::from_std(Duration::ZERO)).as_secs() as i64;
        self.last_check.store(now, Ordering::Relaxed);

        let result = check.await;
        self.healthy.store(result, Ordering::Relaxed);

        log_performance_metrics(
            &format!("{}_health", service),
            if result { 1.0 } else { 0.0 },
            None
        );

        result
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed)
    }

    pub fn last_check_time(&self) -> i64 {
        self.last_check.load(Ordering::Relaxed)
    }
}

// Memory leak detection
pub struct MemoryMonitor {
    last_usage: AtomicU64,
    threshold: u64,
}

impl MemoryMonitor {
    pub fn new(threshold_mb: u64) -> Self {
        Self {
            last_usage: AtomicU64::new(0),
            threshold: threshold_mb * 1024 * 1024,
        }
    }

    pub fn check_memory(&self) -> bool {
        let usage = self.get_memory_usage();
        self.last_usage.store(usage, Ordering::Relaxed);

        if usage > self.threshold {
            log_error_with_context(
                "Memory usage exceeded threshold",
                "memory_monitor",
                Some(&serde_json::json!({
                    "usage_mb": usage / (1024 * 1024),
                    "threshold_mb": self.threshold / (1024 * 1024)
                }))
            );
            false
        } else {
            true
        }
    }

    fn get_memory_usage(&self) -> u64 {
        // Platform-specific memory usage implementation
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
} 