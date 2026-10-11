package tests

import (
	"net/http"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestIndex(test *testing.T) {
	responseStatus, responseBody := callEndpoint(test, "/")
	require.Equal(test, http.StatusOK, responseStatus)
	require.Equal(test, "Hello, world!", string(responseBody))
}

func BenchmarkIndex(bench *testing.B) {
	callEndpoint(bench, "/")
	bench.ResetTimer()

	for bench.Loop() {
		responseStatus, responseBody := callEndpoint(bench, "/")
		require.Equal(bench, http.StatusOK, responseStatus)
		require.Equal(bench, "Hello, world!", string(responseBody))
	}
}
