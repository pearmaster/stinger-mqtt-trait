//! Validation test suite for Mqtt5PubSub implementations
//!
//! This module provides a comprehensive set of test functions that can be used to validate
//! any implementation of the `Mqtt5PubSub` trait. These tests focus on pub/sub operations
//! (publish, subscribe, unsubscribe) and assume the underlying MQTT client is already
//! connected and managed by the application.
//!
//! # Usage
//!
//! ```ignore
//! use stinger_mqtt_trait::validation::*;
//!
//! #[tokio::test]
//! async fn validate_my_pubsub() {
//!     let mut client = MyMqtt5PubSubClient::new();
//!     // Application manages connection
//!     test_publish_qos0(&mut client).await;
//!     test_subscribe_receive_unsubscribe(&mut client).await;
//! }
//! ```
//!
//! ## Using with rumqttd
//!
//! ```ignore
//! use stinger_mqtt_trait::validation::{broker::TestBroker, run_full_pubsub_validation_suite};
//!
//! #[tokio::test]
//! async fn test_with_real_broker() {
//!     let broker = TestBroker::start_default().await.unwrap();
//!     let mut client = MyMqtt5PubSubClient::new();
//!     // Connect your client here (application responsibility)
//!     
//!     run_full_pubsub_validation_suite(&mut client).await.unwrap();
//!     
//!     broker.stop().unwrap();
//! }
//! ```
//!
//! ## Verifying Published Messages
//!
//! Use a `WitnessClient` to verify messages are actually published to the broker:
//!
//! ## Witness Client
//!
//! The `WitnessClient` now uses `rumqttc::AsyncClient` directly for message verification.
//! See the `witness` module documentation for examples.

pub mod broker;
pub mod witness;

use crate::*;
use bytes::Bytes;
use tokio::time::{timeout, Duration};

/// Test client ID retrieval
pub async fn test_get_client_id<C: Mqtt5PubSub>(client: &C) -> Result<String, String> {
    let client_id = client.get_client_id();

    if client_id.is_empty() {
        return Err("Client ID is empty".to_string());
    }

    Ok(client_id)
}

/// Test publishing with QoS 0 (fire and forget)
pub async fn test_publish_qos0<C: Mqtt5PubSub>(client: &mut C) -> Result<(), String> {
    let message = MqttMessage::simple(
        "test/qos0".to_string(),
        message::QoS::AtMostOnce,
        false,
        Bytes::from("QoS 0 test message"),
    );

    client
        .publish(message)
        .await
        .map_err(|e| format!("Failed to publish QoS 0: {}", e))?;

    Ok(())
}

/// Test publishing with QoS 1 (at least once)
pub async fn test_publish_qos1<C: Mqtt5PubSub>(client: &mut C) -> Result<(), String> {
    let message = MqttMessage::simple(
        "test/qos1".to_string(),
        message::QoS::AtLeastOnce,
        false,
        Bytes::from("QoS 1 test message"),
    );

    client
        .publish(message)
        .await
        .map_err(|e| format!("Failed to publish QoS 1: {}", e))?;

    Ok(())
}

/// Test publishing with QoS 2 (exactly once)
pub async fn test_publish_qos2<C: Mqtt5PubSub>(client: &mut C) -> Result<(), String> {
    let message = MqttMessage::simple(
        "test/qos2".to_string(),
        message::QoS::ExactlyOnce,
        false,
        Bytes::from("QoS 2 test message"),
    );

    client
        .publish(message)
        .await
        .map_err(|e| format!("Failed to publish QoS 2: {}", e))?;

    Ok(())
}

/// Test all QoS levels in sequence
pub async fn test_publish_all_qos_levels<C: Mqtt5PubSub>(client: &mut C) -> Result<(), String> {
    test_publish_qos0(client).await?;
    test_publish_qos1(client).await?;
    test_publish_qos2(client).await?;
    Ok(())
}

/// Test publish_nowait (fire and forget, no blocking)
pub async fn test_publish_nowait<C: Mqtt5PubSub>(client: &mut C) -> Result<(), String> {
    let message = MqttMessage::simple(
        "test/nowait".to_string(),
        message::QoS::AtMostOnce,
        false,
        Bytes::from("No-wait test message"),
    );

    client
        .publish_nowait(message)
        .map_err(|e| format!("Failed to publish_nowait: {}", e))?;

    Ok(())
}

/// Test subscribe and verify subscription ID is returned
pub async fn test_subscribe<C: Mqtt5PubSub>(client: &mut C) -> Result<u32, String> {
    let (tx, _rx) = tokio::sync::broadcast::channel(10);

    let sub_id = client
        .subscribe("test/topic".to_string(), message::QoS::AtLeastOnce, tx)
        .await
        .map_err(|e| format!("Failed to subscribe: {}", e))?;

    if sub_id <= 0 {
        return Err(format!(
            "Invalid subscription ID returned: {}",
            sub_id
        ));
    }

    Ok(sub_id)
}

/// Test unsubscribe from a topic
pub async fn test_unsubscribe<C: Mqtt5PubSub>(client: &mut C, topic: &str) -> Result<(), String> {
    client
        .unsubscribe(topic.to_string())
        .await
        .map_err(|e| format!("Failed to unsubscribe: {}", e))?;

    Ok(())
}

/// Test full subscribe -> receive message -> unsubscribe cycle
pub async fn test_subscribe_receive_unsubscribe<C: Mqtt5PubSub>(
    client: &mut C,
) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::broadcast::channel(10);
    let topic = "test/subscribe";

    // Subscribe
    let sub_id = client
        .subscribe(topic.to_string(), message::QoS::AtLeastOnce, tx)
        .await
        .map_err(|e| format!("Failed to subscribe: {}", e))?;

    // Publish a message to the topic
    let message = MqttMessage::simple(
        topic.to_string(),
        message::QoS::AtLeastOnce,
        false,
        Bytes::from("Test subscription message"),
    );

    client
        .publish(message)
        .await
        .map_err(|e| format!("Failed to publish: {}", e))?;

    // Try to receive the message (with timeout)
    let received = timeout(Duration::from_secs(5), rx.recv())
        .await
        .map_err(|_| "Timeout waiting for message".to_string())?
        .map_err(|e| format!("Failed to receive message: {}", e))?;

    // Verify subscription ID matches
    if received.subscription_id != Some(sub_id) {
        return Err(format!(
            "Subscription ID mismatch: expected {}, got {:?}",
            sub_id, received.subscription_id
        ));
    }

    // Verify topic matches
    if received.topic != topic {
        return Err(format!(
            "Topic mismatch: expected {}, got {}",
            topic, received.topic
        ));
    }

    // Unsubscribe
    client
        .unsubscribe(topic.to_string())
        .await
        .map_err(|e| format!("Failed to unsubscribe: {}", e))?;

    Ok(())
}

/// Run the complete pub/sub validation suite
pub async fn run_full_pubsub_validation_suite<C: Mqtt5PubSub>(
    client: &mut C,
) -> Result<(), String> {
    println!("Running full Mqtt5PubSub validation suite...");

    println!("  Testing client ID...");
    let client_id = test_get_client_id(client).await?;
    println!("    ✓ Client ID: {}", client_id);

    println!("  Testing QoS 0 publish...");
    test_publish_qos0(client).await?;
    println!("    ✓ QoS 0 publish");

    println!("  Testing QoS 1 publish...");
    test_publish_qos1(client).await?;
    println!("    ✓ QoS 1 publish");

    println!("  Testing QoS 2 publish...");
    test_publish_qos2(client).await?;
    println!("    ✓ QoS 2 publish");

    println!("  Testing publish_nowait...");
    test_publish_nowait(client).await?;
    println!("    ✓ publish_nowait");

    println!("  Testing subscribe...");
    let sub_id = test_subscribe(client).await?;
    println!("    ✓ Subscribe (ID: {})", sub_id);

    println!("  Testing unsubscribe...");
    test_unsubscribe(client, "test/topic").await?;
    println!("    ✓ Unsubscribe");

    println!("  Testing subscribe-receive-unsubscribe cycle...");
    test_subscribe_receive_unsubscribe(client).await?;
    println!("    ✓ Subscribe-receive-unsubscribe");

    println!("\n✓ All pub/sub validation tests passed!");
    Ok(())
}
