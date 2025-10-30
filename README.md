# stinger-mqtt-trait

A Rust trait-based abstraction for MQTT pub/sub operations, providing a clean interface for publishing and subscribing to MQTT topics without being tied to a specific MQTT library. It allows libraries to perform MQTT pub/sub operations using a client connection managed by the application.

It is part of the "Stinger" suite of inter-process communication tools, but has no requirements on the suite and can easily be used elsewhere.

## Features

- **Trait design** - Libraries can implement `Mqtt5PubSub` trait with any MQTT client
- **MQTT 5.0 support** - `MqttMessage` struct allows for providing MQTTv5 properties
- **Multiple publish mechanisms**: Three mechanisms for publishing messages:
  - `publish()` - Awaits completion based on QoS level
  - `publish_noblock()` - Returns oneshot channel for async acknowledgment
  - `publish_nowait()` - Fire-and-forget
- **Connection state monitoring** - Libraries can monitor connection state through a watch channel
- **Mock client** - Stateless mock implementation for testing (feature: `mock_client`)
- **Validation suite** - Optional test suite for implementations (feature: `validation`)

## Design Philosophy

**Application manages connections, libraries use pub/sub:**
- Application code is responsible for managing the MQTT client connection lifecycle (connecting, disconnecting, reconnecting, etc.)
- Libraries implement `Mqtt5PubSub` to provide pub/sub operations on an already-connected client
- This separation of concerns allows libraries to focus on their domain logic while the application handles connection management

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
use stinger_mqtt_trait::{Mqtt5PubSub, MqttMessage, Mqtt5PubSubError, MqttConnectionState, QoS};
use async_trait::async_trait;
use tokio::sync::{broadcast, oneshot, watch};

struct MyMqtt5PubSubClient {
    // Your implementation fields (wrapping an actual MQTT client)
}

#[async_trait]
impl Mqtt5PubSub for MyMqtt5PubSubClient {
    fn get_client_id(&self) -> String {
        // Return the MQTT client ID
    }

    fn get_state(&self) -> watch::Receiver<MqttConnectionState> {
        // Return watch receiver for connection state monitoring
    }

    async fn subscribe(
        &mut self,
        topic: String,
        qos: QoS,
        tx: broadcast::Sender<MqttMessage>
    ) -> Result<u32, Mqtt5PubSubError> {
        // Your subscribe logic, return subscription ID
    }

    async fn unsubscribe(&mut self, topic: String) -> Result<(), Mqtt5PubSubError> {
        // Your unsubscribe logic
    }

    async fn publish(&mut self, message: MqttMessage) -> Result<MqttPublishSuccess, Mqtt5PubSubError> {
        // Your publish logic (awaits completion)
    }

    async fn publish_noblock(&mut self, message: MqttMessage) 
        -> oneshot::Receiver<Result<MqttPublishSuccess, Mqtt5PubSubError>> {
        // Your non-blocking publish logic
    }

    fn publish_nowait(&mut self, message: MqttMessage) -> Result<MqttPublishSuccess, Mqtt5PubSubError> {
        // Your fire-and-forget publish logic
    }
}
```

## Quality of Service Levels

- `QoS::AtMostOnce` (0) - Fire and forget
- `QoS::AtLeastOnce` (1) - Acknowledged delivery
- `QoS::ExactlyOnce` (2) - Assured delivery

## Validation Suite

The `validation` feature provides a comprehensive test suite for validating `Mqtt5PubSub` implementations:

```rust
use stinger_mqtt_trait::validation::run_full_pubsub_validation_suite;

#[tokio::test]
async fn test_my_implementation() {
    // Application connects the client first
    let mut client = MyMqtt5PubSubClient::new();
    // ... connect your client here ...
    
    // Test pub/sub operations
    run_full_pubsub_validation_suite(&mut client)
        .await
        .unwrap();
}
```

See `src/validation/README.md` for more details on the validation suite.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
