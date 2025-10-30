# Copilot Instructions for stinger-mqtt-trait

## Project Overview
`stinger-mqtt-trait` is a Rust library (2021 edition) providing a trait-based abstraction for MQTT pub/sub operations. It defines the interface and data structures needed for MQTT pub/sub implementations without being tied to a specific MQTT library. **Key design principle: Your application code manages starting the MQTT client and connecting it to a broker; library code uses pub/sub features required by the trait.**

## Architecture

### Module Organization
- **Root module** (`src/lib.rs`): Contains the `Mqtt5PubSub` trait and `Mqtt5PubSubError` type
- **Message module** (`src/message.rs`): Contains `MqttMessage`, `Payload` enum, and `QoS` enum
- **Mock module** (`src/mock/`): Stateless mock client for testing (feature: `mock_client`)
- **Validation module** (`src/validation/`): Test suite for implementations (feature: `validation`)

### Core Components

**Mqtt5PubSub Trait** - Async trait (using `async-trait` crate) defining MQTT pub/sub operations:
- Client identification: `get_client_id()` returns the MQTT client ID
- State monitoring: `get_state()` returns `watch::Receiver<MqttConnectionState>`
- Subscription: `subscribe()` returns subscription ID, `unsubscribe()`
- Three publish variants:
  - `publish()` - awaits completion
  - `publish_noblock()` - returns `oneshot::Receiver<Result<MqttPublishSuccess, Mqtt5PubSubError>>` for async acknowledgment
  - `publish_nowait()` - fire-and-forget (not async)

**MqttMessage Struct** - MQTT 5.0 compliant message with:
- Public fields: `topic`, `qos`, `retain`, `payload`, `content_type`, `subscription_id`, `correlation_data`, `response_topic`, `user_properties`
- Uses `derive_builder` crate - construct via builder: `MqttMessageBuilder::default().topic(...).qos(...).build()?`
- Helper method `simple()` for basic messages without MQTT 5.0 properties
- Helper method `with_json_payload()` for creating messages with JSON serialized payloads

**Payload Enum** - Three variants for message content:
- `String(String)` - text payload
- `Bytes(Bytes)` - binary payload (uses `bytes` crate)
- `Serializable(Vec<u8>)` - serialized struct payload (use `Payload::from_serializable()` helper with `serde::Serialize` types)

**QoS Enum** - MQTT quality of service levels:
- `AtMostOnce` (0), `AtLeastOnce` (1), `ExactlyOnce` (2)

**Mqtt5PubSubError** - Custom error type with variants for pub/sub operations:
- `SubscriptionError`, `UnsubscribeError`, `PublishError`, `InvalidTopic`, `InvalidQoS`, `TimeoutError`, `Other`

**MqttConnectionState** - Enum for connection state monitoring:
- `Disconnected`, `Connecting`, `Connected`, `Disconnecting`

## Development Conventions

### When Implementing Mqtt5PubSub
- All trait methods must be async except `publish_nowait()`, `get_state()`, and `get_client_id()`
- Use `#[async_trait]` on impl blocks
- Return `Mqtt5PubSubError` variants matching the operation type
- The trait assumes the client is already connected (application responsibility)
- Implementations should wrap an actual MQTT client that handles connection management

### Message Construction
- Use builder pattern: `MqttMessageBuilder::default().topic("foo").qos(QoS::AtLeastOnce).retain(false).payload(payload).build()?`
- The builder returns `Result<MqttMessage, MqttMessageBuilderError>`, handle with `?` or `.unwrap()`
- Builder uses `#[builder(setter(into))]` so String fields accept `&str` automatically
- For simple cases: `MqttMessage::simple(topic, qos, retain, payload)`
- Always use fully qualified paths for QoS in the trait: `message::QoS` (not just `QoS`)

### Payload Handling
- For serializable structs: `Payload::from_serializable(&my_struct)?` (requires `serde::Serialize`)
- JSON serialization is handled via `serde_json` internally
- Extract payload with `.payload_as_string()` or `.payload_as_bytes()` - handle potential UTF-8 errors

## Dependencies
Core: `tokio` (sync features), `serde`, `serde_json`, `bytes`, `derive_builder`, `async-trait`

## Questions to Ask
When making changes to this library, pause and ask about:
- Should new error variants be added to `Mqtt5PubSubError`?
- Does a new method belong in the trait or as a utility?
- For new message fields: should they be pub or provide accessor methods?
- Are there MQTT 5.0 properties missing that implementers might need?
- Does this change respect the separation between application (connection management) and library (pub/sub operations)?
