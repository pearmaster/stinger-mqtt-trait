# stinger-mqtt-trait

A Rust trait-based abstraction for MQTT clients, providing a clean interface for implementing MQTT functionality without being tied to a specific MQTT library.

## Features

- **Trait-based design** - Implement `MqttClient` trait with any MQTT library
- **MQTT 5.0 support** - Full support for MQTT 5.0 properties (content type, correlation data, user properties, etc.)
- **Flexible payload types** - String, binary (Bytes), or serializable objects (JSON)
- **Last Will and Testament (LWT)** - Built-in support for presence detection and status updates
- **Multiple publish modes**:
  - `publish()` - Awaits completion
  - `future_publish()` - Returns a future for async acknowledgment
  - `nowait_publish()` - Fire-and-forget
- **Async/await support** - Built with `tokio` and `async-trait`

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
stinger-mqtt-trait = "0.1"
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
let msg = MqttMessage::new()
    .topic("sensor/data".to_string())
    .qos(QoS::ExactlyOnce)
    .retain(false)
    .payload(Payload::from_serializable(&my_struct)?)
    .content_type(Some("application/json".to_string()))
    .response_topic(Some("sensor/response".to_string()))
    .user_properties(properties_map)
    .build();
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

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
