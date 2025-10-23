//! Test broker utilities for running rumqttd during integration tests
//!
//! This module provides utilities to spawn and manage a rumqttd MQTT broker
//! instance for testing purposes.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Transport type for the broker
#[derive(Debug, Clone)]
pub enum BrokerTransport {
    /// TCP socket with host and port
    Tcp { host: String, port: u16 },
    /// Unix domain socket with path
    Unix { path: PathBuf },
}

impl Default for BrokerTransport {
    fn default() -> Self {
        // Generate a random socket name in /tmp
        let random_id: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let socket_path = format!("/tmp/mqtt_test_{}.sock", random_id);
        
        Self::Unix {
            path: PathBuf::from(socket_path),
        }
    }
}

/// Configuration for the test MQTT broker
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    /// Transport type (TCP or Unix socket)
    pub transport: BrokerTransport,
    /// Timeout for broker startup (default: 5 seconds)
    pub startup_timeout: Duration,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            transport: BrokerTransport::default(),
            startup_timeout: Duration::from_secs(5),
        }
    }
}

impl BrokerConfig {
    /// Create a new configuration with TCP transport
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self {
            transport: BrokerTransport::Tcp {
                host: host.into(),
                port,
            },
            startup_timeout: Duration::from_secs(5),
        }
    }

    /// Create a new configuration with Unix socket transport
    pub fn unix(path: impl Into<PathBuf>) -> Self {
        Self {
            transport: BrokerTransport::Unix {
                path: path.into(),
            },
            startup_timeout: Duration::from_secs(5),
        }
    }

    /// Get the MQTT URI for this broker configuration
    pub fn mqtt_uri(&self) -> String {
        match &self.transport {
            BrokerTransport::Tcp { host, port } => format!("mqtt://{}:{}", host, port),
            BrokerTransport::Unix { path } => {
                format!("mqtt+unix://{}", path.display())
            }
        }
    }
}

/// A managed rumqttd broker instance for testing
pub struct TestBroker {
    process: Child,
    config: BrokerConfig,
}

impl TestBroker {
    /// Start a new rumqttd broker instance
    ///
    /// This requires rumqttd to be installed and available in PATH.
    /// Install with: `cargo install rumqttd`
    pub async fn start(config: BrokerConfig) -> Result<Self, String> {
        // Create listener configuration based on transport type
        let listener_config = match &config.transport {
            BrokerTransport::Tcp { host, port } => {
                format!(
                    r#"[v5.1.listeners.tcp1]
name = "tcp1"
port = {}
bind = "{}"
max_connections = 1000
"#,
                    port, host
                )
            }
            BrokerTransport::Unix { path } => {
                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create socket directory: {}", e))?;
                }
                
                // Remove socket file if it exists
                if path.exists() {
                    std::fs::remove_file(path)
                        .map_err(|e| format!("Failed to remove existing socket: {}", e))?;
                }
                
                format!(
                    r#"[v5.1.listeners.unix1]
name = "unix1"
path = "{}"
max_connections = 1000
"#,
                    path.display()
                )
            }
        };

        // Create a minimal rumqttd configuration
        let config_content = format!(
            r#"
[v5]
console_logs = false

[v5.1.router]
max_segment_size = 10240
max_segment_count = 10

[v5.1.connections.1]
connection_timeout_ms = 60000
max_client_id_len = 256
throttle_delay_ms = 0
max_payload_size = 20480
max_inflight_count = 500
max_inflight_size = 1024

{}
"#,
            listener_config
        );

        // Write config to a temporary file
        let config_path = "/tmp/rumqttd_test_config.toml";
        std::fs::write(config_path, config_content)
            .map_err(|e| format!("Failed to write broker config: {}", e))?;

        // Start rumqttd
        let process = Command::new("rumqttd")
            .arg("-c")
            .arg(config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start rumqttd: {}. Is it installed?", e))?;

        // Wait for broker to start
        sleep(config.startup_timeout).await;

        Ok(Self { process, config })
    }

    /// Start a broker with default configuration
    pub async fn start_default() -> Result<Self, String> {
        Self::start(BrokerConfig::default()).await
    }

    /// Get the MQTT URI for connecting to this broker
    pub fn mqtt_uri(&self) -> String {
        self.config.mqtt_uri()
    }

    /// Get the broker configuration
    pub fn config(&self) -> &BrokerConfig {
        &self.config
    }

    /// Stop the broker
    pub fn stop(mut self) -> Result<(), String> {
        self.process
            .kill()
            .map_err(|e| format!("Failed to stop broker: {}", e))?;
        self.process
            .wait()
            .map_err(|e| format!("Failed to wait for broker: {}", e))?;
        
        // Clean up Unix socket if present
        if let BrokerTransport::Unix { path } = &self.config.transport {
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
        
        Ok(())
    }
}

impl Drop for TestBroker {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        
        // Clean up Unix socket if present
        if let BrokerTransport::Unix { path } = &self.config.transport {
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run if rumqttd is installed
    async fn test_broker_start_stop_default() {
        let broker = TestBroker::start_default().await;
        assert!(broker.is_ok());
        
        if let Ok(broker) = broker {
            // Default now uses Unix socket
            assert!(broker.mqtt_uri().starts_with("mqtt+unix:///tmp/mqtt_test_"));
            assert!(broker.mqtt_uri().ends_with(".sock"));
            assert!(broker.stop().is_ok());
        }
    }

    #[tokio::test]
    #[ignore] // Only run if rumqttd is installed
    async fn test_broker_start_stop_unix() {
        let socket_path = "/tmp/test_mqtt.sock";
        let config = BrokerConfig::unix(socket_path);
        
        let broker = TestBroker::start(config).await;
        assert!(broker.is_ok());
        
        if let Ok(broker) = broker {
            assert_eq!(broker.mqtt_uri(), format!("mqtt+unix://{}", socket_path));
            
            // Verify socket was created
            assert!(std::path::Path::new(socket_path).exists());
            
            assert!(broker.stop().is_ok());
            
            // Verify socket was cleaned up
            assert!(!std::path::Path::new(socket_path).exists());
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_broker_custom_tcp_config() {
        let config = BrokerConfig::tcp("127.0.0.1", 1884);
        let broker = TestBroker::start(config).await;
        
        assert!(broker.is_ok());
        if let Ok(broker) = broker {
            assert_eq!(broker.mqtt_uri(), "mqtt://127.0.0.1:1884");
            assert!(broker.stop().is_ok());
        }
    }
}
