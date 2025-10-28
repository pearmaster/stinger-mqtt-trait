//! Tests for the mock client using crate::validation

#[cfg(test)]
mod test_validation {
    use super::*;
    use crate::validation::{
        test_connection_lifecycle,
        test_publish_qos0,
        test_publish_qos1,
        test_publish_qos2,
        test_subscribe_receive_unsubscribe,
    test_publish_nowait,
        run_full_validation_suite,
    };
    use crate::mock::client::MockClient;
    use tokio;

    const TEST_URI: &str = "mock://localhost";

    #[tokio::test]
    async fn test_mock_client_connection_lifecycle() {
        let mut client = MockClient::new();
        test_connection_lifecycle(&mut client, TEST_URI)
            .await
            .expect("Connection lifecycle validation failed");
    }

    #[tokio::test]
    async fn test_mock_client_publish_qos0() {
        let mut client = MockClient::new();
        test_publish_qos0(&mut client)
            .await
            .expect("Publish QoS0 validation failed");
    }

    #[tokio::test]
    async fn test_mock_client_publish_qos1() {
        let mut client = MockClient::new();
        test_publish_qos1(&mut client)
            .await
            .expect("Publish QoS1 validation failed");
    }

    #[tokio::test]
    async fn test_mock_client_publish_qos2() {
        let mut client = MockClient::new();
        test_publish_qos2(&mut client)
            .await
            .expect("Publish QoS2 validation failed");
    }

    #[tokio::test]
    async fn test_mock_client_subscribe_receive_unsubscribe() {
        let mut client = MockClient::new();
        test_subscribe_receive_unsubscribe(&mut client)
            .await
            .expect("Subscribe/receive/unsubscribe validation failed");
    }

    #[tokio::test]
    async fn test_mock_client_publish_nowait() {
        let mut client = MockClient::new();
    test_publish_nowait(&mut client)
            .await
            .expect("Nowait publish validation failed");
    }

    #[tokio::test]
    async fn test_mock_client_full_validation_suite() {
        let mut client = MockClient::new();
        run_full_validation_suite(&mut client, TEST_URI)
            .await
            .expect("Full validation suite failed");
    }
}
