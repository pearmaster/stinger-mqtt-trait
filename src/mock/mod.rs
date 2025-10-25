//! Mock MQTT client for testing
//! 
//! This module provides a [`MockClient`] implementation of the [`MqttClient`](crate::MqttClient) trait
//! that can be used for testing without requiring an actual MQTT broker.
//! 
//! # Features
//! 
//! - Captures all published messages for later inspection
//! - Simulates receiving messages on subscribed topics
//! - Tracks connection state changes
//! - Fully in-memory operation
//! 
//! # Example
//! 
//! ```
//! use stinger_mqtt_trait::{MqttClient, mock::MockClient, MqttMessage, QoS};
//! use bytes::Bytes;
//! 
//! # #[tokio::main]
//! # async fn main() {
//! let mut client = MockClient::new("test-client");
//! client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
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
