//! Watchdog that alerts when a large share of the traffic starts being
//! refused by the rate limiter.
//!
//! The decision itself is a pure function over the counts of one window,
//! so the threshold behaviour is tested without clocks or I/O. The loop
//! that samples the registry, emits the alert and keeps the share gauge
//! current is thin glue around that function.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::services::metrics_service::{MetricsRegistry, RouteRejection};

/// Configuration of the rejection-share alert.
#[derive(Debug, Clone)]
pub struct Alert429Config {
    /// When false the watchdog still publishes the share gauge and nothing else.
    pub enabled: bool,
    /// Share of refused requests that trips the alert, as a fraction (0.10 = 10%).
    pub threshold_share: f64,
    /// Seconds between two evaluations.
    pub window_secs: u64,
}

/// Counts observed in one evaluation window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShareWindow {
    /// Requests seen in the window.
    pub total: u64,
    /// Requests the rate limiter refused in the window.
    pub rejected: u64,
    /// Refused routes, ordered by count descending.
    pub routes: Vec<RouteRejection>,
}

/// One window whose share of refused requests crossed the threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct ShareAlert {
    /// Measured share, as a fraction.
    pub share: f64,
    /// Threshold that was crossed, as a fraction.
    pub threshold: f64,
    /// Requests seen in the window.
    pub total: u64,
    /// Requests refused in the window.
    pub rejected: u64,
    /// Most refused route in the window, when any route was refused.
    pub top_route: Option<RouteRejection>,
}

/// Decide whether a window justifies an alert. Pure: no clock, no I/O.
///
/// A window without traffic has no share to judge, and a disabled
/// configuration never alerts even with all traffic refused.
pub fn evaluate_429_share(window: &ShareWindow, config: &Alert429Config) -> Option<ShareAlert> {
    if !config.enabled || window.total == 0 {
        return None;
    }
    let share = window.rejected as f64 / window.total as f64;
    if share < config.threshold_share {
        return None;
    }
    Some(ShareAlert {
        share,
        threshold: config.threshold_share,
        total: window.total,
        rejected: window.rejected,
        top_route: window.routes.first().cloned(),
    })
}

/// One alert per crossing event, so a long storm does not repeat the alert.
#[derive(Debug, Default)]
pub struct ShareAlertState {
    alerted: bool,
}

impl ShareAlertState {
    /// Feed one window: returns the alert on the crossing, `None` on the
    /// windows already covered by it and on the quiet ones (which also
    /// rearm the next crossing).
    pub fn observe(&mut self, window: &ShareWindow, config: &Alert429Config) -> Option<ShareAlert> {
        match evaluate_429_share(window, config) {
            Some(alert) if !self.alerted => {
                self.alerted = true;
                Some(alert)
            },
            Some(_) => None,
            None => {
                self.alerted = false;
                None
            },
        }
    }
}

/// Turns cumulative registry counters into per-window counts.
#[derive(Debug, Default)]
pub struct ShareSampler {
    started: bool,
    previous_total: u64,
    previous_rejected: u64,
    previous_routes: HashMap<(String, String), u64>,
}

impl ShareSampler {
    /// Window counts since the previous call, computed with saturating
    /// differences so a registry reset cannot produce a negative window.
    pub fn window(&mut self, registry: &MetricsRegistry) -> ShareWindow {
        let current_total = registry.total_requests();
        let current_rejected = registry.rejected_by_status(429);
        let current_routes = registry.route_rejections(429);

        // The first sample only stores the baseline: boot-time traffic has
        // no window to be judged against and would false-fire the first alert.
        if !self.started {
            self.started = true;
            self.previous_total = current_total;
            self.previous_rejected = current_rejected;
            self.previous_routes = current_routes
                .iter()
                .map(|route| ((route.method.clone(), route.path.clone()), route.count))
                .collect();
            return ShareWindow::default();
        }

        let total = current_total.saturating_sub(self.previous_total);
        let rejected = current_rejected.saturating_sub(self.previous_rejected);

        let mut routes: Vec<RouteRejection> = current_routes
            .iter()
            .map(|route| {
                let key = (route.method.clone(), route.path.clone());
                let previous = self.previous_routes.get(&key).copied().unwrap_or(0);
                RouteRejection {
                    count: route.count.saturating_sub(previous),
                    ..route.clone()
                }
            })
            .filter(|route| route.count > 0)
            .collect();

        self.previous_total = current_total;
        self.previous_rejected = current_rejected;
        self.previous_routes = current_routes
            .iter()
            .map(|route| ((route.method.clone(), route.path.clone()), route.count))
            .collect();

        routes.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.method.cmp(&right.method))
                .then_with(|| left.path.cmp(&right.path))
        });

        ShareWindow {
            total,
            rejected,
            routes,
        }
    }
}

/// Periodic evaluation loop: samples the registry, emits one structured
/// warning per crossing event and keeps the share gauge current.
pub async fn run_429_share_watchdog(registry: Arc<MetricsRegistry>, config: Alert429Config) {
    let mut sampler = ShareSampler::default();
    let mut alert_state = ShareAlertState::default();
    let mut ticker = tokio::time::interval(Duration::from_secs(config.window_secs.max(1)));

    loop {
        ticker.tick().await;
        let window = sampler.window(&registry);
        if let Some(alert) = alert_state.observe(&window, &config) {
            registry.record_429_alert_emitted();
            emit_alert(&alert, &config);
        }
        let share = if window.total == 0 {
            0.0
        } else {
            window.rejected as f64 / window.total as f64
        };
        registry.record_429_share(share);
    }
}

/// One structured warning per crossing event: the payload carries the
/// numbers an operator needs to decide, and the most refused route keeps
/// the diagnosis from ending at "something is 429ing".
fn emit_alert(alert: &ShareAlert, config: &Alert429Config) {
    let top_route = alert.top_route.as_ref();
    tracing::warn!(
        event = "rate_limit.429_share_threshold_exceeded",
        share_pct = %format!("{:.2}", alert.share * 100.0),
        threshold_pct = %format!("{:.2}", alert.threshold * 100.0),
        window_secs = config.window_secs,
        window_requests = alert.total,
        window_rejected = alert.rejected,
        top_route_method = top_route.map(|route| route.method.as_str()).unwrap_or("-"),
        top_route_path = top_route.map(|route| route.path.as_str()).unwrap_or("-"),
        top_route_count = top_route.map(|route| route.count).unwrap_or(0),
        "a large share of the traffic was refused by the rate limiter"
    );
}

#[cfg(test)]
mod decision_tests {
    use super::*;

    fn config() -> Alert429Config {
        Alert429Config {
            enabled: true,
            threshold_share: 0.10,
            window_secs: 30,
        }
    }

    #[test]
    fn a_window_without_traffic_asks_for_no_alert() {
        let window = ShareWindow::default();
        assert!(
            evaluate_429_share(&window, &config()).is_none(),
            "an empty window has no share to judge"
        );
    }

    #[test]
    fn a_share_below_the_threshold_asks_for_no_alert() {
        let window = ShareWindow {
            total: 100,
            rejected: 9,
            routes: Vec::new(),
        };
        assert!(
            evaluate_429_share(&window, &config()).is_none(),
            "9% sits below the 10% threshold"
        );
    }

    #[test]
    fn a_share_at_the_threshold_alerts_with_the_top_route() {
        let routes = vec![
            RouteRejection {
                method: "POST".to_string(),
                path: "/api/v1/auth/login".to_string(),
                count: 7,
            },
            RouteRejection {
                method: "POST".to_string(),
                path: "/api/v1/auth/refresh".to_string(),
                count: 2,
            },
        ];
        let window = ShareWindow {
            total: 100,
            rejected: 18,
            routes,
        };
        let alert = evaluate_429_share(&window, &config()).expect("18% crosses the threshold");
        assert_eq!(alert.rejected, 18);
        assert!(
            (alert.share - 0.18).abs() < 1e-9,
            "share must be the measured 18%, got {}",
            alert.share
        );
        let top = alert
            .top_route
            .expect("the top route must travel with the alert");
        assert_eq!(top.path, "/api/v1/auth/login");
        assert_eq!(top.count, 7);
    }

    #[test]
    fn a_disabled_config_never_alerts() {
        let window = ShareWindow {
            total: 10,
            rejected: 10,
            routes: Vec::new(),
        };
        let mut config = config();
        config.enabled = false;
        assert!(
            evaluate_429_share(&window, &config).is_none(),
            "a disabled alert must stay silent even with every request refused"
        );
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn config() -> Alert429Config {
        Alert429Config {
            enabled: true,
            threshold_share: 0.10,
            window_secs: 30,
        }
    }

    fn crossing_window() -> ShareWindow {
        ShareWindow {
            total: 100,
            rejected: 30,
            routes: vec![RouteRejection {
                method: "POST".to_string(),
                path: "/api/v1/auth/login".to_string(),
                count: 30,
            }],
        }
    }

    #[test]
    fn one_storm_emits_one_alert() {
        let mut state = ShareAlertState::default();
        assert!(
            state.observe(&crossing_window(), &config()).is_some(),
            "the first crossing alerts"
        );
        assert!(
            state.observe(&crossing_window(), &config()).is_none(),
            "the windows inside the same storm stay silent"
        );
    }

    #[test]
    fn a_quiet_window_rearms_the_next_crossing() {
        let mut state = ShareAlertState::default();
        assert!(state.observe(&crossing_window(), &config()).is_some());
        assert!(
            state.observe(&ShareWindow::default(), &config()).is_none(),
            "recovery is silent"
        );
        assert!(
            state.observe(&crossing_window(), &config()).is_some(),
            "a new storm alerts again"
        );
    }

    #[test]
    fn quiet_windows_before_any_storm_stay_silent() {
        let mut state = ShareAlertState::default();
        assert!(state.observe(&ShareWindow::default(), &config()).is_none());
    }
}

#[cfg(test)]
mod sampler_tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn the_window_is_the_difference_between_two_samples() {
        let registry = MetricsRegistry::new();
        registry.record("POST", "/api/v1/auth/login", 200, Duration::from_millis(3));
        registry.record("POST", "/api/v1/auth/login", 200, Duration::from_millis(3));

        let mut sampler = ShareSampler::default();
        let baseline = sampler.window(&registry);
        assert_eq!(
            baseline,
            ShareWindow::default(),
            "the first sample only stores the baseline"
        );

        registry.record("POST", "/api/v1/auth/login", 200, Duration::from_millis(3));
        registry.record("POST", "/api/v1/auth/login", 429, Duration::from_millis(3));
        registry.record("POST", "/api/v1/auth/login", 429, Duration::from_millis(3));

        let window = sampler.window(&registry);
        assert_eq!(window.total, 3, "three new requests since the baseline");
        assert_eq!(window.rejected, 2, "two of them were refused");
        assert_eq!(window.routes.len(), 1);
        assert_eq!(window.routes[0].count, 2);
    }

    #[test]
    fn an_idle_registry_yields_empty_windows() {
        let registry = MetricsRegistry::new();
        let mut sampler = ShareSampler::default();
        sampler.window(&registry);
        let window = sampler.window(&registry);
        assert_eq!(window, ShareWindow::default(), "no traffic means no window");
    }
}
