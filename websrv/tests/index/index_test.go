package index_test

import (
	"net/http"
	"testing"

	"github.com/nullbrna/playground/websrv/tests"
	"github.com/stretchr/testify/require"
)

func TestIndexOKResponse(test *testing.T) {
	indexResponse, err := tests.BasicHTTPClient.Get(tests.IndexRouteURL)
	require.NoError(test, err)
	defer indexResponse.Body.Close()

	require.Equal(test, http.StatusOK, indexResponse.StatusCode)
}
