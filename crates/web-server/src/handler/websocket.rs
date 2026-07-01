use axum::Extension;
use axum::extract::State;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::Message;
use axum::extract::ws::Utf8Bytes;
use axum::extract::ws::WebSocket;
use axum::response::IntoResponse;
use futures_util::StreamExt;

use crate::handler::HandlerState;

pub async fn core(
    ws_handler: WebSocketUpgrade,
    Extension(identifier): Extension<String>,
    State(state): State<HandlerState>,
) -> impl IntoResponse {
    ws_handler.on_upgrade(move |socket| upgrade_handler(socket, identifier, state))
}

async fn upgrade_handler(mut socket: WebSocket, identifier: String, state: HandlerState) {
    while let Some(Ok(message)) = socket.next().await {
        if let Message::Close(frame) = message {
            tracing::debug!(?frame, "[WEBSOCKET] Received closure frame");
            break;
        }

        if let Message::Text(text) = message {
            text_frame_handler(&mut socket, text).await;
        }
    }
}

async fn text_frame_handler(socket: &mut WebSocket, text: Utf8Bytes) {
    tracing::debug!(%text, "[WEBSOCKET] Received text");

    let response = Utf8Bytes::from_static("Hello, world!");
    if let Err(err) = socket.send(Message::Text(response)).await {
        tracing::error!(%err, "[WEBSOCKET] Sending text response");
    }
}

#[cfg(test)]
mod tests {
    use crate::handler::HandlerState;
    use crate::handler::TEST_ID_HEADER_KEY;

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
        let identifier_header = HeaderValue::from_str(identifier).expect("Parsing identifier");

        request
            .headers_mut()
            .insert(TEST_ID_HEADER_KEY, identifier_header);

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

        let body = Utf8Bytes::from_static("Hello");
        socket
            .send(Message::Text(body))
            .await
            .expect("Sending text body");

        let message = socket
            .next()
            .await
            .expect("Iterating over stream")
            .expect("Unwrapping stream item");

        let response = Utf8Bytes::from_static("Hello, world!");
        assert_eq!(message, Message::Text(response));
    }

    #[tokio::test]
    async fn should_accept_close_frame() {
        let identifier = HandlerState::setup_for_test().await;
        let mut socket = connect_socket(&identifier).await;

        socket
            .send(Message::Close(None))
            .await
            .expect("Sending close frame");

        let body = Utf8Bytes::from_static("Goodbye");
        let response = socket
            .send(Message::Text(body))
            .await
            .expect_err("Sending to a closed socket");

        let has_closed = matches!(response, Error::Protocol(ProtocolError::SendAfterClosing));
        assert!(has_closed);
    }
}
