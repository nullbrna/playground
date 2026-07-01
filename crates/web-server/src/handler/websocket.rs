use axum::extract::WebSocketUpgrade;
use axum::extract::ws::Message;
use axum::extract::ws::Utf8Bytes;
use axum::extract::ws::WebSocket;
use axum::response::IntoResponse;
use futures_util::StreamExt;

/// Default response for a "text" message.
const TEXT_REPLY: &str = "Hello, world!";

pub async fn core(upgrader: WebSocketUpgrade) -> impl IntoResponse {
    upgrader
        .on_failed_upgrade(|err| tracing::error!(%err, "Upgrading request"))
        // NOTE: Here onward, the connection is a "WebSocket", so the usual
        // handler returns no longer work i.e. HTTP status codes.
        .on_upgrade(upgrade_handler)
}

async fn upgrade_handler(mut socket: WebSocket) {
    while let Some(message) = socket.next().await {
        // Can be I/O or protocol related errors i.e. malformed frames but is
        // commonly an informal closure without a close frame.
        if let Err(err) = message {
            tracing::error!(%err, "Receiving socket message");
            break;
        }

        if let Ok(Message::Text(text)) = message {
            text_message_handler(&mut socket, text).await;
        } else if let Ok(Message::Close(frame)) = message {
            tracing::info!(?frame, "Received close frame");
            // NOTE: If needed, a close frame is sent back automatically for us.
            break;
        }
    }
}

async fn text_message_handler(socket: &mut WebSocket, text: Utf8Bytes) {
    tracing::info!(%text, "Received \"text\" message");

    let reply = Utf8Bytes::from_static(TEXT_REPLY);
    if let Err(err) = socket.send(Message::Text(reply)).await {
        tracing::error!(%err, "Sending text reply");
    }
}

#[cfg(test)]
mod tests {
    use crate::handler::HandlerState;
    use crate::handler::TEST_ID_HEADER_KEY;
    use crate::handler::websocket::TEXT_REPLY;

    use axum::http::HeaderValue;
    use axum::http::StatusCode;
    use futures_util::SinkExt;
    use futures_util::StreamExt;
    use tokio::net::TcpStream;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use tokio_tungstenite::tungstenite::Error;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::Utf8Bytes;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::error::ProtocolError;

    const ENDPOINT: &str = "ws://localhost:8080/ws";

    async fn connect_socket(identifier: &str) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let mut request = ENDPOINT.into_client_request().expect("Building request");
        let identifier = HeaderValue::from_str(identifier).expect("Parsing identifier");

        request.headers_mut().insert(TEST_ID_HEADER_KEY, identifier);
        let (socket, response) = tokio_tungstenite::connect_async(request)
            .await
            .expect("Connecting to socket");

        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        socket
    }

    #[tokio::test]
    async fn should_reply_with_response_text() {
        let identifier = HandlerState::setup_for_test().await;
        let mut socket = connect_socket(&identifier).await;

        let payload = Utf8Bytes::from_static("Hello");
        socket
            .send(Message::Text(payload))
            .await
            .expect("Sending text payload");

        let message = socket
            .next()
            .await
            .expect("Iterating over stream")
            .expect("Unwrapping stream message");

        let reply = Utf8Bytes::from_static(TEXT_REPLY);
        assert_eq!(message, Message::Text(reply));
    }

    #[tokio::test]
    async fn should_accept_close_frame() {
        let identifier = HandlerState::setup_for_test().await;
        let mut socket = connect_socket(&identifier).await;

        socket
            .send(Message::Close(None))
            .await
            .expect("Sending close frame");

        let payload = Utf8Bytes::from_static("Goodbye");
        let err = socket
            .send(Message::Text(payload))
            .await
            .expect_err("Sending to a closed socket");

        let has_closed = matches!(err, Error::Protocol(ProtocolError::SendAfterClosing));
        assert!(has_closed);
    }
}
