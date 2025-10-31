use crate::{MqttMessage, MqttMessageBuilder, QoS};
use derive_builder::Builder;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

/// Data structure for online/offline status messages
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into, strip_option))]
pub struct OnlineData {
    #[builder(default = "chrono::Utc::now()")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub system_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub system_version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub client_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub client_version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub service_id: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub service_version: Option<String>,
    
    #[builder(default = "false")]
    pub online: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub description: Option<String>,
}

impl Default for OnlineData {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            system_id: None,
            system_version: None,
            client_id: None,
            client_version: None,
            service_id: None,
            service_version: None,
            online: false,
            name: None,
            description: None,
        }
    }
}

impl OnlineData {
    /// Serialize the OnlineData to JSON and return as bytes
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(Vec<u8>)` containing the JSON bytes, or an error if serialization fails.
    /// 
    /// # Example
    /// 
    /// ```
    /// use stinger_mqtt_trait::availability::OnlineData;
    /// 
    /// let data = OnlineData {
    ///     online: true,
    ///     system_id: Some("my-system".to_string()),
    ///     ..Default::default()
    /// };
    /// 
    /// let json_bytes = data.as_json_bytes().unwrap();
    /// assert!(!json_bytes.is_empty());
    /// ```
    pub fn as_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Message builder for online/offline status messages
#[derive(Debug, Clone)]
pub struct AvailabilityHelper {
    topic_parts: Vec<String>,
    pub data: Arc<Mutex<OnlineData>>,
}

impl AvailabilityHelper {
    /// Create an availability message for a system
    pub fn system_availability(system_id: String) -> Self {
        let data = Arc::new(Mutex::new(OnlineData::default()));
        
        Self {
            topic_parts: vec![system_id],
            data,
        }
    }

    /// Create an availability message for a client within a system
    pub fn client_availability(system_id: String, client_id: String) -> Self {
        let data = Arc::new(Mutex::new(OnlineData::default()));
        
        Self {
            topic_parts: vec![system_id, client_id],
            data,
        }
    }

    /// Create an availability message for a service within a client
    pub fn service_availability(system_id: String, client_id: String, service_id: String) -> Self {
        let data = Arc::new(Mutex::new(OnlineData::default()));
        
        Self {
            topic_parts: vec![system_id, client_id, service_id],
            data,
        }
    }

    /// Get the MQTT topic for this online message
    pub fn get_topic(&self) -> String {
        let joined = self.topic_parts.join("/");
        format!("system/{}", joined)
    }

    /// Build an MQTT message with the specified online status
    pub fn get_message(&mut self, online: bool) -> Result<MqttMessage, Box<dyn std::error::Error>> {
        // Lock the mutex and update the data
        let mut data = self.data.lock().unwrap();
        
        data.timestamp = chrono::Utc::now();
        data.online = online;
        
        if let Some(system_id) = self.topic_parts.get(0) {
            data.system_id = Some(system_id.clone());
        }
        if let Some(client_id) = self.topic_parts.get(1) {
            data.client_id = Some(client_id.clone());
        }
        if let Some(service_id) = self.topic_parts.get(2) {
            data.service_id = Some(service_id.clone());
        }
        
        let msg = MqttMessageBuilder::default()
            .topic(self.get_topic())
            .qos(QoS::AtLeastOnce)
            .retain(true)
            .object_payload(&*data)?
            .build()?;
        
        Ok(msg)
    }
}


/// This function publishes an availability message on a repeating interval.  It should probably be called from a tokio::task.
/// 
/// Note: This function requires tokio's `time` feature to be enabled.
#[cfg(feature = "availability_publish")]
pub async fn publish_online_availability_periodically<T: crate::Mqtt5PubSub + Send + 'static>(
    mut client: T,
    interval_secs: u64,
) {
    let interval_duration = std::time::Duration::from_secs(interval_secs);
    let mut interval = tokio::time::interval(interval_duration);
    let mut availability_helper = client.get_availability_helper();
    
    loop {
        interval.tick().await;

        match availability_helper.get_message(true) {
            Ok(mut msg) => {
                msg.message_expiry_interval = Some((interval_secs * 2) as u32);
                if let Err(e) = client.publish(msg).await {
                    eprintln!("Failed to publish availability message: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to build availability message: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_online_topic() {
        let msg = AvailabilityHelper::system_availability("sys1".to_string());
        assert_eq!(msg.get_topic(), "system/sys1");
    }

    #[test]
    fn test_client_online_topic() {
        let msg = AvailabilityHelper::client_availability("sys1".to_string(), "client1".to_string());
        assert_eq!(msg.get_topic(), "system/sys1/client1");
    }

    #[test]
    fn test_service_online_topic() {
        let msg = AvailabilityHelper::service_availability(
            "sys1".to_string(),
            "client1".to_string(),
            "svc1".to_string()
        );
        assert_eq!(msg.get_topic(), "system/sys1/client1/svc1");
    }

    #[test]
    fn test_get_message() {
        let mut msg = AvailabilityHelper::system_availability("sys1".to_string());
        let mqtt_msg = msg.get_message(true).unwrap();
        
        assert_eq!(mqtt_msg.topic, "system/sys1");
        assert_eq!(mqtt_msg.qos, QoS::AtLeastOnce);
        assert_eq!(mqtt_msg.retain, true);
        assert_eq!(mqtt_msg.content_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_online_data_fields() {
        let mut msg = AvailabilityHelper::service_availability(
            "sys1".to_string(),
            "client1".to_string(),
            "svc1".to_string()
        );
        
        let _mqtt_msg = msg.get_message(true).unwrap();
        
        // Verify the data structure contains the right IDs
        let data = msg.data.lock().unwrap();
        assert_eq!(data.system_id, Some("sys1".to_string()));
        assert_eq!(data.client_id, Some("client1".to_string()));
        assert_eq!(data.service_id, Some("svc1".to_string()));
        assert_eq!(data.online, true);
    }

    #[test]
    fn test_online_status_toggle() {
        let mut msg = AvailabilityHelper::system_availability("sys1".to_string());
        
        msg.get_message(true).unwrap();
        {
            let data = msg.data.lock().unwrap();
            assert_eq!(data.online, true);
        }
        
        msg.get_message(false).unwrap();
        {
            let data = msg.data.lock().unwrap();
            assert_eq!(data.online, false);
        }
    }

    #[test]
    fn test_online_data_builder() {
        let data = OnlineDataBuilder::default()
            .system_id("test-system")
            .system_version("1.0.0")
            .client_id("test-client")
            .client_version("2.0.0")
            .online(true)
            .name("Test System")
            .description("A test system description")
            .build()
            .unwrap();
        
        assert_eq!(data.system_id, Some("test-system".to_string()));
        assert_eq!(data.system_version, Some("1.0.0".to_string()));
        assert_eq!(data.client_id, Some("test-client".to_string()));
        assert_eq!(data.client_version, Some("2.0.0".to_string()));
        assert_eq!(data.online, true);
        assert_eq!(data.name, Some("Test System".to_string()));
        assert_eq!(data.description, Some("A test system description".to_string()));
    }

    #[test]
    fn test_online_data_builder_minimal() {
        // Test that builder works with minimal fields thanks to defaults
        let data = OnlineDataBuilder::default()
            .online(true)
            .build()
            .unwrap();
        
        assert_eq!(data.online, true);
        assert_eq!(data.system_id, None);
        assert_eq!(data.client_id, None);
        assert_eq!(data.service_id, None);
    }

    #[test]
    fn test_as_json_bytes() {
        let data = OnlineDataBuilder::default()
            .system_id("test-system")
            .client_id("test-client")
            .online(true)
            .name("Test Name")
            .build()
            .unwrap();
        
        let json_bytes = data.as_json_bytes().unwrap();
        
        // Verify we got bytes back
        assert!(!json_bytes.is_empty());
        
        // Verify it's valid JSON by deserializing it back
        let deserialized: OnlineData = serde_json::from_slice(&json_bytes).unwrap();
        assert_eq!(deserialized.system_id, Some("test-system".to_string()));
        assert_eq!(deserialized.client_id, Some("test-client".to_string()));
        assert_eq!(deserialized.online, true);
        assert_eq!(deserialized.name, Some("Test Name".to_string()));
    }

    #[test]
    fn test_as_json_bytes_minimal() {
        let data = OnlineData::default();
        let json_bytes = data.as_json_bytes().unwrap();
        
        // Even a minimal struct should serialize
        assert!(!json_bytes.is_empty());
        
        // Verify it can be deserialized back
        let deserialized: OnlineData = serde_json::from_slice(&json_bytes).unwrap();
        assert_eq!(deserialized.online, false);
    }
}
