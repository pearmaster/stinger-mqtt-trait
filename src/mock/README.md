# Mock MQTT Client

A mock implementation of the `MqttClient` trait for testing purposes.

## Overview

The `MockClient` provides a fully functional in-memory implementation of the `MqttClient` trait that can be used in unit tests without requiring an actual MQTT broker. It captures published messages and allows you to simulate receiving messages on subscribed topics.

## Features

- **Message Capture**: All published messages (via `publish()` or `publish_nowait()` or `publish_noblock()`) are stored and can be retrieved
- **Subscription Simulation**: Register broadcast channels for subscriptions and simulate incoming messages
- **State Management**: Properly tracks and updates connection state
- **No Broker Required**: Completely in-memory, perfect for unit testing

## Usage

### Basic Publishing and Retrieval

```rust
use stinger_mqtt_trait::{MqttClient, mock::MockClient, MqttMessage, QoS};
use bytes::Bytes;

#[tokio::test]
async fn test_publish() {
    let mut client = MockClient::new("test-client");
    client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
    
    let message = MqttMessage::simple(
        "sensor/temperature".to_string(),
        QoS::AtLeastOnce,
        false,
        Bytes::from("23.5"),
    );
    
    client.publish(message).await.unwrap();
    
    // Retrieve the last published message
    let last_msg = client.last_published_message().unwrap();
    assert_eq!(last_msg.topic, "sensor/temperature");
    assert_eq!(last_msg.payload, Bytes::from("23.5"));
}
```

### Simulating Received Messages

```rust
use tokio::sync::broadcast;

#[tokio::test]
async fn test_subscription() {
    let mut client = MockClient::new("test-client");
    let (tx, mut rx) = broadcast::channel(10);
    
    // Subscribe to a topic
    client.subscribe(
        "sensor/temperature".to_string(),
        QoS::AtLeastOnce,
        tx,
    ).await.unwrap();
    
    // Simulate receiving a message from the broker
    let incoming_msg = MqttMessage::simple(
        "sensor/temperature".to_string(),
        QoS::AtLeastOnce,
        false,
        Bytes::from("24.8"),
    );
    
    client.simulate_receive(incoming_msg).unwrap();
    
    // Receive the message from the channel
    let received_msg = rx.recv().await.unwrap();
    assert_eq!(received_msg.topic, "sensor/temperature");
}
```

### Inspecting Published Messages

```rust
#[tokio::test]
async fn test_multiple_publishes() {
    let mut client = MockClient::new("test-client");
    
    // Publish multiple messages
    for i in 1..=5 {
        let message = MqttMessage::simple(
            format!("sensor/reading/{}", i),
            QoS::AtLeastOnce,
            false,
            Bytes::from(format!("value-{}", i)),
        );
        client.publish(message).await.unwrap();
    }
    
    // Get all published messages
    let all_messages = client.published_messages();
    assert_eq!(all_messages.len(), 5);
    
    // Get just the last one
    let last = client.last_published_message().unwrap();
    assert_eq!(last.topic, "sensor/reading/5");
    
    // Clear for next test
    client.clear_published_messages();
}
```

## API Reference

### Creation

- `MockClient::new(client_id)` - Create a new mock client with a specific client ID
- `MockClient::new_default()` - Create a new mock client with default client ID "mock-client"

### Message Inspection

- `last_published_message()` - Returns `Option<MqttMessage>` with the most recently published message
- `published_messages()` - Returns `Vec<MqttMessage>` with all published messages
- `clear_published_messages()` - Clears the internal message history

### Simulation

- `simulate_receive(message)` - Simulates receiving a message on subscribed topics. Returns the number of subscriptions that received the message
- `set_connection_state(state)` - Manually set the connection state for testing state transitions

## Implementation Details

- Published messages are stored in order and can be retrieved multiple times
- The `subscribe()` method stores the broadcast sender and returns incrementing subscription IDs (1, 2, 3, ...)
- The `simulate_receive()` method performs simple exact topic matching (wildcards not supported)
- Connection state changes are properly sent through the watch channel returned by `get_state()`
- QoS levels are respected in the return values of `publish()` (Sent/Acknowledged/Completed)

## Thread Safety

The `MockClient` is designed to be used within single-threaded test contexts. While it uses `Arc<Mutex<>>` internally for shared state, it's not intended for heavy concurrent access across threads.

## Examples

See `examples/mock_usage.rs` for comprehensive usage examples, including:
- Basic publish and retrieve
- Subscribe and simulate receive
- Multiple publishes
- JSON payload serialization/deserialization

Run the examples with:
```bash
cargo test --example mock_usage -- --nocapture
```
