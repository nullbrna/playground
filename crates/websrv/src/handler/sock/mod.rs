use axum::Error;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::Message;
use axum::extract::ws::Utf8Bytes;
use axum::extract::ws::WebSocket;
use axum::response::IntoResponse;

pub async fn core(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(async |mut socket| {
        while let Some(result) = socket.recv().await {
            handle_socket(&mut socket, result).await
        }
    })
}

async fn handle_socket(socket: &mut WebSocket, payload: Result<Message, Error>) {
    let Ok(message) = payload else {
        tracing::error!(err = %payload.unwrap_err(), "Reading socket message");
        return;
    };

    tracing::info!(?message, "Received socket payload");
    if let Message::Text(text) = message
        && text == "ping"
    {
        let pong = Utf8Bytes::from_static("pong");
        socket.send(Message::Text(pong)).await.ok();
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use futures_util::SinkExt;
    use futures_util::StreamExt;
    use tokio::net::TcpStream;
    use tokio_tungstenite::MaybeTlsStream;
    use tokio_tungstenite::WebSocketStream;
    use tokio_tungstenite::tungstenite::Bytes;
    use tokio_tungstenite::tungstenite::Message;

    const ENDPOINT: &str = "ws://localhost:8080/sock";

    async fn new_socket_connection() -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let (socket, response) = tokio_tungstenite::connect_async(ENDPOINT).await.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

        socket
    }

    #[tokio::test]
    async fn should_ping_pong_then_close() {
        let mut socket = new_socket_connection().await;

        let ping = Bytes::from_static(b"Hello, world!");
        let pong = ping.clone();

        socket.send(Message::Ping(ping)).await.unwrap();
        let response = socket.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Pong(pong));

        socket.close(None).await.unwrap();
        let response = socket.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Close(None));
    }
}
