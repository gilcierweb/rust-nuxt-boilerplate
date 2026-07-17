use opentelemetry_sdk::trace::Sampler;

/// Environment variable for enabling/disabling sampling.
pub const ENV_OTEL_ENABLED: &str = "OTEL_ENABLED";

/// Environment variable for sampler type.
/// Supported values: "always_on", "always_off", "parent_based", "ratio_based"
pub const ENV_OTEL_SAMPLER: &str = "OTEL_SAMPLER";

/// Environment variable for sampler ratio (0.0 – 1.0).
/// Only used when OTEL_SAMPLER=ratio_based.
pub const ENV_OTEL_SAMPLER_RATIO: &str = "OTEL_SAMPLER_RATIO";

/// Environment variable for the OTLP endpoint.
pub const ENV_OTEL_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// Default OTLP gRPC endpoint.
pub const DEFAULT_OTEL_ENDPOINT: &str = "http://localhost:4317";

/// Default sampling ratio for production (10% of traces).
pub const DEFAULT_PROD_RATIO: f64 = 0.1;

/// Default sampling ratio for staging (50% of traces).
pub const DEFAULT_STAGING_RATIO: f64 = 0.5;

/// Maximum allowed sampling ratio.
pub const MAX_RATIO: f64 = 1.0;

/// Minimum allowed sampling ratio.
pub const MIN_RATIO: f64 = 0.0;

/// Telemetry configuration read from environment variables.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub sampler: SamplerType,
}

/// The sampler strategy selected via environment variables.
#[derive(Debug, Clone, PartialEq)]
pub enum SamplerType {
    AlwaysOn,
    AlwaysOff,
    ParentBased,
    RatioBased(f64),
}

impl TelemetryConfig {
    /// Build configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        let enabled = std::env::var(ENV_OTEL_ENABLED)
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        let endpoint =
            std::env::var(ENV_OTEL_ENDPOINT).unwrap_or_else(|_| DEFAULT_OTEL_ENDPOINT.to_string());

        let sampler = if !enabled {
            SamplerType::AlwaysOff
        } else {
            Self::resolve_sampler()
        };

        Self {
            enabled,
            endpoint,
            sampler,
        }
    }

    /// Resolve sampler type from environment variables.
    fn resolve_sampler() -> SamplerType {
        let sampler_str =
            std::env::var(ENV_OTEL_SAMPLER).unwrap_or_else(|_| "parent_based".to_string());

        match sampler_str.as_str() {
            "always_on" => SamplerType::AlwaysOn,
            "always_off" => SamplerType::AlwaysOff,
            "parent_based" => SamplerType::ParentBased,
            "ratio_based" => {
                let ratio = Self::read_ratio();
                SamplerType::RatioBased(ratio)
            },
            _ => {
                tracing::warn!(
                    sampler = %sampler_str,
                    "Unknown OTEL_SAMPLER value, falling back to parent_based"
                );
                SamplerType::ParentBased
            },
        }
    }

    /// Read and validate the sampling ratio from environment.
    fn read_ratio() -> f64 {
        std::env::var(ENV_OTEL_SAMPLER_RATIO)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .map(|r| r.clamp(MIN_RATIO, MAX_RATIO))
            .unwrap_or(DEFAULT_PROD_RATIO)
    }

    /// Build the OpenTelemetry `Sampler` from this configuration.
    pub fn build_sampler(&self) -> Sampler {
        match &self.sampler {
            SamplerType::AlwaysOn => Sampler::AlwaysOn,
            SamplerType::AlwaysOff => Sampler::AlwaysOff,
            SamplerType::ParentBased => Sampler::ParentBased(Box::new(Sampler::AlwaysOn)),
            SamplerType::RatioBased(ratio) => Sampler::TraceIdRatioBased(*ratio),
        }
    }

    /// Log the active sampling configuration at startup.
    pub fn log_config(&self) {
        tracing::info!(
            otel_enabled = self.enabled,
            otel_endpoint = %self.endpoint,
            otel_sampler = ?self.sampler,
            "OpenTelemetry sampling configuration loaded"
        );
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: DEFAULT_OTEL_ENDPOINT.to_string(),
            sampler: SamplerType::ParentBased,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn from_env_defaults() {
        unsafe {
            std::env::remove_var(ENV_OTEL_ENABLED);
            std::env::remove_var(ENV_OTEL_SAMPLER);
            std::env::remove_var(ENV_OTEL_SAMPLER_RATIO);
        }

        let config = TelemetryConfig::from_env();
        assert!(config.enabled);
        assert_eq!(config.endpoint, DEFAULT_OTEL_ENDPOINT);
        assert_eq!(config.sampler, SamplerType::ParentBased);
    }

    #[test]
    #[serial]
    fn disabled_when_env_false() {
        unsafe { std::env::set_var(ENV_OTEL_ENABLED, "false") };
        let config = TelemetryConfig::from_env();
        assert!(!config.enabled);
        assert_eq!(config.sampler, SamplerType::AlwaysOff);
        unsafe { std::env::remove_var(ENV_OTEL_ENABLED) };
    }

    #[test]
    #[serial]
    fn disabled_when_env_zero() {
        unsafe { std::env::set_var(ENV_OTEL_ENABLED, "0") };
        let config = TelemetryConfig::from_env();
        assert!(!config.enabled);
        unsafe { std::env::remove_var(ENV_OTEL_ENABLED) };
    }

    #[test]
    #[serial]
    fn always_on_sampler() {
        unsafe { std::env::set_var(ENV_OTEL_SAMPLER, "always_on") };
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::AlwaysOn);
        unsafe { std::env::remove_var(ENV_OTEL_SAMPLER) };
    }

    #[test]
    #[serial]
    fn always_off_sampler() {
        unsafe { std::env::set_var(ENV_OTEL_SAMPLER, "always_off") };
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::AlwaysOff);
        unsafe { std::env::remove_var(ENV_OTEL_SAMPLER) };
    }

    #[test]
    #[serial]
    fn ratio_based_custom() {
        unsafe {
            std::env::set_var(ENV_OTEL_SAMPLER, "ratio_based");
            std::env::set_var(ENV_OTEL_SAMPLER_RATIO, "0.25");
        }
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::RatioBased(0.25));
        unsafe {
            std::env::remove_var(ENV_OTEL_SAMPLER);
            std::env::remove_var(ENV_OTEL_SAMPLER_RATIO);
        }
    }

    #[test]
    #[serial]
    fn ratio_based_clamped_max() {
        unsafe {
            std::env::set_var(ENV_OTEL_SAMPLER, "ratio_based");
            std::env::set_var(ENV_OTEL_SAMPLER_RATIO, "5.0");
        }
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::RatioBased(1.0));
        unsafe {
            std::env::remove_var(ENV_OTEL_SAMPLER);
            std::env::remove_var(ENV_OTEL_SAMPLER_RATIO);
        }
    }

    #[test]
    #[serial]
    fn ratio_based_clamped_min() {
        unsafe {
            std::env::set_var(ENV_OTEL_SAMPLER, "ratio_based");
            std::env::set_var(ENV_OTEL_SAMPLER_RATIO, "-1.0");
        }
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::RatioBased(0.0));
        unsafe {
            std::env::remove_var(ENV_OTEL_SAMPLER);
            std::env::remove_var(ENV_OTEL_SAMPLER_RATIO);
        }
    }

    #[test]
    #[serial]
    fn ratio_based_invalid_falls_back() {
        unsafe {
            std::env::set_var(ENV_OTEL_SAMPLER, "ratio_based");
            std::env::set_var(ENV_OTEL_SAMPLER_RATIO, "not_a_number");
        }
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::RatioBased(DEFAULT_PROD_RATIO));
        unsafe {
            std::env::remove_var(ENV_OTEL_SAMPLER);
            std::env::remove_var(ENV_OTEL_SAMPLER_RATIO);
        }
    }

    #[test]
    #[serial]
    fn unknown_sampler_falls_back_to_parent_based() {
        unsafe { std::env::set_var(ENV_OTEL_SAMPLER, "unknown_sampler") };
        let config = TelemetryConfig::from_env();
        assert_eq!(config.sampler, SamplerType::ParentBased);
        unsafe { std::env::remove_var(ENV_OTEL_SAMPLER) };
    }

    #[test]
    #[serial]
    fn custom_endpoint() {
        unsafe { std::env::set_var(ENV_OTEL_ENDPOINT, "http://collector:4318") };
        let config = TelemetryConfig::from_env();
        assert_eq!(config.endpoint, "http://collector:4318");
        unsafe { std::env::remove_var(ENV_OTEL_ENDPOINT) };
    }

    #[test]
    fn build_sampler_always_on() {
        let config = TelemetryConfig {
            sampler: SamplerType::AlwaysOn,
            ..Default::default()
        };
        let _sampler = config.build_sampler();
    }

    #[test]
    fn build_sampler_always_off() {
        let config = TelemetryConfig {
            sampler: SamplerType::AlwaysOff,
            ..Default::default()
        };
        let _sampler = config.build_sampler();
    }

    #[test]
    fn build_sampler_parent_based() {
        let config = TelemetryConfig {
            sampler: SamplerType::ParentBased,
            ..Default::default()
        };
        let _sampler = config.build_sampler();
    }

    #[test]
    fn build_sampler_ratio_based() {
        let config = TelemetryConfig {
            sampler: SamplerType::RatioBased(0.5),
            ..Default::default()
        };
        let _sampler = config.build_sampler();
    }

    #[test]
    fn default_config_is_parent_based() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.sampler, SamplerType::ParentBased);
    }
}
