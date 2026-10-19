package tests

import (
	"net/http"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

const indexRoute = "http://localhost:8080/"
const httpTimeout = 5 * time.Second

var httpClient = http.Client{Timeout: httpTimeout}

func TestIndexRoute(test *testing.T) {
	indexResponse, err := httpClient.Get(indexRoute)
	require.NoError(test, err)
	defer indexResponse.Body.Close()

	require.Equal(test, http.StatusOK, indexResponse.StatusCode)
}
