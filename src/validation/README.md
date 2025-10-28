# MqttClient Validation Suite

This module provides a comprehensive validation suite for testing implementations of the `MqttClient` trait.

## Features

- **Generic test functions**: Work with any `MqttClient` implementation
- **Comprehensive coverage**: Tests all trait methods and behaviors
- **Real broker support**: Includes utilities for running `rumqttd` during tests
- **Easy integration**: Simple to add to your test suite

## Usage

### Basic Validation

Test individual aspects of your client:

```rust
use stinger_mqtt_trait::validation::*;

#[tokio::test]
async fn test_my_client() {
    let mut client = MyMqttClient::new();
    let uri = "mqtt://localhost:1883";
    
    // Test connection
    test_connection_lifecycle(&mut client, uri).await.unwrap();
    
    // Test publishing
    test_publish_qos0(&mut client).await.unwrap();
    test_publish_qos1(&mut client).await.unwrap();
    test_publish_qos2(&mut client).await.unwrap();
}
```

### Full Validation Suite

Run all validation tests:

```rust
use stinger_mqtt_trait::validation::run_full_validation_suite;

#[tokio::test]
async fn full_validation() {
    let mut client = MyMqttClient::new();
    run_full_validation_suite(&mut client, "mqtt://localhost:1883")
        .await
        .unwrap();
}
```

### With Real Broker (rumqttd)

Test against a real MQTT broker:

```rust
use stinger_mqtt_trait::validation::{broker::TestBroker, run_full_validation_suite};

#[tokio::test]
async fn test_with_rumqttd() {
    // Start broker
    let broker = TestBroker::start_default().await.unwrap();
    
    // Test your client
    let mut client = MyMqttClient::new();
    run_full_validation_suite(&mut client, &broker.mqtt_uri())
        .await
        .unwrap();
    
    // Cleanup
    broker.stop().unwrap();
}
```

## Prerequisites for Broker Tests

To use the broker utilities with rumqttd:

```bash
cargo install rumqttd
```

## Available Test Functions

### Connection Tests
- `test_connection_lifecycle()` - Connect, verify state, disconnect
- `test_state_transitions()` - Verify state changes during lifecycle
- `test_reconnect_clean()` - Reconnect with clean session
- `test_reconnect_resume()` - Reconnect resuming previous session

### Publishing Tests
- `test_publish_qos0()` - Publish with QoS 0
- `test_publish_qos1()` - Publish with QoS 1 (wait for PUBACK)
- `test_publish_qos2()` - Publish with QoS 2 (wait for PUBCOMP)
- `test_publish_all_qos_levels()` - Test all QoS levels
- `test_publish_nowait()` - Fire-and-forget publish

### Subscription Tests
- `test_subscribe()` - Subscribe and verify subscription ID
- `test_unsubscribe()` - Unsubscribe from topic
- `test_subscribe_receive_unsubscribe()` - Full subscribe cycle with message receipt

### Other Tests
- `test_get_client_id()` - Verify client ID is set
- `test_last_will_setup()` - Set Last Will before connecting

## Custom Broker Configuration

### Unix Domain Socket (Default)

```rust
use stinger_mqtt_trait::validation::broker::{TestBroker, BrokerConfig};

// Default uses Unix socket with random name in /tmp
let broker = TestBroker::start_default().await.unwrap();
// URI will be something like:unix:///tmp/mqtt_test_1234567890.sock

// Custom Unix socket path
let config = BrokerConfig::unix("/tmp/my_mqtt.sock");
let broker = TestBroker::start(config).await.unwrap();
```

### TCP Socket

```rust
use stinger_mqtt_trait::validation::broker::{TestBroker, BrokerConfig};

// Use TCP instead of Unix socket
let config = BrokerConfig::tcp("127.0.0.1", 1883);
let broker = TestBroker::start(config).await.unwrap();

// Custom TCP port
let config = BrokerConfig::tcp("127.0.0.1", 1884);
let broker = TestBroker::start(config).await.unwrap();
```

Unix sockets are automatically created and cleaned up when the broker stops. Using Unix sockets by default provides better isolation for tests running in parallel.

## Witness Client - Verifying Published Messages

The `WitnessClient` creates its own `rumqttc::AsyncClient` internally and can subscribe to topics to verify that messages are actually reaching the broker.

### Basic Verification

```rust
use rumqttc::{MqttOptions, AsyncClient};
use stinger_mqtt_trait::validation::broker::BrokerTransport;
use stinger_mqtt_trait::validation::witness::{WitnessClient, verify_publish};
use stinger_mqtt_trait::message::QoS;
use bytes::Bytes;

#[tokio::test]
async fn test_publish_verification() {
    // Create publisher client
    let pub_opts = MqttOptions::new("publisher", "localhost", 1883);
    let (pub_client, mut pub_eventloop) = AsyncClient::new(pub_opts, 10);
    
    // Spawn task to poll publisher event loop
    tokio::spawn(async move {
        loop {
            let _ = pub_eventloop.poll().await;
        }
    });
    
    // Create witness client - it manages its own connection
    let transport = BrokerTransport::Tcp {
        host: "localhost".to_string(),
        port: 1883,
    };
    let witness = WitnessClient::new(transport);
    
    // Subscribe to topic
    witness.subscribe("test/topic", QoS::AtLeastOnce).await.unwrap();
    
    // Publish a message
    let payload = Bytes::from("hello");
    pub_client.publish("test/topic", rumqttc::QoS::AtLeastOnce, false, payload.clone())
        .await
        .unwrap();
    
    // Verify it was received
    witness.wait_for_payload("test/topic", &payload, Duration::from_secs(5))
        .await
        .unwrap();
}
```

### Using with TestBroker

The witness client works seamlessly with `TestBroker`:

```rust
use stinger_mqtt_trait::validation::broker::TestBroker;

let broker = TestBroker::start_default().await.unwrap();
let witness = WitnessClient::new(broker.config().transport.clone());

// Subscribe and verify messages...
```

### Using the Helper Function

For simpler testing, use `verify_publish`:

```rust
use stinger_mqtt_trait::validation::witness::verify_publish;

let payload = Bytes::from("data");
verify_publish(&pub_client, &witness, "test/topic", payload, QoS::AtLeastOnce)
    .await
    .unwrap();
```

### Testing Multiple Messages

```rust
// Create witness with Unix socket
let transport = BrokerTransport::Unix {
    path: "/tmp/mqtt_test.sock".into(),
};
let witness = WitnessClient::new(transport);

// Subscribe to wildcard
witness.subscribe("sensor/+", QoS::AtLeastOnce).await.unwrap();

// Publish multiple messages
for i in 1..=5 {
    let payload = Bytes::from(format!("value-{}", i));
    pub_client.publish(
        format!("sensor/{}", i),
        rumqttc::QoS::AtLeastOnce,
        false,
        payload
    ).await.unwrap();
}

// Wait for all messages (total count, not per-topic)
witness.wait_for_message_count(5, Duration::from_secs(5)).await.unwrap();

// Verify specific message received
let payload = Bytes::from("value-3");
assert!(witness.verify_message_received("sensor/3", &payload).is_ok());
```

### WitnessClient Methods

- `new(transport)` - Create a witness client from `BrokerTransport` (creates client internally)
- `client()` - Get reference to the underlying `rumqttc::AsyncClient`
- `subscribe(topic, qos)` - Subscribe to a topic
- `wait_for_message_count(count, timeout)` - Wait for N messages (across all topics)
- `wait_for_messages(topic, count, timeout)` - Wait for N messages on specific topic
- `wait_for_payload(topic, payload, timeout)` - Wait for specific payload
- `verify_message_received(topic, payload)` - Check if message with payload was received
- `message_count()` - Get count of captured messages
- `get_messages()` - Get all captured messages
- `get_messages_for_topic(topic)` - Get messages for specific topic
- `clear()` - Clear captured messages

## Return Values

All test functions return `Result<T, String>` where:

- `Ok(T)` indicates the test passed
- `Err(String)` contains a description of the failure

This makes it easy to use with `?` operator or `unwrap()` in tests.
