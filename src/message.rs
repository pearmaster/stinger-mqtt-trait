use bytes::Bytes;
use builder_pattern::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// Quality of Service levels for MQTT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QoS {
    /// At most once delivery (Fire and forget)
    AtMostOnce = 0,
    /// At least once delivery (Acknowledged delivery)
    AtLeastOnce = 1,
    /// Exactly once delivery (Assured delivery)
    ExactlyOnce = 2,
}

/// MQTT message structure with support for MQTT 5.0 properties
#[derive(Debug, Clone, Builder)]
pub struct MqttMessage {
    /// Topic to publish to or received from
    pub topic: String,
    
    /// Quality of Service level
    pub qos: QoS,
    
    /// Retain flag - if true, the message will be retained by the broker
    pub retain: bool,
    
    /// Message payload as bytes
    pub payload: Bytes,
    
    /// Content type (MQTT 5.0 property)
    pub content_type: Option<String>,
    
    /// Subscription identifier (MQTT 5.0 property)
    pub subscription_id: Option<i32>,
    
    /// Correlation data for request/response pattern (MQTT 5.0 property)
    pub correlation_data: Option<Bytes>,
    
    /// Response topic for request/response pattern (MQTT 5.0 property)
    pub response_topic: Option<String>,
    
    /// User properties - custom key-value pairs (MQTT 5.0 property)
    pub user_properties: HashMap<String, String>,
}

impl MqttMessage {
    /// Create a new MQTT message with required fields
    pub fn simple(topic: String, qos: QoS, retain: bool, payload: Bytes) -> Self {
        Self {
            topic,
            qos,
            retain,
            payload,
            content_type: None,
            subscription_id: None,
            correlation_data: None,
            response_topic: None,
            user_properties: HashMap::new(),
        }
    }

    /// Create a message with JSON payload from a serializable object
    pub fn with_json_payload<T: Serialize>(
        topic: String,
        qos: QoS,
        retain: bool,
        payload: &T,
    ) -> Result<Self, serde_json::Error> {
        let json_bytes = serde_json::to_vec(payload)?;
        Ok(Self {
            topic,
            qos,
            retain,
            payload: Bytes::from(json_bytes),
            content_type: Some("application/json".to_string()),
            subscription_id: None,
            correlation_data: None,
            response_topic: None,
            user_properties: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_values() {
        assert_eq!(QoS::AtMostOnce as i32, 0);
        assert_eq!(QoS::AtLeastOnce as i32, 1);
        assert_eq!(QoS::ExactlyOnce as i32, 2);
    }

    #[test]
    fn test_mqtt_message_simple() {
        let payload = Bytes::from("test");
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtLeastOnce,
            false,
            payload,
        );

        assert_eq!(msg.topic, "test/topic");
        assert_eq!(msg.qos as i32, 1);
        assert_eq!(msg.retain, false);
        assert_eq!(msg.payload, Bytes::from("test"));
        assert!(msg.content_type.is_none());
        assert!(msg.subscription_id.is_none());
        assert!(msg.user_properties.is_empty());
    }

    #[test]
    fn test_mqtt_message_builder() {
        let payload = Bytes::from("builder test");
        let mut user_props = HashMap::new();
        user_props.insert("key1".to_string(), "value1".to_string());

        let msg = MqttMessage::new()
            .topic("builder/topic".to_string())
            .qos(QoS::ExactlyOnce)
            .retain(true)
            .payload(payload)
            .content_type(Some("text/plain".to_string()))
            .subscription_id(Some(123))
            .response_topic(Some("response/topic".to_string()))
            .correlation_data(Some(Bytes::from(vec![1, 2, 3])))
            .user_properties(user_props)
            .build();

        assert_eq!(msg.topic, "builder/topic");
        assert_eq!(msg.qos as i32, 2);
        assert_eq!(msg.retain, true);
        assert_eq!(msg.payload, Bytes::from("builder test"));
        assert_eq!(msg.content_type, Some("text/plain".to_string()));
        assert_eq!(msg.subscription_id, Some(123));
        assert_eq!(msg.response_topic, Some("response/topic".to_string()));
        assert_eq!(msg.correlation_data, Some(Bytes::from(vec![1, 2, 3])));
        assert_eq!(msg.user_properties.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_mqtt_message_with_mqtt5_properties() {
        let payload = Bytes::from(vec![0xFF, 0xAA]);
        let msg = MqttMessage::new()
            .topic("mqtt5/test".to_string())
            .qos(QoS::AtLeastOnce)
            .retain(false)
            .payload(payload)
            .content_type(Some("application/octet-stream".to_string()))
            .subscription_id(Some(999))
            .correlation_data(None)
            .response_topic(None)
            .user_properties(HashMap::new())
            .build();

        assert_eq!(msg.content_type, Some("application/octet-stream".to_string()));
        assert_eq!(msg.subscription_id, Some(999));
        assert_eq!(msg.payload, Bytes::from(vec![0xFF, 0xAA]));
    }

    #[test]
    fn test_mqtt_message_bytes_payload() {
        let data = vec![1, 2, 3, 4, 5];
        let msg = MqttMessage::simple(
            "test/binary".to_string(),
            QoS::AtMostOnce,
            false,
            Bytes::from(data.clone()),
        );

        assert_eq!(msg.payload, Bytes::from(data));
    }

    #[test]
    fn test_mqtt_message_simple_string() {
        let msg = MqttMessage::simple(
            "test/string".to_string(),
            QoS::AtLeastOnce,
            false,
            Bytes::from("Hello, MQTT!"),
        );

        assert_eq!(msg.payload, Bytes::from("Hello, MQTT!"));
        assert_eq!(String::from_utf8(msg.payload.to_vec()).unwrap(), "Hello, MQTT!");
    }

    #[test]
    fn test_mqtt_message_string_payload() {
        let msg = MqttMessage::simple(
            "test/str".to_string(),
            QoS::ExactlyOnce,
            true,
            Bytes::from("Test message"),
        );

        assert_eq!(msg.payload, Bytes::from("Test message"));
    }

    #[test]
    fn test_mqtt_message_with_json_payload() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct TestData {
            temperature: f32,
            humidity: i32,
            location: String,
        }

        let data = TestData {
            temperature: 23.5,
            humidity: 65,
            location: "Living Room".to_string(),
        };

        let msg = MqttMessage::with_json_payload(
            "sensor/data".to_string(),
            QoS::AtLeastOnce,
            false,
            &data,
        ).unwrap();

        // Verify it's valid JSON containing our data
        let json_str = String::from_utf8(msg.payload.to_vec()).unwrap();
        assert!(json_str.contains("23.5"));
        assert!(json_str.contains("65"));
        assert!(json_str.contains("Living Room"));
        
        // Verify content_type was automatically set to application/json
        assert_eq!(msg.content_type, Some("application/json".to_string()));
    }
}
