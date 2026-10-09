package tests

import (
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/require"
)

var (
	httpClient = http.Client{Timeout: 5 * time.Second}
	wsClient   = websocket.Dialer{HandshakeTimeout: 5 * time.Second}
)

func callEndpoint(test testing.TB, endpoint string) (int, []byte) {
	test.Helper()

	response, err := httpClient.Get("http://localhost:8080" + endpoint)
	require.NoError(test, err)
	defer response.Body.Close()

	responseBody, err := io.ReadAll(response.Body)
	require.NoError(test, err)

	return response.StatusCode, responseBody
}

func callSocket(test testing.TB) *websocket.Conn {
	test.Helper()

	conn, response, err := wsClient.Dial("ws://localhost:8080/ws", nil)
	if response != nil && response.Body != nil {
		defer response.Body.Close()
	}

	require.NoError(test, err)
	return conn
}
