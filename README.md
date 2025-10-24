# stinger-mqtt-trait

A Rust trait-based abstraction for MQTT clients, providing a clean interface for implementing MQTT functionality without being tied to a specific MQTT library.  It allows a 3rd party library to make use of an MQTT connection, but without requiring a specific MQTT client.

It is part of the "Stinger" suite of inter-process communication tools, but has no requirements on the suite and can easily be used elsewhere.

## Features

- **Trait design** - Libraries can implement `MqttClient` trait with any MQTT client
- **MQTT 5.0 support** - `MqttMessage` struct allows for providing MQTTv5 properties.
- **Multiple publish modes**: Libraries can provide two mechanisms for publishing messages:
  - `publish()` - Awaits completion based on QoS level
  - `nowait_publish()` - Fire-and-forget
- **Connection state monitoring** - Libraries can share the connection state through a watch channel.
- **Validation suite** - Optional (currently limited) test suite for implementations (feature: `validation`)

## Consuming-application design

Application code instantiates whatever MQTT connection object that implements the `MqttClient` trait.  It can then provide that connection object to a library or other utility that requires an MQTT connection and accepts the `MqttClient` trait.

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
