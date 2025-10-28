# stinger-mqtt-trait

A **minimal trait abstraction** for MQTT clients in Rust, designed to enable library code to work with any MQTT implementation without coupling to a specific client library.

It is part of the "Stinger" suite of inter-process communication tools, but has no requirements on the suite and can easily be used elsewhere.  [See documentation](https://docs.rs/stinger-mqtt-trait/latest/stinger_mqtt_trait/)

## Design Philosophy

**stinger-mqtt-trait is intentionally minimal.** It defines the essential operations that all MQTT clients must support, while deliberately avoiding imposing implementation details or advanced features that would constrain implementers.  Application code should implement a client instance with whatever concrete-class specific initializers, and then provide the client instance to something requiring the `MqttClient` trait.

### Why Minimal?

- **Maximum Flexibility**: Implementers can use any underlying MQTT library (rumqttc, paho-mqtt, custom embedded clients, etc.)
- **No Lock-in**: Libraries accepting `MqttClient` work with any implementation - swap MQTT backends without changing application code
- **Implementation Freedom**: Rich features (metrics, compression, batching, RPC patterns) belong in concrete implementations or consuming-application code, not the trait
- **Testability**: The minimal surface area makes mocking straightforward (see `mock_client` feature)

### What This Trait Is NOT

This is **not** a batteries-included MQTT framework. If you need:

- Extensive configuration options (connection timeouts, keep-alive tuning, TLS settings, etc.)
- Advanced features (automatic reconnection, circuit breakers, compression, metrics)
- Convenience helpers (JSON publish/subscribe, RPC patterns)
- Optimized batch operations

...these belong in **your concrete implementation** or wrapper libraries. The trait simply ensures your implementation can be used anywhere `MqttClient` is accepted.

## Features

- **Trait-based abstraction** - Define MQTT operations once, swap implementations freely
- **MQTT 5.0 ready** - `MqttMessage` struct supports MQTT v5 properties (content type, correlation data, response topic, user properties)
- **Flexible publish semantics** - Three distinct methods for different use cases:
  - `publish()` - Await until broker acknowledges (QoS-aware: sent/PUBACK/PUBCOMP)
  - `publish_nowait()` - Fire-and-forget, returns immediately after queuing (synchronous, no async overhead)
  - `publish_noblock()` - Returns oneshot receiver for concurrent operations
- **Channel-based subscriptions** - User-provided `broadcast::Sender` enables flexible message routing:
  - Multiple concurrent handlers via broadcast fan-out
  - Channels are more idiomatic in _async_ Rust code, it supports `Sync + Send`, and make it much easier to use with context than with callbacks which must have `'static` lifetimes in Rust.  
- **Connection state monitoring** - `watch::Receiver` for tracking connection lifecycle
- **Builder pattern for messages** - Ergonomic construction with compile-time field validation
- **Validation suite** - Optional test suite for implementation verification (feature: `validation`)
- **Mock client** - Built-in mock for unit testing (feature: `mock_client`)

## Design Decisions & Rationale

### Why `broadcast::Sender` for Subscriptions?

The `subscribe()` method requires a `broadcast::Sender<MqttMessage>` parameter.   While other channel types are viable options, `tokio::sync::broadcast` was selected because of the wide-support for `tokio` and allow for multiple consumers.  This design choice maximizes flexibility:

**Application controls the channel**: You create the broadcast channel with your desired capacity and lagging strategy, then pass the sender to `subscribe()`. This means:

- **Fan-out**: Clone receivers for multiple independent handlers
- **Adaptable**: Wrap with your preferred pattern (callbacks, mpsc conversion, custom routing)
- **Explicit**: Buffer sizes and overflow behavior are in your control, not hidden in the trait

### Why Three Publish Methods?

Each method serves a **semantically distinct** use case:

- **`publish()`** - "I need confirmation" - Most common case, blocks until QoS acknowledgment received
- **`publish_noblock()`** - "I need to do other work" - Returns immediately with oneshot receiver for concurrent workflows
- **`publish_nowait()`** - "Fire and forget" - **Synchronous** method with no async overhead, ideal for high-frequency telemetry or non-critical messages

These are not redundant - they represent fundamentally different control flow patterns and have distinctly different return types and conditions.

### Why URI String for Connection?

The `connect(uri: String)` signature is deliberately simple:

- **Trait minimalism**: Connection configuration varies wildly between MQTT libraries (rumqttc's `MqttOptions` vs paho's `CreateOptions` vs embedded clients)
- **Extensibility**: Implementations can (and should) provide rich constructors.

Concrete implementations have different connection abilities.  For example, rumqttc can connect to a Unix socket, whereas libmosquitto cannot.  Using a string for connection information allows for flexibility between the application and the MQTT client.  

### Why Separate `start()` / `stop()` Methods?

Different MQTT client libraries have different lifecycle models:

- Some auto-start event loops on `connect()`
- Some require explicit `start()` before connecting
- Some have separate "clean shutdown" vs "immediate shutdown" semantics

The trait accommodates both patterns by providing lifecycle hooks that implementations can use as needed or no-op. This flexibility enables wrapping diverse underlying libraries.

## Common Misconceptions

### "The trait should include connection configuration options"

**Why not**: Configuration is inherently implementation-specific. An embedded MQTT client and a full-featured async client have completely different configuration needs. Forcing a unified config struct would either:

- Be too minimal (missing options implementers need)
- Be too complex (forcing implementers to support features they don't have)

**The solution**: Implementations provide their own constructors with rich configuration. The trait just ensures there's a standard `connect(uri)` method for simple cases.

### "Subscriptions should support callbacks, not channels"

**Why channels are better for a trait**:

- **More flexible**: Broadcast channels can be converted to callbacks, but callbacks can't easily be converted to channels
- **Decoupled**: The trait doesn't dictate concurrency model - you spawn handlers however you want
- **Multiple consumers**: Broadcast allows fan-out; callbacks typically don't
- **Standard library**: Uses tokio's well-tested primitives rather than defining custom callback traits

If you prefer callbacks, write an adapter function.

### "The trait is missing essential features like metrics/batching/reconnection"

**Why they're not in the trait**:

- **Not universal**: Embedded clients may not have metrics; some protocols don't support batching efficiently
- **Implementation detail**: How you implement auto-reconnection varies wildly between clients
- **Optional**: Many use cases don't need these features

**Where they belong**: In your concrete implementation or as extension traits. The base trait ensures interoperability; implementations provide richness.

## When to Use This Trait

### ✅ Use stinger-mqtt-trait when

- **Writing library code** that needs MQTT but shouldn't force a specific client implementation
- **Building modular systems** where different components might use different MQTT backends
- **Testing** - swap real MQTT clients with mocks without changing code
- **Long-term maintenance** - avoid coupling to a specific MQTT library that might become unmaintained
- **Embedded systems** - implement the trait with minimal, specialized MQTT clients

### ❌ Don't use this trait when

- You're building an end application that will only ever use one MQTT client - just use that client directly
- You're working in a constrained environment where trait object overhead matters

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

## FAQ

### Q: Why doesn't the trait return the granted QoS from subscribe?

**A**: This is a valid point - MQTT brokers can grant a different QoS than requested.  A future version could return both the subscription ID and granted QoS, and this is an area for improvement.

### Q: Can I add my own methods to implementations?

**A**: Absolutely! The trait defines the minimum interface. Your implementation can (and should) add:

- Rich configuration in constructors
- Convenience methods (e.g., `publish_json`, `subscribe_many`)
- Advanced features (metrics, reconnection policies, etc.)
- Domain-specific helpers

### Q: What about MQTT v3.1.1 vs v5.0?

**A**: The trait is designed to work with both:

- `MqttMessage` includes v5.0 properties (optional to use)
- Implementations can support either protocol version
- V3.1.1 clients simply ignore v5 fields, although this isn't recommended without making it abundantly clear to the application developer (would break the swap-ability that using a trait provides).

### Q: How do I use the start/force_stop/clean_stop/connect/reconnect/disconnect methods?  What is the difference?

**A**: This is a good question, and a future version should clarify their usage or simplify the trait.

The start, force_stop, and clean_stop methods control the event loop that must be running to receive and send message.  Generally, you'd call `start` when starting your application, and a stop method when exiting your application.

The connect, disconnect, and reconnect methods refer to the actual MQTT connection to the broker.

## License

MIT


