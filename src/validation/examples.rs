//! Example showing how to use the validation suite with a real MqttClient implementation
//!
//! This example demonstrates various ways to validate an MqttClient implementation
//! using the validation module.

#![allow(dead_code)]

use stinger_mqtt_trait::validation::*;
use stinger_mqtt_trait::*;

// Example: Testing individual functions
#[cfg(test)]
mod individual_tests {
    use super::*;

    // Note: Replace MockMqttClient with your actual implementation
    #[tokio::test]
    async fn test_connection() {
        let mut client = MockMqttClient::new();
        let result = test_connection_lifecycle(&mut client, "mqtt://localhost:1883").await;
        assert!(result.is_ok(), "Connection test failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_publishing() {
        let mut client = MockMqttClient::new();
        // Connect first
        client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
        
        // Test all QoS levels
        let result = test_publish_all_qos_levels(&mut client).await;
        assert!(result.is_ok(), "Publishing test failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_subscriptions() {
        let mut client = MockMqttClient::new();
        client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
        
        // Subscribe
        let sub_result = test_subscribe(&mut client).await;
        assert!(sub_result.is_ok(), "Subscribe failed: {:?}", sub_result);
        
        // Unsubscribe
        let unsub_result = test_unsubscribe(&mut client, "test/topic").await;
        assert!(unsub_result.is_ok(), "Unsubscribe failed: {:?}", unsub_result);
    }
}

// Example: Running full validation suite
#[cfg(test)]
mod full_suite_tests {
    use super::*;

    #[tokio::test]
    async fn run_full_suite() {
        let mut client = MockMqttClient::new();
        
        let result = run_full_validation_suite(&mut client, "mqtt://localhost:1883").await;
        
        match result {
            Ok(_) => println!("✓ All validation tests passed!"),
            Err(e) => panic!("Validation failed: {}", e),
        }
    }
}

// Example: Testing with rumqttd broker
#[cfg(test)]
mod broker_integration_tests {
    use super::*;
    use broker::TestBroker;

    #[tokio::test]
    #[ignore] // Only run if rumqttd is installed
    async fn test_with_real_broker() {
        // Start the broker
        let broker = TestBroker::start_default()
            .await
            .expect("Failed to start broker. Is rumqttd installed?");
        
        println!("Broker started at: {}", broker.mqtt_uri());
        
        // Create your client
        let mut client = MockMqttClient::new();
        
        // Run validation suite
        let result = run_full_validation_suite(&mut client, &broker.mqtt_uri()).await;
        
        // Stop the broker
        broker.stop().expect("Failed to stop broker");
        
        // Check results
        assert!(result.is_ok(), "Validation failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_custom_tcp_broker() {
        use broker::BrokerConfig;
        
        // Custom TCP configuration
        let config = BrokerConfig::tcp("127.0.0.1", 1884);
        
        let broker = TestBroker::start(config).await.unwrap();
        let mut client = MockMqttClient::new();
        
        // Your tests here
        test_connection_lifecycle(&mut client, &broker.mqtt_uri())
            .await
            .unwrap();
        
        broker.stop().unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_unix_socket_broker() {
        use broker::BrokerConfig;
        
        // Unix socket configuration
        let config = BrokerConfig::unix("/tmp/test_mqtt.sock");
        
        let broker = TestBroker::start(config).await.unwrap();
        println!("Broker listening on Unix socket: {}", broker.mqtt_uri());
        
        let mut client = MockMqttClient::new();
        
        // Your tests here
        test_connection_lifecycle(&mut client, &broker.mqtt_uri())
            .await
            .unwrap();
        
        broker.stop().unwrap();
    }
}

// Example: Custom validation workflow
#[cfg(test)]
mod custom_workflow_tests {
    use super::*;

    #[tokio::test]
    async fn custom_validation_workflow() {
        let mut client = MockMqttClient::new();
        let uri = "mqtt://localhost:1883";
        
        // Step 1: Verify client ID
        println!("Step 1: Checking client ID...");
        let client_id = test_get_client_id(&client).await.unwrap();
        println!("  Client ID: {}", client_id);
        
        // Step 2: Test connection with Last Will
        println!("Step 2: Testing connection with Last Will...");
        test_last_will_setup(&mut client, uri).await.unwrap();
        
        // Step 3: Test state transitions
        println!("Step 3: Testing state transitions...");
        client.disconnect().await.unwrap();
        test_state_transitions(&mut client, uri).await.unwrap();
        
        // Step 4: Test publishing
        println!("Step 4: Testing publishing...");
        test_publish_qos0(&mut client).await.unwrap();
        test_publish_qos1(&mut client).await.unwrap();
        test_publish_qos2(&mut client).await.unwrap();
        
        // Step 5: Test subscriptions with message receipt
        println!("Step 5: Testing subscription cycle...");
        test_subscribe_receive_unsubscribe(&mut client).await.unwrap();
        
        // Step 6: Test reconnection
        println!("Step 6: Testing reconnection...");
        test_reconnect_clean(&mut client).await.unwrap();
        test_reconnect_resume(&mut client).await.unwrap();
        
        println!("✓ Custom workflow completed successfully!");
    }
}

// Example: End-to-end validation with broker and witness client
// Note: WitnessClient now requires rumqttc::AsyncClient directly
// These examples would need to be rewritten to use rumqttc instead of the trait
#[cfg(test)]
mod witness_verification_tests {
    use super::*;
    use broker::TestBroker;
    use bytes::Bytes;

    #[tokio::test]
    #[ignore] // Only run if rumqttd is installed
    async fn test_publish_with_witness_verification() {
        // Start test broker
        let broker = TestBroker::start_default()
            .await
            .expect("Failed to start broker. Is rumqttd installed?");
        
        println!("Broker started at: {}", broker.mqtt_uri());
        
        // Create two clients - one to publish, one to witness
        let mut publisher = MockMqttClient::new();
        let mut subscriber = MockMqttClient::new();
        
        // Connect both
        publisher.connect(broker.mqtt_uri()).await.unwrap();
        subscriber.connect(broker.mqtt_uri()).await.unwrap();
        
        // Test QoS 0 with verification
        println!("Testing QoS 0 publish with verification...");
        let payload = Bytes::from("test qos0");
        test_publish_with_verification(
            &mut publisher,
            &mut subscriber,
            "test/qos0",
            payload,
            message::QoS::AtMostOnce,
        )
        .await
        .unwrap();
        println!("  ✓ QoS 0 verified");
        
        // Test QoS 1 with verification
        println!("Testing QoS 1 publish with verification...");
        let payload = Bytes::from("test qos1");
        test_publish_with_verification(
            &mut publisher,
            &mut subscriber,
            "test/qos1",
            payload,
            message::QoS::AtLeastOnce,
        )
        .await
        .unwrap();
        println!("  ✓ QoS 1 verified");
        
        // Test QoS 2 with verification
        println!("Testing QoS 2 publish with verification...");
        let payload = Bytes::from("test qos2");
        test_publish_with_verification(
            &mut publisher,
            &mut subscriber,
            "test/qos2",
            payload,
            message::QoS::ExactlyOnce,
        )
        .await
        .unwrap();
        println!("  ✓ QoS 2 verified");
        
        // Clean up
        publisher.disconnect().await.unwrap();
        subscriber.disconnect().await.unwrap();
        broker.stop().unwrap();
        
        println!("\n✓ All broker verification tests passed!");
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_multiple_topics_verification() {
        use witness;
        
        let broker = TestBroker::start_default().await.unwrap();
        let mut publisher = MockMqttClient::new();
        let mut subscriber = MockMqttClient::new();
        
        publisher.connect(broker.mqtt_uri()).await.unwrap();
        subscriber.connect(broker.mqtt_uri()).await.unwrap();
        
        // Create witness client
        let witness = witness::WitnessClient::new();
        
        // Subscribe to multiple topics
        witness.subscribe_with(&mut subscriber, "sensor/+".to_string(), message::QoS::AtLeastOnce)
            .await
            .unwrap();
        
        // Publish to various sensor topics
        let topics = vec!["sensor/temp", "sensor/humidity", "sensor/pressure"];
        for topic in &topics {
            let msg = MqttMessage::simple(
                topic.to_string(),
                message::QoS::AtLeastOnce,
                false,
                Bytes::from(format!("data-{}", topic)),
            );
            publisher.publish(msg).await.unwrap();
        }
        
        // Verify all messages received
        witness.wait_for_messages(3, std::time::Duration::from_secs(5))
            .await
            .unwrap();
        
        println!("✓ All {} sensor messages verified", topics.len());
        
        broker.stop().unwrap();
    }
}

// Mock implementation used in examples
// In real usage, replace this with your actual MqttClient implementation
use async_trait::async_trait;

struct MockMqttClient {
    client_id: String,
    state_rx: tokio::sync::watch::Receiver<MqttConnectionState>,
}

impl MockMqttClient {
    fn new() -> Self {
        let (tx, rx) = tokio::sync::watch::channel(MqttConnectionState::Disconnected);
        drop(tx);
        Self {
            client_id: "mock-client".to_string(),
            state_rx: rx,
        }
    }
}

#[async_trait]
impl MqttClient for MockMqttClient {
    async fn connect(&mut self, _uri: String) -> Result<(), MqttError> {
        Ok(())
    }

    fn set_last_will(&mut self, _message: MqttMessage) {}

    fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    fn get_state(&self) -> tokio::sync::watch::Receiver<MqttConnectionState> {
        self.state_rx.clone()
    }

    async fn subscribe(
        &mut self,
        _topic: String,
        _qos: message::QoS,
        _tx: tokio::sync::broadcast::Sender<MqttMessage>,
    ) -> Result<u32, MqttError> {
        Ok(1)
    }

    async fn unsubscribe(&mut self, _topic: String) -> Result<(), MqttError> {
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), MqttError> {
        Ok(())
    }

    async fn publish(&mut self, _message: MqttMessage) -> Result<(), MqttError> {
        Ok(())
    }

    fn nowait_publish(&mut self, _message: MqttMessage) -> Result<(), MqttError> {
        Ok(())
    }

    async fn start(&mut self) -> Result<(), MqttError> {
        Ok(())
    }

    async fn clean_stop(&mut self) -> Result<(), MqttError> {
        Ok(())
    }

    async fn force_stop(&mut self) -> Result<(), MqttError> {
        Ok(())
    }

    async fn reconnect(&mut self, _clean_start: bool) -> Result<(), MqttError> {
        Ok(())
    }
}
