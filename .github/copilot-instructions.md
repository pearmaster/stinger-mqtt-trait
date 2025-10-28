# Copilot Instructions for stinger-mqtt-trait

## Project Overview
`stinger-mqtt-trait` is a Rust library (2021 edition) providing a trait-based abstraction for MQTT clients. It defines the interface and data structures needed for MQTT client implementations without being tied to a specific MQTT library.

## Architecture

### Module Organization
- **Root module** (`src/lib.rs`): Contains the `MqttClient` trait and `MqttError` type
- **Message module** (`src/message.rs`): Contains `MqttMessage`, `Payload` enum, and `QoS` enum

### Core Components

**MqttClient Trait** - Async trait (using `async-trait` crate) defining MQTT operations:
- Connection lifecycle: `connect()`, `disconnect()`, `reconnect()`
- Control methods: `start()`, `clean_stop()`, `force_stop()`
- Subscription: `subscribe()` returns subscription ID, `unsubscribe()`
- Three publish variants:
  - `publish()` - awaits completion
  - `publish_noblock()` - returns `oneshot::Receiver<Result<(), MqttError>>` for async acknowledgment
  - `publish_nowait()` - fire-and-forget (not async)
- `receive_channel()` - returns `broadcast::Receiver<MqttMessage>` for incoming messages

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

**MqttError** - Custom error type with variants for each operation type

## Development Conventions

### When Implementing MqttClient
- All trait methods must be async except `publish_nowait()` and `receive_channel()`
- Use `#[async_trait]` on impl blocks
- Return `MqttError` variants matching the operation type
- The broadcast channel should be created with appropriate capacity for message buffering

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
- Should new error variants be added to `MqttError`?
- Does a new method belong in the trait or as a utility?
- For new message fields: should they be pub or provide accessor methods?
- Are there MQTT 5.0 properties missing that implementers might need?
