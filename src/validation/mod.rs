//! Validation test suite for MqttClient implementations
//!
//! This module provides a comprehensive set of test functions that can be used to validate
//! any implementation of the `MqttClient` trait. Tests can be run against a mock broker or
//! a real MQTT broker instance (e.g., rumqttd).
//!
//! # Usage
//!
//! ```ignore
//! use stinger_mqtt_trait::validation::*;
//!
//! #[tokio::test]
//! async fn validate_my_client() {
//!     let mut client = MyMqttClient::new();
//!     test_connection_lifecycle(&mut client, "mqtt://localhost:1883").await;
//!     test_publish_qos_levels(&mut client).await;
//! }
//! ```
//!
//! ## Using with rumqttd
//!
//! ```ignore
//! use stinger_mqtt_trait::validation::{broker::TestBroker, run_full_validation_suite};
//!
//! #[tokio::test]
//! async fn test_with_real_broker() {
//!     let broker = TestBroker::start_default().await.unwrap();
//!     let mut client = MyMqttClient::new();
//!     
//!     run_full_validation_suite(&mut client, &broker.mqtt_uri()).await.unwrap();
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

/// Test basic connection lifecycle: connect -> verify state -> disconnect
pub async fn test_connection_lifecycle<C: MqttClient>(
    client: &mut C,
    uri: &str,
) -> Result<(), String> {
    // Test connect
    client
        .connect(uri.to_string())
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Verify state is Connected
    let state_rx = client.get_state();
    let current_state = *state_rx.borrow();
    if current_state != MqttConnectionState::Connected {
        return Err(format!(
            "Expected Connected state, got {:?}",
            current_state
        ));
    }

    // Test disconnect
    client
        .disconnect()
        .await
        .map_err(|e| format!("Failed to disconnect: {}", e))?;

    Ok(())
}

/// Test state transitions during connection lifecycle
pub async fn test_state_transitions<C: MqttClient>(
    client: &mut C,
    uri: &str,
) -> Result<(), String> {
    let state_rx = client.get_state();

    // Initial state should be Disconnected
    let initial_state = *state_rx.borrow();
    if initial_state != MqttConnectionState::Disconnected {
        return Err(format!(
            "Expected initial Disconnected state, got {:?}",
            initial_state
        ));
    }

    // Connect and wait for state change
    client
        .connect(uri.to_string())
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // State should now be Connected
    let connected_state = *state_rx.borrow();
    if connected_state != MqttConnectionState::Connected {
        return Err(format!(
            "Expected Connected state after connect, got {:?}",
            connected_state
        ));
    }

    // Disconnect
    client
        .disconnect()
        .await
        .map_err(|e| format!("Failed to disconnect: {}", e))?;

    Ok(())
}

/// Test publishing with QoS 0 (fire and forget)
pub async fn test_publish_qos0<C: MqttClient>(client: &mut C) -> Result<(), String> {
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
pub async fn test_publish_qos1<C: MqttClient>(client: &mut C) -> Result<(), String> {
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
pub async fn test_publish_qos2<C: MqttClient>(client: &mut C) -> Result<(), String> {
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
pub async fn test_publish_all_qos_levels<C: MqttClient>(client: &mut C) -> Result<(), String> {
    test_publish_qos0(client).await?;
    test_publish_qos1(client).await?;
    test_publish_qos2(client).await?;
    Ok(())
}

/// Test publish_nowait (fire and forget, no blocking)
pub async fn test_publish_nowait<C: MqttClient>(client: &mut C) -> Result<(), String> {
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
pub async fn test_subscribe<C: MqttClient>(client: &mut C) -> Result<u32, String> {
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
pub async fn test_unsubscribe<C: MqttClient>(client: &mut C, topic: &str) -> Result<(), String> {
    client
        .unsubscribe(topic.to_string())
        .await
        .map_err(|e| format!("Failed to unsubscribe: {}", e))?;

    Ok(())
}

/// Test full subscribe -> receive message -> unsubscribe cycle
pub async fn test_subscribe_receive_unsubscribe<C: MqttClient>(
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

/// Test setting Last Will message before connecting
pub async fn test_last_will_setup<C: MqttClient>(
    client: &mut C,
    uri: &str,
) -> Result<(), String> {
    // Set Last Will before connecting
    let lwt = MqttMessage::simple(
        "test/lwt".to_string(),
        message::QoS::AtLeastOnce,
        true,
        Bytes::from("offline"),
    );

    client.set_last_will(lwt);

    // Connect
    client
        .connect(uri.to_string())
        .await
        .map_err(|e| format!("Failed to connect with LWT: {}", e))?;

    Ok(())
}

/// Test reconnect with clean start
pub async fn test_reconnect_clean<C: MqttClient>(client: &mut C) -> Result<(), String> {
    client
        .reconnect(true)
        .await
        .map_err(|e| format!("Failed to reconnect with clean start: {}", e))?;

    // Verify state is Connected
    let state_rx = client.get_state();
    let current_state = *state_rx.borrow();
    if current_state != MqttConnectionState::Connected {
        return Err(format!(
            "Expected Connected state after reconnect, got {:?}",
            current_state
        ));
    }

    Ok(())
}

/// Test reconnect without clean start (resume session)
pub async fn test_reconnect_resume<C: MqttClient>(client: &mut C) -> Result<(), String> {
    client
        .reconnect(false)
        .await
        .map_err(|e| format!("Failed to reconnect with session resume: {}", e))?;

    // Verify state is Connected
    let state_rx = client.get_state();
    let current_state = *state_rx.borrow();
    if current_state != MqttConnectionState::Connected {
        return Err(format!(
            "Expected Connected state after reconnect, got {:?}",
            current_state
        ));
    }

    Ok(())
}

/// Test client ID retrieval
pub async fn test_get_client_id<C: MqttClient>(client: &C) -> Result<String, String> {
    let client_id = client.get_client_id();

    if client_id.is_empty() {
        return Err("Client ID is empty".to_string());
    }

    Ok(client_id)
}

/// Run the complete validation suite
pub async fn run_full_validation_suite<C: MqttClient>(
    client: &mut C,
    uri: &str,
) -> Result<(), String> {
    println!("Running full MqttClient validation suite...");

    println!("  Testing client ID...");
    let client_id = test_get_client_id(client).await?;
    println!("    ✓ Client ID: {}", client_id);

    println!("  Testing connection lifecycle...");
    test_connection_lifecycle(client, uri).await?;
    println!("    ✓ Connection lifecycle");

    println!("  Reconnecting for remaining tests...");
    client.connect(uri.to_string()).await.map_err(|e| e.to_string())?;

    println!("  Testing QoS 0 publish...");
    test_publish_qos0(client).await?;
    println!("    ✓ QoS 0 publish");

    println!("  Testing QoS 1 publish...");
    test_publish_qos1(client).await?;
    println!("    ✓ QoS 1 publish");

    println!("  Testing QoS 2 publish...");
    test_publish_qos2(client).await?;
    println!("    ✓ QoS 2 publish");

    println!("  Testing nowait publish...");
    test_publish_nowait(client).await?;
    println!("    ✓ No-wait publish");

    println!("  Testing subscribe...");
    let sub_id = test_subscribe(client).await?;
    println!("    ✓ Subscribe (ID: {})", sub_id);

    println!("  Testing unsubscribe...");
    test_unsubscribe(client, "test/topic").await?;
    println!("    ✓ Unsubscribe");

    println!("  Testing subscribe-receive-unsubscribe cycle...");
    test_subscribe_receive_unsubscribe(client).await?;
    println!("    ✓ Subscribe-receive-unsubscribe");

    println!("  Testing reconnect with clean start...");
    test_reconnect_clean(client).await?;
    println!("    ✓ Reconnect (clean)");

    println!("  Testing reconnect with session resume...");
    test_reconnect_resume(client).await?;
    println!("    ✓ Reconnect (resume)");

    println!("  Disconnecting...");
    client.disconnect().await.map_err(|e| e.to_string())?;
    println!("    ✓ Disconnect");

    println!("\n✓ All validation tests passed!");
    Ok(())
}
