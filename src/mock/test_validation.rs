//! Tests for the mock pub/sub client using crate::validation

#[cfg(test)]
mod test_validation {
    use crate::validation::{
        test_publish_qos0,
        test_publish_qos1,
        test_publish_qos2,
        test_subscribe_receive_unsubscribe,
        test_publish_nowait,
        run_full_pubsub_validation_suite,
    };
    use crate::mock::client::MockClient;
    use tokio;

    #[tokio::test]
    async fn test_mock_pubsub_publish_qos0() {
        let mut client = MockClient::new_default();
        test_publish_qos0(&mut client)
            .await
            .expect("Publish QoS0 validation failed");
    }

    #[tokio::test]
    async fn test_mock_pubsub_publish_qos1() {
        let mut client = MockClient::new_default();
        test_publish_qos1(&mut client)
            .await
            .expect("Publish QoS1 validation failed");
    }

    #[tokio::test]
    async fn test_mock_pubsub_publish_qos2() {
        let mut client = MockClient::new_default();
        test_publish_qos2(&mut client)
            .await
            .expect("Publish QoS2 validation failed");
    }

    #[tokio::test]
    async fn test_mock_pubsub_subscribe_receive_unsubscribe() {
        let mut client = MockClient::new_default();
        test_subscribe_receive_unsubscribe(&mut client)
            .await
            .expect("Subscribe/receive/unsubscribe validation failed");
    }

    #[tokio::test]
    async fn test_mock_pubsub_publish_nowait() {
        let mut client = MockClient::new_default();
        test_publish_nowait(&mut client)
            .await
            .expect("publish_nowait validation failed");
    }

    #[tokio::test]
    async fn test_mock_pubsub_full_validation_suite() {
        let mut client = MockClient::new_default();
        run_full_pubsub_validation_suite(&mut client)
            .await
            .expect("Full pub/sub validation suite failed");
    }
}