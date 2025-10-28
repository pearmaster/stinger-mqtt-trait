# stinger-mqtt-trait

A Rust trait-based abstraction for MQTT clients, providing a clean interface for implementing MQTT functionality without being tied to a specific MQTT library.  It allows a 3rd party library to make use of an MQTT connection, but without requiring a specific MQTT client.

It is part of the "Stinger" suite of inter-process communication tools, but has no requirements on the suite and can easily be used elsewhere.

[See documentation](https://docs.rs/stinger-mqtt-trait/latest/stinger_mqtt_trait/)

## Features

- **Trait design** - Libraries can implement `MqttClient` trait with any MQTT client
- **MQTT 5.0 support** - `MqttMessage` struct allows for providing MQTTv5 properties.
- **Multiple publish modes**: Libraries can provide two mechanisms for publishing messages:
  - `publish()` - Method call blocks until the correct broker acknowledgment is received for the published message.
  - `publish_nowait()` - Fire-and-forget.  Method returns as soon as the message queues to be sent.
  - `publish_noblock()` - Method returns a future as soon as the message is queued to be sent.  The future resolves when the correct broker acknowledgment is received.
- **Message Builder** - Instead of a more complicated array of publish methods, an `MqttMessage` object with **builder pattern** is provided.  
- **Connection state monitoring** - Libraries can share the connection state through a watch channel.
- **Validation suite** - Optional (currently limited, unverified) test suite for implementations (feature: `validation`)
- **Mock Client** - Optional (feature: `mock_client`) A very simple mock object implementing the `MqttClient` trait allows for libraries to use a mock client for unit testing.

## Consuming-application design

Application code instantiates whatever MQTT connection object that implements the `MqttClient` trait.  It can then provide that connection object to a library or other utility that requires an MQTT connection and accepts the `MqttClient` trait.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
stinger-mqtt-trait = "0.2"
```

To use the validation suite for testing your implementation:

```toml
[dev-dependencies]
stinger-mqtt-trait = { version = "0.2", features = ["validation"] }
```

### Creating Messages

```rust
use stinger_mqtt_trait::{MqttMessage, MqttMessageBuilder, QoS};

let msg = MqttMessageBuilder::default()
    .topic("sensor/data")
    .qos(QoS::ExactlyOnce)
    .retain(false)
    .payload(Payload::from_serializable(&my_struct)?)
    .content_type(Some("application/json".to_string()))
    .response_topic(Some("sensor/response".to_string()))
    .user_property("PropertyKey", "PropertyValue")
    .build()?;
```

Each consuming library will have their own style of messages, and I recommend the libraries simplify the builder pattern with static functions to create messages like this:

```rust
pub fn request_message<T: Serialize>(topic: &str, payload: &T, correlation_id: Uuid, response_topic: String) -> Result<MqttMessage> {
    MqttMessageBuilder::default()
        .topic(topic)
        .object_payload(payload)?
        .qos(QoS::ExactlyOnce)
        .retain(false)
        .correlation_data(Bytes::from(correlation_id.to_string()))
        .response_topic(response_topic)
        .build()?
}
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
    async fn connect(&mut self, uri: String) -> Result<(), MqttError> {
        // Your connection logic
    }

    async fn publish(&mut self, message: MqttMessage) -> Result<MqttPublishSuccess, MqttError> {
        // Your publish logic
    }

    // Implement other trait methods...
}
```

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


