pub mod message;

use async_trait::async_trait;
use message::MqttMessage;
use std::fmt;
use tokio::sync::{broadcast, watch};

/// Custom error type for MQTT operations
#[derive(Debug, Clone)]
pub enum MqttError {
    /// Connection error
    ConnectionError(String),
    /// Subscription error
    SubscriptionError(String),
    /// Unsubscribe error
    UnsubscribeError(String),
    /// Publish error
    PublishError(String),
    /// Disconnection error
    DisconnectionError(String),
    /// Invalid topic
    InvalidTopic(String),
    /// Invalid QoS
    InvalidQoS(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            MqttError::SubscriptionError(msg) => write!(f, "Subscription error: {}", msg),
            MqttError::UnsubscribeError(msg) => write!(f, "Unsubscribe error: {}", msg),
            MqttError::PublishError(msg) => write!(f, "Publish error: {}", msg),
            MqttError::DisconnectionError(msg) => write!(f, "Disconnection error: {}", msg),
            MqttError::InvalidTopic(msg) => write!(f, "Invalid topic: {}", msg),
            MqttError::InvalidQoS(msg) => write!(f, "Invalid QoS: {}", msg),
            MqttError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MqttError {}

/// Represents the connection state of an MQTT client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttConnectionState {
    /// Client is disconnected from the broker
    Disconnected,
    /// Client is in the process of connecting to the broker
    Connecting,
    /// Client is connected to the broker
    Connected,
    /// Client is in the process of disconnecting from the broker
    Disconnecting,
}

/// Trait defining the interface for an MQTT client
#[async_trait]
pub trait MqttClient {
    /// Connect to an MQTT broker using the provided URI
    /// 
    /// This function awaits until a CONNACK is received from the broker before returning,
    /// or returns an error if the connection fails.
    /// 
    /// The URI should be in the format `<scheme>://<hostname>:<port>[/path]` where:
    /// - `scheme` can be: `unix`, `ws`, `wss`, `mqtt`, or `mqtts`
    /// - `hostname` is the broker address (IP or domain name)
    /// - `port` is the broker port number (not for unix domain socket connections)
    /// - `path` (optional) is the base path for WebSocket connections
    /// 
    /// # Examples
    /// - `mqtt://localhost:1883` - Standard MQTT over TCP
    /// - `mqtts://broker.example.com:8883` - MQTT over TLS
    /// - `ws://localhost:9001` - MQTT over WebSocket
    /// - `ws://localhost:9001/ws` - MQTT over WebSocket with base path
    /// - `wss://broker.example.com:443` - MQTT over secure WebSocket
    /// - `wss://broker.example.com:443/mqtt` - MQTT over secure WebSocket with base path
    /// - `unix:///tmp/mqtt.sock` - MQTT over Unix domain socket
    async fn connect(&mut self, uri: String) -> Result<(), MqttError>;

    /// Set the Last Will message to be sent by the broker when the client disconnects unexpectedly
    /// 
    /// This should be called before `connect()` to ensure the Last Will is registered with the broker
    /// during the initial connection handshake.
    fn set_last_will(&mut self, message: MqttMessage);

    /// Get the client ID
    fn get_client_id(&self) -> String;

    /// Get a receiver for monitoring the client's connection state
    /// 
    /// The implementation must send a new state value to the watch channel whenever the
    /// connection state changes (e.g., from `Connecting` to `Connected`, or from `Connected`
    /// to `Disconnected`).
    fn get_state(&self) -> watch::Receiver<MqttConnectionState>;

    /// Subscribe to a topic with the specified QoS level
    /// 
    /// This function awaits until a SUBACK is received from the broker before returning.
    /// Messages received on this subscription will be sent to the provided channel.
    /// Returns a subscription identifier that will be set in the `subscription_id` field
    /// of all `MqttMessage`s received on this subscription.
    async fn subscribe(&mut self, topic: String, qos: message::QoS, tx: broadcast::Sender<MqttMessage>) -> Result<i32, MqttError>;

    /// Unsubscribe from a topic
    /// 
    /// This function awaits until an UNSUBACK is received from the broker before returning.
    /// The topic must be identical to the one used with the `subscribe()` method.
    async fn unsubscribe(&mut self, topic: String) -> Result<(), MqttError>;

    /// Disconnect from the MQTT broker
    async fn disconnect(&mut self) -> Result<(), MqttError>;

    /// Publish a message to the broker (awaits completion)
    /// 
    /// The function blocks according to the QoS level set in the message:
    /// - QoS 0: Blocks until the message is sent
    /// - QoS 1: Blocks until a PUBACK is received from the broker
    /// - QoS 2: Blocks until a PUBCOMP is received from the broker
    async fn publish(&mut self, message: MqttMessage) -> Result<(), MqttError>;

    /// Publish a message without waiting for completion (fire and forget)
    /// 
    /// This function returns as soon as the message is queued to be sent, without waiting
    /// for any acknowledgment from the broker.
    fn nowait_publish(&mut self, message: MqttMessage) -> Result<(), MqttError>;

    /// Start the MQTT client event loop
    /// 
    /// Application code must call this method before or after connecting to enable
    /// the client to process incoming and outgoing messages.
    async fn start(&mut self) -> Result<(), MqttError>;

    /// Cleanly stop the MQTT client
    async fn clean_stop(&mut self) -> Result<(), MqttError>;

    /// Force stop the MQTT client immediately
    async fn force_stop(&mut self) -> Result<(), MqttError>;

    /// Reconnect to the MQTT broker
    /// 
    /// Application code can call this to disconnect from the broker (if connected) and then
    /// reconnect. The `clean_start` parameter controls whether the new connection should start
    /// with a clean session (`true`) or resume the previous session (`false`).
    async fn reconnect(&mut self, clean_start: bool) -> Result<(), MqttError>;
}

// Re-export commonly used types
pub use message::QoS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_error_display() {
        let err = MqttError::ConnectionError("failed to connect".to_string());
        assert_eq!(err.to_string(), "Connection error: failed to connect");

        let err = MqttError::SubscriptionError("invalid topic".to_string());
        assert_eq!(err.to_string(), "Subscription error: invalid topic");

        let err = MqttError::UnsubscribeError("not subscribed".to_string());
        assert_eq!(err.to_string(), "Unsubscribe error: not subscribed");

        let err = MqttError::PublishError("publish failed".to_string());
        assert_eq!(err.to_string(), "Publish error: publish failed");

        let err = MqttError::DisconnectionError("disconnect failed".to_string());
        assert_eq!(err.to_string(), "Disconnection error: disconnect failed");

        let err = MqttError::InvalidTopic("bad topic".to_string());
        assert_eq!(err.to_string(), "Invalid topic: bad topic");

        let err = MqttError::InvalidQoS("bad qos".to_string());
        assert_eq!(err.to_string(), "Invalid QoS: bad qos");

        let err = MqttError::Other("something went wrong".to_string());
        assert_eq!(err.to_string(), "Error: something went wrong");
    }

    #[test]
    fn test_mqtt_error_is_error_trait() {
        let err = MqttError::ConnectionError("test".to_string());
        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_mqtt_error_clone() {
        let err1 = MqttError::PublishError("test error".to_string());
        let err2 = err1.clone();
        assert_eq!(err1.to_string(), err2.to_string());
    }

    // Mock implementation for testing trait signature
    struct MockMqttClient {
        client_id: String,
        last_will: Option<MqttMessage>,
        state_rx: watch::Receiver<MqttConnectionState>,
    }

    impl MockMqttClient {
        fn new() -> Self {
            let (tx, rx) = watch::channel(MqttConnectionState::Disconnected);
            drop(tx); // Drop the sender as we don't need it in the mock
            Self {
                client_id: "test-client".to_string(),
                last_will: None,
                state_rx: rx,
            }
        }
    }

    #[async_trait]
    impl MqttClient for MockMqttClient {
        async fn connect(&mut self, _uri: String) -> Result<(), MqttError> {
            Ok(())
        }

        fn set_last_will(&mut self, message: MqttMessage) {
            self.last_will = Some(message);
        }

        fn get_client_id(&self) -> String {
            self.client_id.clone()
        }

        fn get_state(&self) -> watch::Receiver<MqttConnectionState> {
            self.state_rx.clone()
        }

        async fn subscribe(&mut self, _topic: String, _qos: message::QoS, _tx: broadcast::Sender<MqttMessage>) -> Result<i32, MqttError> {
            Ok(1)
        }

        async fn unsubscribe(&mut self, _topic: String) -> Result<(), MqttError> {
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<(), MqttError> {
            Ok(())
        }

        async fn publish(&mut self, _message: MqttMessage) -> Result<(), MqttError> {
            Ok(())
        }

        fn nowait_publish(&mut self, _message: MqttMessage) -> Result<(), MqttError> {
            Ok(())
        }

        async fn start(&mut self) -> Result<(), MqttError> {
            Ok(())
        }

        async fn clean_stop(&mut self) -> Result<(), MqttError> {
            Ok(())
        }

        async fn force_stop(&mut self) -> Result<(), MqttError> {
            Ok(())
        }

        async fn reconnect(&mut self, _clean_start: bool) -> Result<(), MqttError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_client_connect() {
        let mut client = MockMqttClient::new();
        let lwt = MqttMessage::simple(
            "status/test".to_string(),
            message::QoS::AtLeastOnce,
            true,
            bytes::Bytes::from("offline"),
        );
        client.set_last_will(lwt);
        let result = client.connect("mqtt://localhost:1883".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_client_connect_without_lwt() {
        let mut client = MockMqttClient::new();
        let result = client.connect("mqtt://localhost:1883".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_client_get_client_id() {
        let client = MockMqttClient::new();
        assert_eq!(client.get_client_id(), "test-client");
    }

    #[tokio::test]
    async fn test_mock_client_subscribe() {
        let (tx, _rx) = broadcast::channel(10);
        let mut client = MockMqttClient::new();
        let result = client.subscribe("test/topic".to_string(), message::QoS::AtLeastOnce, tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_mock_client_publish() {
        let mut client = MockMqttClient::new();
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            message::QoS::AtMostOnce,
            false,
            bytes::Bytes::from("test"),
        );
        let result = client.publish(msg).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_client_nowait_publish() {
        let mut client = MockMqttClient::new();
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            message::QoS::AtMostOnce,
            false,
            bytes::Bytes::from("test"),
        );
        let result = client.nowait_publish(msg);
        assert!(result.is_ok());
    }
}
