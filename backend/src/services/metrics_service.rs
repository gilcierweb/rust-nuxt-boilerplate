use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Default)]
struct RouteMetric {
    requests: u64,
    duration_ms_sum: f64,
}

#[derive(Default)]
pub struct MetricsRegistry {
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    per_status: Mutex<HashMap<u16, u64>>,
    per_route_status: Mutex<HashMap<(String, String, u16), RouteMetric>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

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
                .or_default();
            entry.requests += 1;
            entry.duration_ms_sum += duration.as_secs_f64() * 1000.0;
        }
    }

    pub fn render_prometheus(&self) -> String {
        let mut output = String::new();

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

        output.push_str(
            "# HELP http_requests_route_total Total HTTP requests grouped by method, route and status.\n",
        );
        output.push_str("# TYPE http_requests_route_total counter\n");
        output.push_str(
            "# HELP http_request_duration_ms_sum Accumulated request duration in milliseconds grouped by method, route and status.\n",
        );
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

        output
    }
}

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
}
