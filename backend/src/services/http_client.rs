#![allow(dead_code)]

use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Circuit breaker is open, failing fast")]
    CircuitBreakerOpen,
    #[error("Request timeout")]
    Timeout,
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    state: Arc<tokio::sync::RwLock<CircuitState>>,
    failures: AtomicU32,
    successes: AtomicU32,
    last_failure_time: AtomicU64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            failures: AtomicU32::new(0),
            successes: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
        }
    }

    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, HttpClientError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: Into<HttpClientError>,
    {
        if !self.can_execute().await {
            return Err(HttpClientError::CircuitBreakerOpen);
        }

        let result = f().await.map_err(Into::into);

        match &result {
            Ok(_) => self.on_success().await,
            Err(_) => self.on_failure().await,
        }

        result
    }

    async fn can_execute(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now - last_failure >= self.timeout.as_secs() {
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::HalfOpen;
                    self.successes.store(0, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            },
            CircuitState::HalfOpen => true,
        }
    }

    async fn on_success(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => {
                self.failures.store(0, Ordering::Relaxed);
            },
            CircuitState::HalfOpen => {
                let successes = self.successes.fetch_add(1, Ordering::Relaxed) + 1;
                if successes >= self.success_threshold {
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::Closed;
                    self.failures.store(0, Ordering::Relaxed);
                    self.successes.store(0, Ordering::Relaxed);
                    debug!("Circuit breaker closed after successful requests");
                }
            },
            CircuitState::Open => {},
        }
    }

    async fn on_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_failure_time.store(now, Ordering::Relaxed);

        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => {
                if failures >= self.failure_threshold {
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::Open;
                    warn!("Circuit breaker opened after {} failures", failures);
                }
            },
            CircuitState::HalfOpen => {
                drop(state);
                let mut state = self.state.write().await;
                *state = CircuitState::Open;
                warn!("Circuit breaker reopened after failure in half-open state");
            },
            CircuitState::Open => {},
        }
    }

    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }
}

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_base_delay: Duration,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: Duration,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(100),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: Duration::from_secs(60),
        }
    }
}

pub struct HttpClient {
    client: Client,
    breaker: Arc<CircuitBreaker>,
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Result<Self, HttpClientError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(HttpClientError::HttpError)?;

        let breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_threshold,
            2,
            config.circuit_breaker_timeout,
        ));

        Ok(Self {
            client,
            breaker,
            config,
        })
    }

    pub async fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.client.post(url),
            self.breaker.clone(),
            self.config.clone(),
        )
    }

    pub async fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.client.get(url),
            self.breaker.clone(),
            self.config.clone(),
        )
    }

    pub async fn circuit_state(&self) -> CircuitState {
        self.breaker.state().await
    }
}

pub struct RequestBuilder {
    request: reqwest::RequestBuilder,
    breaker: Arc<CircuitBreaker>,
    config: HttpClientConfig,
}

impl RequestBuilder {
    fn new(
        request: reqwest::RequestBuilder,
        breaker: Arc<CircuitBreaker>,
        config: HttpClientConfig,
    ) -> Self {
        Self {
            request,
            breaker,
            config,
        }
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.request = self.request.header(key, value);
        self
    }

    pub fn json<T: serde::Serialize>(mut self, json: &T) -> Self {
        self.request = self.request.json(json);
        self
    }

    pub async fn send(self) -> Result<reqwest::Response, HttpClientError> {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt <= self.config.max_retries {
            if !self.breaker.can_execute().await {
                return Err(HttpClientError::CircuitBreakerOpen);
            }

            let response = self
                .request
                .try_clone()
                .ok_or(HttpClientError::IoError(std::io::Error::other(
                    "Failed to clone request",
                )))?
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() || resp.status().is_redirection() {
                        self.breaker.on_success().await;
                        return Ok(resp);
                    } else if resp.status().is_server_error() {
                        self.breaker.on_failure().await;
                        last_error = Some(HttpClientError::IoError(std::io::Error::other(
                            format!("Server error: {}", resp.status()),
                        )));
                    } else {
                        // Client error (4xx) - don't retry, don't count as failure
                        return Ok(resp);
                    }
                },
                Err(e) => {
                    self.breaker.on_failure().await;
                    last_error = Some(HttpClientError::HttpError(e));
                },
            }

            attempt += 1;
            if attempt <= self.config.max_retries {
                let delay = self.config.retry_base_delay * 2_u32.pow(attempt - 1);
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error.unwrap_or(HttpClientError::MaxRetriesExceeded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, 2, Duration::from_secs(60));

        for _ in 0..3 {
            let _ = cb
                .call(|| async {
                    Err::<(), _>(HttpClientError::IoError(std::io::Error::other("test")))
                })
                .await;
        }

        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets_failures() {
        let cb = CircuitBreaker::new(3, 2, Duration::from_secs(60));

        let _ = cb
            .call(|| async {
                Err::<(), _>(HttpClientError::IoError(std::io::Error::other("test")))
            })
            .await;

        let _ = cb.call(|| async { Ok::<(), HttpClientError>(()) }).await;
        let _ = cb.call(|| async { Ok::<(), HttpClientError>(()) }).await;
        let _ = cb.call(|| async { Ok::<(), HttpClientError>(()) }).await;

        assert_eq!(cb.state().await, CircuitState::Closed);
    }
}
