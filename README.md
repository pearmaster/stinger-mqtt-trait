# stinger-mqtt-trait

A Rust trait-based abstraction for MQTT clients, providing a clean interface for implementing MQTT functionality without being tied to a specific MQTT library.

## Features

- **Trait-based design** - Implement `MqttClient` trait with any MQTT library
- **MQTT 5.0 support** - Full support for MQTT 5.0 properties (content type, correlation data, user properties, etc.)
- **Flexible payload types** - Binary (Bytes) or serializable objects (JSON)
- **Last Will and Testament (LWT)** - Built-in support for presence detection and status updates
- **Multiple publish modes**:
  - `publish()` - Awaits completion based on QoS level
  - `nowait_publish()` - Fire-and-forget
- **Connection state monitoring** - Watch channel for connection state changes
- **Async/await support** - Built with `tokio` and `async-trait`
- **Validation suite** - Optional comprehensive test suite for implementations (feature: `validation`)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
stinger-mqtt-trait = "0.1"
```

To use the validation suite for testing your implementation:

```toml
[dev-dependencies]
stinger-mqtt-trait = { version = "0.1", features = ["validation"] }
```

### Creating Messages

```rust
use stinger_mqtt_trait::{MqttMessage, Payload, QoS};

// Simple message
let msg = MqttMessage::simple(
    "home/temperature".to_string(),
    QoS::AtLeastOnce,
    false,
    Payload::String("22.5".to_string()),
);

// With MQTT 5.0 properties using builder pattern
use stinger_mqtt_trait::MqttMessageBuilder;

let msg = MqttMessageBuilder::default()
    .topic("sensor/data")
    .qos(QoS::ExactlyOnce)
    .retain(false)
    .payload(Payload::from_serializable(&my_struct)?)
    .content_type(Some("application/json".to_string()))
    .response_topic(Some("sensor/response".to_string()))
    .user_properties(properties_map)
    .build()?;
```

### Implementing the Trait

```rust
use stinger_mqtt_trait::{MqttClient, MqttMessage, MqttError, LastWill, QoS};
use async_trait::async_trait;
use tokio::sync::{broadcast, oneshot};

struct MyMqttClient {
    // Your implementation fields
}

#[async_trait]
impl MqttClient for MyMqttClient {
    async fn connect(&mut self, uri: String, last_will: LastWill) -> Result<(), MqttError> {
        // Your connection logic
    }

    async fn publish(&mut self, message: MqttMessage) -> Result<(), MqttError> {
        // Your publish logic
    }

    // Implement other trait methods...
}
```

### Last Will and Testament

```rust
use stinger_mqtt_trait::{LastWill, Payload};
use std::time::Duration;

let lwt = LastWill::new(
    "device/status".to_string(),
    Payload::String("online".to_string()),
    Payload::String("offline".to_string()),
    Duration::from_secs(30),
);
```

## Quality of Service Levels

- `QoS::AtMostOnce` (0) - Fire and forget
- `QoS::AtLeastOnce` (1) - Acknowledged delivery
- `QoS::ExactlyOnce` (2) - Assured delivery

## Validation Suite

The `validation` feature provides a comprehensive test suite for validating `MqttClient` implementations:

```rust
use stinger_mqtt_trait::validation::{broker::TestBroker, run_full_validation_suite};

#[tokio::test]
async fn test_my_implementation() {
    // Start a test broker (requires rumqttd: cargo install rumqttd)
    let broker = TestBroker::start_default().await.unwrap();
    
    // Test your client implementation
    let mut client = MyMqttClient::new();
    run_full_validation_suite(&mut client, &broker.mqtt_uri())
        .await
        .unwrap();
    
    broker.stop().unwrap();
}
```

See `src/validation/README.md` for more details on the validation suite.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
