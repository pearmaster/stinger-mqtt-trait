//! Mock MQTT client for testing purposes

use crate::{MqttClient, MqttConnectionState, MqttError, MqttPublishSuccess, message::{MqttMessage, QoS}};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, watch};

/// A mock MQTT client for testing purposes
/// 
/// This implementation stores published messages and allows simulating received messages
/// through registered subscriptions. It's designed for unit testing MQTT-based applications
/// without requiring an actual MQTT broker.
#[derive(Clone)]
pub struct MockClient {
    client_id: String,
    last_will: Option<MqttMessage>,
    state_tx: Arc<Mutex<watch::Sender<MqttConnectionState>>>,
    state_rx: watch::Receiver<MqttConnectionState>,
    /// Storage for published messages
    published_messages: Arc<Mutex<Vec<MqttMessage>>>,
    /// Storage for subscription senders
    subscriptions: Arc<Mutex<Vec<(String, broadcast::Sender<MqttMessage>)>>>,
}

impl MockClient {
    /// Create a new MockClient with the given client ID
    pub fn new(client_id: impl Into<String>) -> Self {
        let (state_tx, state_rx) = watch::channel(MqttConnectionState::Disconnected);
        Self {
            client_id: client_id.into(),
            last_will: None,
            state_tx: Arc::new(Mutex::new(state_tx)),
            state_rx,
            published_messages: Arc::new(Mutex::new(Vec::new())),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new MockClient with a default client ID
    pub fn new_default() -> Self {
        Self::new("mock-client")
    }

    /// Get the last published message, if any
    pub fn last_published_message(&self) -> Option<MqttMessage> {
        self.published_messages.lock().unwrap().last().cloned()
    }

    /// Get all published messages
    pub fn published_messages(&self) -> Vec<MqttMessage> {
        self.published_messages.lock().unwrap().clone()
    }

    /// Clear all published messages
    pub fn clear_published_messages(&mut self) {
        self.published_messages.lock().unwrap().clear();
    }

    /// Simulate receiving a message on a subscribed topic
    /// 
    /// This will send the message through any registered subscription senders
    /// that match the given topic. Returns the number of subscriptions that
    /// received the message.
    pub fn simulate_receive(&self, message: MqttMessage) -> Result<usize, MqttError> {
        let subscriptions = self.subscriptions.lock().unwrap();
        let mut sent_count = 0;

        for (topic, tx) in subscriptions.iter() {
            // Simple topic matching (exact match only, no wildcards)
            if topic == &message.topic {
                tx.send(message.clone())
                    .map_err(|e| MqttError::Other(format!("Failed to send message to subscription: {}", e)))?;
                sent_count += 1;
            }
        }

        Ok(sent_count)
    }

    /// Set the connection state (for testing state transitions)
    pub fn set_connection_state(&self, state: MqttConnectionState) -> Result<(), MqttError> {
        self.state_tx.lock().unwrap()
            .send(state)
            .map_err(|e| MqttError::Other(format!("Failed to send state update: {}", e)))
    }
}

#[async_trait]
impl MqttClient for MockClient {
    async fn connect(&mut self, _uri: String) -> Result<(), MqttError> {
        self.set_connection_state(MqttConnectionState::Connecting)?;
        self.set_connection_state(MqttConnectionState::Connected)?;
        Ok(())
    }

    fn set_last_will(&mut self, message: MqttMessage) {
        self.last_will = Some(message);
    }

    fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    fn get_state(&self) -> watch::Receiver<MqttConnectionState> {
        self.state_rx.clone()
    }

    async fn subscribe(
        &mut self,
        topic: String,
        _qos: QoS,
        tx: broadcast::Sender<MqttMessage>,
    ) -> Result<i32, MqttError> {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        let subscription_id = (subscriptions.len() + 1) as i32;
        subscriptions.push((topic, tx));
        Ok(subscription_id)
    }

    async fn unsubscribe(&mut self, topic: String) -> Result<(), MqttError> {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.retain(|(t, _)| t != &topic);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), MqttError> {
        self.set_connection_state(MqttConnectionState::Disconnecting)?;
        self.set_connection_state(MqttConnectionState::Disconnected)?;
        Ok(())
    }

    async fn publish(&mut self, message: MqttMessage) -> Result<MqttPublishSuccess, MqttError> {
        self.published_messages.lock().unwrap().push(message.clone());
        
        // Return appropriate success variant based on QoS
        match message.qos {
            QoS::AtMostOnce => Ok(MqttPublishSuccess::Sent),
            QoS::AtLeastOnce => Ok(MqttPublishSuccess::Acknowledged),
            QoS::ExactlyOnce => Ok(MqttPublishSuccess::Completed),
        }
    }

    fn nowait_publish(&mut self, message: MqttMessage) -> Result<MqttPublishSuccess, MqttError> {
        self.published_messages.lock().unwrap().push(message);
        Ok(MqttPublishSuccess::Queued)
    }

    async fn start(&mut self) -> Result<(), MqttError> {
        Ok(())
    }

    async fn clean_stop(&mut self) -> Result<(), MqttError> {
        self.set_connection_state(MqttConnectionState::Disconnected)?;
        Ok(())
    }

    async fn force_stop(&mut self) -> Result<(), MqttError> {
        self.set_connection_state(MqttConnectionState::Disconnected)?;
        Ok(())
    }

    async fn reconnect(&mut self, _clean_start: bool) -> Result<(), MqttError> {
        self.set_connection_state(MqttConnectionState::Disconnecting)?;
        self.set_connection_state(MqttConnectionState::Disconnected)?;
        self.set_connection_state(MqttConnectionState::Connecting)?;
        self.set_connection_state(MqttConnectionState::Connected)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_mock_client_creation() {
        let client = MockClient::new("test-client");
        assert_eq!(client.get_client_id(), "test-client");
        
        let client_default = MockClient::new_default();
        assert_eq!(client_default.get_client_id(), "mock-client");
    }

    #[tokio::test]
    async fn test_mock_client_connect() {
        let mut client = MockClient::new("test");
        let result = client.connect("mqtt://localhost:1883".to_string()).await;
        assert!(result.is_ok());
        
        let state = *client.get_state().borrow();
        assert_eq!(state, MqttConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_mock_client_publish() {
        let mut client = MockClient::new("test");
        
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtLeastOnce,
            false,
            Bytes::from("test payload"),
        );
        
        let result = client.publish(msg.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MqttPublishSuccess::Acknowledged);
        
        let last_msg = client.last_published_message();
        assert!(last_msg.is_some());
        let last_msg = last_msg.unwrap();
        assert_eq!(last_msg.topic, "test/topic");
        assert_eq!(last_msg.payload, Bytes::from("test payload"));
    }

    #[tokio::test]
    async fn test_mock_client_publish_qos_variants() {
        let mut client = MockClient::new("test");
        
        // QoS 0
        let msg0 = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtMostOnce,
            false,
            Bytes::from("qos0"),
        );
        let result = client.publish(msg0).await;
        assert_eq!(result.unwrap(), MqttPublishSuccess::Sent);
        
        // QoS 1
        let msg1 = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtLeastOnce,
            false,
            Bytes::from("qos1"),
        );
        let result = client.publish(msg1).await;
        assert_eq!(result.unwrap(), MqttPublishSuccess::Acknowledged);
        
        // QoS 2
        let msg2 = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::ExactlyOnce,
            false,
            Bytes::from("qos2"),
        );
        let result = client.publish(msg2).await;
        assert_eq!(result.unwrap(), MqttPublishSuccess::Completed);
    }

    #[tokio::test]
    async fn test_mock_client_nowait_publish() {
        let mut client = MockClient::new("test");
        
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtMostOnce,
            false,
            Bytes::from("test"),
        );
        
        let result = client.nowait_publish(msg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MqttPublishSuccess::Queued);
        
        assert!(client.last_published_message().is_some());
    }

    #[tokio::test]
    async fn test_mock_client_multiple_publishes() {
        let mut client = MockClient::new("test");
        
        for i in 0..5 {
            let msg = MqttMessage::simple(
                format!("test/topic/{}", i),
                QoS::AtLeastOnce,
                false,
                Bytes::from(format!("payload {}", i)),
            );
            client.publish(msg).await.unwrap();
        }
        
        let messages = client.published_messages();
        assert_eq!(messages.len(), 5);
        
        let last = client.last_published_message().unwrap();
        assert_eq!(last.topic, "test/topic/4");
        
        client.clear_published_messages();
        assert_eq!(client.published_messages().len(), 0);
        assert!(client.last_published_message().is_none());
    }

    #[tokio::test]
    async fn test_mock_client_subscribe_and_simulate_receive() {
        let mut client = MockClient::new("test");
        let (tx, mut rx) = broadcast::channel(10);
        
        let sub_id = client.subscribe("test/topic".to_string(), QoS::AtLeastOnce, tx).await;
        assert!(sub_id.is_ok());
        assert_eq!(sub_id.unwrap(), 1);
        
        // Simulate receiving a message
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtMostOnce,
            false,
            Bytes::from("incoming message"),
        );
        
        let sent_count = client.simulate_receive(msg.clone());
        assert!(sent_count.is_ok());
        assert_eq!(sent_count.unwrap(), 1);
        
        // Verify the message was received
        let received = rx.recv().await;
        assert!(received.is_ok());
        let received_msg = received.unwrap();
        assert_eq!(received_msg.topic, "test/topic");
        assert_eq!(received_msg.payload, Bytes::from("incoming message"));
    }

    #[tokio::test]
    async fn test_mock_client_unsubscribe() {
        let mut client = MockClient::new("test");
        let (tx, _rx) = broadcast::channel(10);
        
        client.subscribe("test/topic".to_string(), QoS::AtLeastOnce, tx).await.unwrap();
        
        // Unsubscribe
        let result = client.unsubscribe("test/topic".to_string()).await;
        assert!(result.is_ok());
        
        // Simulate receiving a message - should not be delivered
        let msg = MqttMessage::simple(
            "test/topic".to_string(),
            QoS::AtMostOnce,
            false,
            Bytes::from("incoming message"),
        );
        
        let sent_count = client.simulate_receive(msg);
        assert!(sent_count.is_ok());
        assert_eq!(sent_count.unwrap(), 0); // No subscriptions matched
    }

    #[tokio::test]
    async fn test_mock_client_disconnect() {
        let mut client = MockClient::new("test");
        client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
        
        let result = client.disconnect().await;
        assert!(result.is_ok());
        
        let state = *client.get_state().borrow();
        assert_eq!(state, MqttConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_mock_client_reconnect() {
        let mut client = MockClient::new("test");
        client.connect("mqtt://localhost:1883".to_string()).await.unwrap();
        
        let result = client.reconnect(true).await;
        assert!(result.is_ok());
        
        let state = *client.get_state().borrow();
        assert_eq!(state, MqttConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_mock_client_set_last_will() {
        let mut client = MockClient::new("test");
        
        let lwt = MqttMessage::simple(
            "status/test".to_string(),
            QoS::AtLeastOnce,
            true,
            Bytes::from("offline"),
        );
        
        client.set_last_will(lwt.clone());
        assert!(client.last_will.is_some());
        
        let stored_lwt = client.last_will.as_ref().unwrap();
        assert_eq!(stored_lwt.topic, "status/test");
    }
}
