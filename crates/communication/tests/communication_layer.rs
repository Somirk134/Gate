use std::sync::Arc;

use gate_communication::{
    mock::{MockClient, MockTransport},
    transport::{Transport, TransportEndpoint, TransportState},
};
use gate_protocol::{Body, Command, Message, Metadata};

#[tokio::test]
async fn mock_transport_connect_send_disconnect_flow() {
    let transport = Arc::new(MockTransport::new());
    let client = MockClient::new(Arc::clone(&transport));

    client
        .connect(TransportEndpoint::Tcp {
            host: "127.0.0.1".to_owned(),
            port: 7000,
        })
        .await
        .expect("mock connect");

    assert_eq!(transport.state(), TransportState::Connected);

    let message = Message::request(Command::SystemHealth, Body::Empty, Metadata::default());
    client.send(message.clone()).await.expect("mock send");

    let outgoing = transport.pop_outgoing().expect("outgoing message");
    assert_eq!(outgoing.header.request_id, message.header.request_id);

    client.disconnect().await.expect("mock disconnect");
    assert_eq!(transport.state(), TransportState::Disconnected);
}
