package tests

import (
	"testing"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
)

func TestSocketPingPong(test *testing.T) {
	conn := callSocket(test)
	defer conn.Close()

	err := conn.WriteMessage(websocket.TextMessage, []byte("ping"))
	require.NoError(test, err)

	messageType, message, err := conn.ReadMessage()
	require.NoError(test, err)
	require.Equal(test, websocket.TextMessage, messageType)
	require.Equal(test, "pong", string(message))
}

func BenchmarkSocketPingPong(bench *testing.B) {
	conn := callSocket(bench)
	defer conn.Close()

	bench.ResetTimer()
	for bench.Loop() {
		err := conn.WriteMessage(websocket.TextMessage, []byte("ping"))
		require.NoError(bench, err)

		messageType, message, err := conn.ReadMessage()
		require.NoError(bench, err)
		require.Equal(bench, websocket.TextMessage, messageType)
		require.Equal(bench, "pong", string(message))
	}
}
