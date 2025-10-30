//! Mock MQTT pub/sub client for testing
//! 
//! This module provides a [`MockClient`] implementation of the [`Mqtt5PubSub`](crate::Mqtt5PubSub) trait
//! that can be used for testing pub/sub operations without requiring an actual MQTT broker.
//! 
//! # Features
//! 
//! - Stateless - assumes connection is always available
//! - Captures all published messages for later inspection
//! - Simulates receiving messages on subscribed topics
//! - Fully in-memory operation
//! 
//! # Example
//! 
//! ```
//! use stinger_mqtt_trait::{Mqtt5PubSub, mock::MockClient, MqttMessage, QoS};
//! use bytes::Bytes;
//! 
//! # #[tokio::main]
//! # async fn main() {
//! let mut client = MockClient::new("test-device");
//! assert_eq!(client.get_client_id(), "test-device");
//! 
//! let message = MqttMessage::simple(
//!     "sensor/temperature".to_string(),
//!     QoS::AtLeastOnce,
//!     false,
//!     Bytes::from("23.5"),
//! );
//! 
//! client.publish(message).await.unwrap();
//! 
//! // Retrieve the last published message
//! let last_msg = client.last_published_message().unwrap();
//! assert_eq!(last_msg.topic, "sensor/temperature");
//! # }
//! ```

mod client;

pub use client::MockClient;
