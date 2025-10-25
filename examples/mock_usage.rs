//! Example demonstrating the use of MockClient for testing
//! 
//! Run this example with: `cargo test --example mock_usage -- --nocapture`

fn main() {
    println!("This example contains tests demonstrating MockClient usage.");
    println!("Run with: cargo test --example mock_usage -- --nocapture");
}

#[cfg(test)]
mod tests {
    use stinger_mqtt_trait::{MqttClient, mock::MockClient, MqttMessage, QoS};
    use bytes::Bytes;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn example_basic_publish_and_retrieve() {
        // Create a mock client
        let mut client = MockClient::new("test-client");
        
        // Connect (doesn't actually connect to a broker)
        client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
        
        // Publish a message
        let message = MqttMessage::simple(
            "sensor/temperature".to_string(),
            QoS::AtLeastOnce,
            false,
            Bytes::from("23.5"),
        );
        
        client.publish(message).await.unwrap();
        
        // Retrieve the last published message
        let last_msg = client.last_published_message().unwrap();
        assert_eq!(last_msg.topic, "sensor/temperature");
        assert_eq!(last_msg.payload, Bytes::from("23.5"));
        
        println!("✓ Published and retrieved message: topic={}, payload={}", 
                 last_msg.topic, String::from_utf8_lossy(&last_msg.payload));
    }

    #[tokio::test]
    async fn example_subscribe_and_simulate_receive() {
        // Create a mock client
        let mut client = MockClient::new("test-client");
        
        // Create a channel for receiving messages
        let (tx, mut rx) = broadcast::channel(10);
        
        // Subscribe to a topic
        let sub_id = client.subscribe(
            "sensor/temperature".to_string(),
            QoS::AtLeastOnce,
            tx,
        ).await.unwrap();
        
        println!("✓ Subscribed to sensor/temperature with subscription ID: {}", sub_id);
        
        // Simulate receiving a message from the broker
        let incoming_msg = MqttMessage::simple(
            "sensor/temperature".to_string(),
            QoS::AtLeastOnce,
            false,
            Bytes::from("24.8"),
        );
        
        let sent_count = client.simulate_receive(incoming_msg).unwrap();
        println!("✓ Simulated receive sent to {} subscription(s)", sent_count);
        
        // Receive the message from the channel
        let received_msg = rx.recv().await.unwrap();
        assert_eq!(received_msg.topic, "sensor/temperature");
        assert_eq!(received_msg.payload, Bytes::from("24.8"));
        
        println!("✓ Received message: topic={}, payload={}", 
                 received_msg.topic, String::from_utf8_lossy(&received_msg.payload));
    }

    #[tokio::test]
    async fn example_multiple_publishes() {
        let mut client = MockClient::new("test-client");
        
        // Publish multiple messages
        for i in 1..=5 {
            let message = MqttMessage::simple(
                format!("sensor/reading/{}", i),
                QoS::AtLeastOnce,
                false,
                Bytes::from(format!("value-{}", i)),
            );
            client.publish(message).await.unwrap();
        }
        
        // Get all published messages
        let all_messages = client.published_messages();
        assert_eq!(all_messages.len(), 5);
        
        println!("✓ Published {} messages", all_messages.len());
        for (i, msg) in all_messages.iter().enumerate() {
            println!("  Message {}: topic={}, payload={}", 
                     i + 1, msg.topic, String::from_utf8_lossy(&msg.payload));
        }
        
        // Get the last one
        let last = client.last_published_message().unwrap();
        assert_eq!(last.topic, "sensor/reading/5");
        
        println!("✓ Last message: topic={}", last.topic);
    }

    #[tokio::test]
    async fn example_with_json_payload() {
        use serde::{Serialize, Deserialize};
        
        #[derive(Serialize, Deserialize, Debug)]
        struct SensorReading {
            temperature: f32,
            humidity: i32,
            timestamp: u64,
        }
        
        let mut client = MockClient::new("sensor-client");
        
        // Create a message with JSON payload
        let reading = SensorReading {
            temperature: 23.5,
            humidity: 65,
            timestamp: 1234567890,
        };
        
        use stinger_mqtt_trait::MqttMessageBuilder;
        let message = MqttMessageBuilder::default()
            .topic("sensor/data")
            .qos(QoS::AtLeastOnce)
            .retain(false)
            .object_payload(&reading)
            .unwrap()
            .build()
            .unwrap();
        
        client.publish(message).await.unwrap();
        
        // Retrieve and deserialize
        let last_msg = client.last_published_message().unwrap();
        let deserialized: SensorReading = serde_json::from_slice(&last_msg.payload).unwrap();
        
        assert_eq!(deserialized.temperature, 23.5);
        assert_eq!(deserialized.humidity, 65);
        
        println!("✓ Published and retrieved JSON message: {:?}", deserialized);
    }
}
