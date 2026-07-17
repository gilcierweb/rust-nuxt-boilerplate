use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Histogram for P95/P99 latency tracking
// ---------------------------------------------------------------------------

/// Default histogram bucket boundaries in milliseconds, covering 1ms to 10s.
const LATENCY_BUCKETS_MS: &[f64] = &[
    1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
];

/// A simple cumulative histogram for latency measurements.
#[derive(Debug, Clone)]
struct DurationHistogram {
    count: u64,
    sum_ms: f64,
    /// Cumulative bucket counts: buckets[i] = number of observations <= LATENCY_BUCKETS_MS[i].
    buckets: Vec<u64>,
    /// All raw observations, capped at MAX_SAMPLES, for exact percentile calculation.
    samples_ms: Vec<f64>,
}

impl Default for DurationHistogram {
    fn default() -> Self {
        Self::new()
    }
}

const MAX_SAMPLES: usize = 1024;

impl DurationHistogram {
    fn new() -> Self {
        Self {
            count: 0,
            sum_ms: 0.0,
            buckets: vec![0; LATENCY_BUCKETS_MS.len()],
            samples_ms: Vec::with_capacity(MAX_SAMPLES),
        }
    }

    fn observe(&mut self, duration: Duration) {
        let ms = duration.as_secs_f64() * 1000.0;
        self.count += 1;
        self.sum_ms += ms;

        for (i, &bucket_upper) in LATENCY_BUCKETS_MS.iter().enumerate() {
            if ms <= bucket_upper {
                self.buckets[i] += 1;
            }
        }
        // +Inf bucket is implicit (count - last bucket).

        if self.samples_ms.len() < MAX_SAMPLES {
            self.samples_ms.push(ms);
        } else {
            // Ring-buffer: overwrite oldest sample so we get rolling window.
            let idx = (self.count as usize) % MAX_SAMPLES;
            self.samples_ms[idx] = ms;
        }
    }

    /// Compute exact P95 and P99 from stored samples.
    fn percentile(&self, p: f64) -> Option<f64> {
        if self.samples_ms.is_empty() {
            return None;
        }
        let mut sorted = self.samples_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let rank = ((p / 100.0) * (sorted.len() as f64 - 1.0)).ceil() as usize;
        let idx = rank.min(sorted.len() - 1);
        Some(sorted[idx])
    }

    fn p95(&self) -> Option<f64> {
        self.percentile(95.0)
    }

    fn p99(&self) -> Option<f64> {
        self.percentile(99.0)
    }
}

// ---------------------------------------------------------------------------
// Per-route metric entry (existing fields + histogram)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct RouteMetric {
    requests: u64,
    duration_ms_sum: f64,
    histogram: DurationHistogram,
}

impl RouteMetric {
    fn new() -> Self {
        Self {
            histogram: DurationHistogram::new(),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Disposable probe timing
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct ProbeMetrics {
    count: u64,
    sum_ms: f64,
    histogram: DurationHistogram,
}

impl ProbeMetrics {
    fn avg(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum_ms / self.count as f64)
        }
    }

    fn observe(&mut self, duration: Duration) {
        self.count += 1;
        self.sum_ms += duration.as_secs_f64() * 1000.0;
        self.histogram.observe(duration);
    }
}

// ---------------------------------------------------------------------------
// Cold start tracking
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct ColdStartMetric {
    duration_ms: f64,
    recorded: bool,
}

// ---------------------------------------------------------------------------
// System resource measures (memory / CPU)
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct SystemMeasures {
    used_memory_bytes: u64,
    total_memory_bytes: u64,
    cpu_usage_percent: f32,
}

// ---------------------------------------------------------------------------
// Metrics registry
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct MetricsRegistry {
    // HTTP request counters (existing)
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    per_status: Mutex<HashMap<u16, u64>>,
    per_route_status: Mutex<HashMap<(String, String, u16), RouteMetric>>,

    // Performance baselines
    cold_start: Mutex<ColdStartMetric>,
    db_query_timing: Mutex<ProbeMetrics>,
    redis_op_timing: Mutex<ProbeMetrics>,
    system_measures: Mutex<SystemMeasures>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // ---- Existing HTTP metric recording (now with histogram) ----

    pub fn record(&self, method: &str, path: &str, status: u16, duration: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if status >= 500 {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut per_status) = self.per_status.lock() {
            *per_status.entry(status).or_insert(0) += 1;
        }

        if let Ok(mut per_route_status) = self.per_route_status.lock() {
            let entry = per_route_status
                .entry((method.to_owned(), normalize_path(path), status))
                .or_insert_with(RouteMetric::new);
            entry.requests += 1;
            entry.duration_ms_sum += duration.as_secs_f64() * 1000.0;
            entry.histogram.observe(duration);
        }
    }

    // ---- Cold start ----

    /// Record the server cold-start duration. Should be called once during boot.
    pub fn record_cold_start(&self, duration: Duration) {
        if let Ok(mut cs) = self.cold_start.lock() {
            cs.duration_ms = duration.as_secs_f64() * 1000.0;
            cs.recorded = true;
        }
        tracing::info!(
            cold_start_ms = format!("{:.3}", duration.as_secs_f64() * 1000.0),
            "Server cold start"
        );
    }

    pub fn cold_start_ms(&self) -> Option<f64> {
        self.cold_start
            .lock()
            .ok()
            .filter(|cs| cs.recorded)
            .map(|cs| cs.duration_ms)
    }

    // ---- DB query timing ----

    /// Record a single DB query (or transaction) duration.
    pub fn record_db_query(&self, duration: Duration) {
        if let Ok(mut timing) = self.db_query_timing.lock() {
            timing.observe(duration);
        }
    }

    pub fn db_query_p95_ms(&self) -> Option<f64> {
        self.db_query_timing
            .lock()
            .ok()
            .and_then(|t| t.histogram.p95())
    }

    pub fn db_query_p99_ms(&self) -> Option<f64> {
        self.db_query_timing
            .lock()
            .ok()
            .and_then(|t| t.histogram.p99())
    }

    pub fn db_query_avg_ms(&self) -> Option<f64> {
        self.db_query_timing.lock().ok().and_then(|t| t.avg())
    }

    pub fn db_query_count(&self) -> u64 {
        self.db_query_timing.lock().map(|t| t.count).unwrap_or(0)
    }

    // ---- Redis op timing ----

    /// Record a single Redis operation duration.
    pub fn record_redis_op(&self, duration: Duration) {
        if let Ok(mut timing) = self.redis_op_timing.lock() {
            timing.observe(duration);
        }
    }

    pub fn redis_p95_ms(&self) -> Option<f64> {
        self.redis_op_timing
            .lock()
            .ok()
            .and_then(|t| t.histogram.p95())
    }

    pub fn redis_p99_ms(&self) -> Option<f64> {
        self.redis_op_timing
            .lock()
            .ok()
            .and_then(|t| t.histogram.p99())
    }

    pub fn redis_avg_ms(&self) -> Option<f64> {
        self.redis_op_timing.lock().ok().and_then(|t| t.avg())
    }

    pub fn redis_count(&self) -> u64 {
        self.redis_op_timing.lock().map(|t| t.count).unwrap_or(0)
    }

    // ---- System resources (memory / CPU) ----

    /// Snapshot current system resource usage. Called by the metrics controller
    /// or a periodic background sampler.
    pub fn refresh_system_measures(&self) {
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu_all();
        let used = sys.used_memory();
        let total = sys.total_memory();

        let cpus = sys.cpus();
        let cpu_usage = if !cpus.is_empty() {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        } else {
            0.0
        };

        if let Ok(mut measures) = self.system_measures.lock() {
            measures.used_memory_bytes = used;
            measures.total_memory_bytes = total;
            measures.cpu_usage_percent = cpu_usage;
        }
    }

    pub fn used_memory_bytes(&self) -> u64 {
        self.system_measures
            .lock()
            .map(|m| m.used_memory_bytes)
            .unwrap_or(0)
    }

    pub fn total_memory_bytes(&self) -> u64 {
        self.system_measures
            .lock()
            .map(|m| m.total_memory_bytes)
            .unwrap_or(0)
    }

    pub fn cpu_usage_percent(&self) -> f32 {
        self.system_measures
            .lock()
            .map(|m| m.cpu_usage_percent)
            .unwrap_or(0.0)
    }

    // ---- Prometheus render (now with histogram, baselines, system metrics) ----

    pub fn render_prometheus(&self) -> String {
        let mut output = String::with_capacity(4096);

        // --- HTTP counters (existing) ---
        output.push_str("# HELP http_requests_total Total HTTP requests processed.\n");
        output.push_str("# TYPE http_requests_total counter\n");
        output.push_str(&format!(
            "http_requests_total {}\n",
            self.total_requests.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP http_request_errors_total Total HTTP 5xx responses.\n");
        output.push_str("# TYPE http_request_errors_total counter\n");
        output.push_str(&format!(
            "http_request_errors_total {}\n",
            self.total_errors.load(Ordering::Relaxed)
        ));

        output.push_str(
            "# HELP http_requests_by_status_total Total HTTP requests grouped by status.\n",
        );
        output.push_str("# TYPE http_requests_by_status_total counter\n");
        if let Ok(per_status) = self.per_status.lock() {
            let mut entries = per_status.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(status, _)| **status);
            for (status, count) in entries {
                output.push_str(&format!(
                    "http_requests_by_status_total{{status=\"{}\"}} {}\n",
                    status, count
                ));
            }
        }

        output.push_str("# HELP http_requests_route_total Total HTTP requests grouped by method, route and status.\n");
        output.push_str("# TYPE http_requests_route_total counter\n");
        output.push_str("# HELP http_request_duration_ms_sum Accumulated request duration in milliseconds grouped by method, route and status.\n");
        output.push_str("# TYPE http_request_duration_ms_sum counter\n");
        if let Ok(per_route_status) = self.per_route_status.lock() {
            let mut entries = per_route_status.iter().collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for ((method, path, status), metric) in entries {
                let method = escape_label(method);
                let path = escape_label(path);
                output.push_str(&format!(
                    "http_requests_route_total{{method=\"{}\",path=\"{}\",status=\"{}\"}} {}\n",
                    method, path, status, metric.requests
                ));
                output.push_str(&format!(
                    "http_request_duration_ms_sum{{method=\"{}\",path=\"{}\",status=\"{}\"}} {:.3}\n",
                    method, path, status, metric.duration_ms_sum
                ));
            }
        }

        // --- P95/P99 API latency (per-route) ---
        output.push_str("# HELP http_request_duration_p95_ms 95th percentile request latency in milliseconds per route.\n");
        output.push_str("# TYPE http_request_duration_p95_ms gauge\n");
        if let Ok(per_route_status) = self.per_route_status.lock() {
            for ((method, path, _status), metric) in per_route_status.iter() {
                if let Some(p95) = metric.histogram.p95() {
                    output.push_str(&format!(
                        "http_request_duration_p95_ms{{method=\"{}\",path=\"{}\"}} {:.3}\n",
                        escape_label(method),
                        escape_label(path),
                        p95
                    ));
                }
            }
        }

        output.push_str("# HELP http_request_duration_p99_ms 99th percentile request latency in milliseconds per route.\n");
        output.push_str("# TYPE http_request_duration_p99_ms gauge\n");
        if let Ok(per_route_status) = self.per_route_status.lock() {
            for ((method, path, _status), metric) in per_route_status.iter() {
                if let Some(p99) = metric.histogram.p99() {
                    output.push_str(&format!(
                        "http_request_duration_p99_ms{{method=\"{}\",path=\"{}\"}} {:.3}\n",
                        escape_label(method),
                        escape_label(path),
                        p99
                    ));
                }
            }
        }

        // --- Cold start ---
        if let Some(cs_ms) = self.cold_start_ms() {
            output.push_str("# HELP cold_start_ms Server cold start duration in milliseconds.\n");
            output.push_str("# TYPE cold_start_ms gauge\n");
            output.push_str(&format!("cold_start_ms {:.3}\n", cs_ms));
        }

        // --- DB query timing ---
        output.push_str("# HELP db_query_duration_count Total DB queries executed.\n");
        output.push_str("# TYPE db_query_duration_count counter\n");
        output.push_str(&format!(
            "db_query_duration_count {}\n",
            self.db_query_count()
        ));

        output.push_str(
            "# HELP db_query_duration_p95_ms 95th percentile DB query latency in milliseconds.\n",
        );
        output.push_str("# TYPE db_query_duration_p95_ms gauge\n");
        if let Some(p95) = self.db_query_p95_ms() {
            output.push_str(&format!("db_query_duration_p95_ms {:.3}\n", p95));
        }

        output.push_str(
            "# HELP db_query_duration_p99_ms 99th percentile DB query latency in milliseconds.\n",
        );
        output.push_str("# TYPE db_query_duration_p99_ms gauge\n");
        if let Some(p99) = self.db_query_p99_ms() {
            output.push_str(&format!("db_query_duration_p99_ms {:.3}\n", p99));
        }

        if let Some(avg) = self.db_query_avg_ms() {
            output.push_str(
                "# HELP db_query_duration_avg_ms Average DB query latency in milliseconds.\n",
            );
            output.push_str("# TYPE db_query_duration_avg_ms gauge\n");
            output.push_str(&format!("db_query_duration_avg_ms {:.3}\n", avg));
        }

        // --- Redis op timing ---
        output.push_str("# HELP redis_op_duration_count Total Redis operations executed.\n");
        output.push_str("# TYPE redis_op_duration_count counter\n");
        output.push_str(&format!("redis_op_duration_count {}\n", self.redis_count()));

        output.push_str("# HELP redis_op_duration_p95_ms 95th percentile Redis operation latency in milliseconds.\n");
        output.push_str("# TYPE redis_op_duration_p95_ms gauge\n");
        if let Some(p95) = self.redis_p95_ms() {
            output.push_str(&format!("redis_op_duration_p95_ms {:.3}\n", p95));
        }

        output.push_str("# HELP redis_op_duration_p99_ms 99th percentile Redis operation latency in milliseconds.\n");
        output.push_str("# TYPE redis_op_duration_p99_ms gauge\n");
        if let Some(p99) = self.redis_p99_ms() {
            output.push_str(&format!("redis_op_duration_p99_ms {:.3}\n", p99));
        }

        if let Some(avg) = self.redis_avg_ms() {
            output.push_str("# HELP redis_op_duration_avg_ms Average Redis operation latency in milliseconds.\n");
            output.push_str("# TYPE redis_op_duration_avg_ms gauge\n");
            output.push_str(&format!("redis_op_duration_avg_ms {:.3}\n", avg));
        }

        // --- System resources (memory / CPU) ---
        output.push_str("# HELP process_used_memory_bytes Used memory in bytes.\n");
        output.push_str("# TYPE process_used_memory_bytes gauge\n");
        output.push_str(&format!(
            "process_used_memory_bytes {}\n",
            self.used_memory_bytes()
        ));

        output.push_str("# HELP process_total_memory_bytes Total system memory in bytes.\n");
        output.push_str("# TYPE process_total_memory_bytes gauge\n");
        output.push_str(&format!(
            "process_total_memory_bytes {}\n",
            self.total_memory_bytes()
        ));

        output.push_str("# HELP process_cpu_usage_percent CPU usage percentage.\n");
        output.push_str("# TYPE process_cpu_usage_percent gauge\n");
        output.push_str(&format!(
            "process_cpu_usage_percent {:.2}\n",
            self.cpu_usage_percent()
        ));

        output
    }
}

// ---------------------------------------------------------------------------
// Path normalization and helpers (existing)
// ---------------------------------------------------------------------------

fn normalize_path(path: &str) -> String {
    let normalized = path
        .split('/')
        .map(|segment| {
            if looks_like_uuid(segment) || is_numeric(segment) {
                ":id"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/");

    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized
    }
}

fn looks_like_uuid(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    if bytes.len() != 36 {
        return false;
    }

    for (idx, byte) in bytes.iter().enumerate() {
        let should_be_dash = matches!(idx, 8 | 13 | 18 | 23);
        if should_be_dash {
            if *byte != b'-' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn is_numeric(segment: &str) -> bool {
    !segment.is_empty() && segment.chars().all(|char| char.is_ascii_digit())
}

fn escape_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

// ---------------------------------------------------------------------------
// Convenience wrapper for timing a closure
// ---------------------------------------------------------------------------

/// Times the execution of a closure and records the duration to the given metric name.
/// Intended for use as: `registry.time_db(|| async { ... }).await`
impl MetricsRegistry {
    /// Execute a future, recording the elapsed time as a DB query duration.
    pub async fn time_db<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = f.await;
        self.record_db_query(start.elapsed());
        result
    }

    /// Execute a future, recording the elapsed time as a Redis operation duration.
    pub async fn time_redis<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = f.await;
        self.record_redis_op(start.elapsed());
        result
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{MetricsRegistry, normalize_path};

    #[test]
    fn normalize_path_replaces_uuid_and_numeric_segments() {
        assert_eq!(
            normalize_path("/api/v1/admin/users/123"),
            "/api/v1/admin/users/:id"
        );
        assert_eq!(
            normalize_path("/api/v1/admin/customers/833df066-04e8-476a-a29e-430cbc86eef6"),
            "/api/v1/admin/customers/:id"
        );
    }

    #[test]
    fn render_prometheus_includes_status_and_route_counters() {
        let registry = MetricsRegistry::new();
        registry.record(
            "GET",
            "/api/v1/admin/users/1",
            200,
            Duration::from_millis(15),
        );
        registry.record(
            "GET",
            "/api/v1/admin/users/2",
            200,
            Duration::from_millis(10),
        );
        registry.record(
            "POST",
            "/api/v1/admin/users",
            503,
            Duration::from_millis(50),
        );

        let rendered = registry.render_prometheus();

        assert!(rendered.contains("http_requests_total 3"));
        assert!(rendered.contains("http_request_errors_total 1"));
        assert!(rendered.contains("http_requests_by_status_total{status=\"200\"} 2"));
        assert!(rendered.contains("http_requests_by_status_total{status=\"503\"} 1"));
        assert!(rendered.contains(
            "http_requests_route_total{method=\"GET\",path=\"/api/v1/admin/users/:id\",status=\"200\"} 2"
        ));
        assert!(rendered.contains(
            "http_request_duration_ms_sum{method=\"GET\",path=\"/api/v1/admin/users/:id\",status=\"200\"} 25.000"
        ));
    }

    #[test]
    fn histogram_records_p95_p99() {
        let registry = MetricsRegistry::new();
        // Record 100 requests: 99 at 1ms, 1 at 100ms — P95 should be 1, P99 100
        for _ in 0..99 {
            registry.record("GET", "/test", 200, Duration::from_millis(1));
        }
        for _ in 0..1 {
            registry.record("GET", "/test", 200, Duration::from_millis(100));
        }

        let per_route = registry.per_route_status.lock().unwrap();
        let key = ("GET".to_string(), "/test".to_string(), 200);
        let metric = per_route.get(&key).unwrap();
        let p95 = metric.histogram.p95().unwrap();
        let p99 = metric.histogram.p99().unwrap();

        assert_eq!(
            p95, 1.0,
            "P95 should be 1ms (95th percentile falls in the fast group)"
        );
        assert_eq!(
            p99, 100.0,
            "P99 should be 100ms (top 1% falls in the slow group)"
        );
    }

    #[test]
    fn cold_start_is_recorded_and_rendered() {
        let registry = MetricsRegistry::new();
        assert!(registry.cold_start_ms().is_none());

        registry.record_cold_start(Duration::from_millis(1500));
        assert!(registry.cold_start_ms().is_some());
        assert_eq!(registry.cold_start_ms().unwrap(), 1500.0);

        let rendered = registry.render_prometheus();
        assert!(rendered.contains("cold_start_ms 1500.000"));
        assert!(rendered.contains("# TYPE cold_start_ms gauge"));
    }

    #[test]
    fn db_query_timing_tracks_count_and_percentiles() {
        let registry = MetricsRegistry::new();

        for _ in 0..96 {
            registry.record_db_query(Duration::from_millis(5));
        }
        for _ in 0..4 {
            registry.record_db_query(Duration::from_millis(50));
        }

        assert_eq!(registry.db_query_count(), 100);
        let p95 = registry.db_query_p95_ms().unwrap();
        let p99 = registry.db_query_p99_ms().unwrap();

        assert_eq!(p95, 5.0, "P95 should be 5ms");
        assert_eq!(p99, 50.0, "P99 should be 50ms");

        let rendered = registry.render_prometheus();
        assert!(rendered.contains("db_query_duration_count 100"));
        assert!(rendered.contains("db_query_duration_p95_ms 5.000"));
        assert!(rendered.contains("db_query_duration_p99_ms 50.000"));
    }

    #[test]
    fn redis_op_timing_tracks_count_and_percentiles() {
        let registry = MetricsRegistry::new();

        for _ in 0..96 {
            registry.record_redis_op(Duration::from_micros(500));
        }
        for _ in 0..4 {
            registry.record_redis_op(Duration::from_millis(10));
        }

        assert_eq!(registry.redis_count(), 100);

        let p95 = registry.redis_p95_ms().unwrap();
        let p99 = registry.redis_p99_ms().unwrap();

        // P95 should be 0.5ms, P99 should be 10ms
        assert!(
            (p95 - 0.5).abs() < 0.01,
            "P95 should be ~0.5ms, got {}",
            p95
        );
        assert!(
            (p99 - 10.0).abs() < 0.01,
            "P99 should be ~10ms, got {}",
            p99
        );

        let rendered = registry.render_prometheus();
        assert!(rendered.contains("redis_op_duration_count 100"));
        assert!(rendered.contains("# TYPE redis_op_duration_p95_ms gauge"));
        assert!(rendered.contains("# TYPE redis_op_duration_p99_ms gauge"));
    }

    #[test]
    fn time_db_wrapper_records_duration() {
        let registry = MetricsRegistry::new();
        let _result = futures::executor::block_on(registry.time_db(async {
            std::thread::sleep(Duration::from_millis(5));
            42
        }));
        assert_eq!(registry.db_query_count(), 1);
        let avg = registry.db_query_avg_ms().unwrap();
        assert!(avg >= 5.0, "Avg should be at least 5ms, got {}", avg);
    }

    #[test]
    fn time_redis_wrapper_records_duration() {
        let registry = MetricsRegistry::new();
        let _result = futures::executor::block_on(registry.time_redis(async {
            std::thread::sleep(Duration::from_millis(2));
            "ok"
        }));
        assert_eq!(registry.redis_count(), 1);
        let avg = registry.redis_avg_ms().unwrap();
        assert!(avg >= 2.0, "Avg should be at least 2ms, got {}", avg);
    }

    #[test]
    fn render_includes_system_measures() {
        let registry = MetricsRegistry::new();
        registry.refresh_system_measures();

        let rendered = registry.render_prometheus();
        assert!(rendered.contains("# TYPE process_used_memory_bytes gauge"));
        assert!(rendered.contains("# TYPE process_total_memory_bytes gauge"));
        assert!(rendered.contains("# TYPE process_cpu_usage_percent gauge"));
        assert!(rendered.contains("process_used_memory_bytes"));
        assert!(rendered.contains("process_total_memory_bytes"));
        assert!(rendered.contains("process_cpu_usage_percent"));
    }

    #[test]
    fn render_with_no_data_does_not_panic() {
        let registry = MetricsRegistry::new();
        let rendered = registry.render_prometheus();
        // Should still render header/comment blocks even with no observations
        assert!(rendered.contains("http_requests_total 0"));
        assert!(!rendered.contains("cold_start_ms")); // not recorded
    }

    #[test]
    fn histogram_avg_works() {
        let registry = MetricsRegistry::new();
        registry.record_db_query(Duration::from_millis(10));
        registry.record_db_query(Duration::from_millis(30));
        let avg = registry.db_query_avg_ms().unwrap();
        assert!((avg - 20.0).abs() < 0.01, "Avg should be 20ms, got {}", avg);
    }
}
