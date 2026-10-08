package tests

import (
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

const url = "http://localhost:8080"

var httpClient = http.Client{Timeout: 5 * time.Second}

func TestIndex(test *testing.T) {
	response, err := httpClient.Get(url)
	require.NoError(test, err)
	defer response.Body.Close()

	require.Equal(test, http.StatusOK, response.StatusCode)

	responseBody, err := io.ReadAll(response.Body)
	require.NoError(test, err)

	responseText := string(responseBody)
	require.Equal(test, "Hello, world!", responseText)
}
