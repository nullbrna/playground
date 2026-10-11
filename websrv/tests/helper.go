package tests

import (
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

var httpClient = http.Client{Timeout: 5 * time.Second}

func callEndpoint(test testing.TB, endpoint string) (int, []byte) {
	test.Helper()

	response, err := httpClient.Get("http://localhost:8080" + endpoint)
	require.NoError(test, err)
	defer response.Body.Close()

	responseBody, err := io.ReadAll(response.Body)
	require.NoError(test, err)

	return response.StatusCode, responseBody
}
