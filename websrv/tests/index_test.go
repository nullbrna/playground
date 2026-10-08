package tests

import (
	"encoding/json"
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/nullbrna/playground/websrv/internal/store"
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

func TestTenUsers(test *testing.T) {
	status, body := callEndpoint(test, "/ten-users")
	require.Equal(test, http.StatusOK, status)

	var users []store.User
	err := json.Unmarshal(body, &users)

	require.NoError(test, err)
	require.Len(test, users, 10)

	for i := 0; i < len(users); i++ {
		user := users[i]
		require.NotEmpty(test, user.Name)
		require.NotEmpty(test, user.Email)
	}
}

func BenchmarkTenUsers(bench *testing.B) {
	status, _ := callEndpoint(bench, "/ten-users")
	require.Equal(bench, http.StatusOK, status)

	bench.ResetTimer()
	for bench.Loop() {
		status, _ := callEndpoint(bench, "/ten-users")
		require.Equal(bench, http.StatusOK, status)
	}
}

func TestUserByID(test *testing.T) {
	status, body := callEndpoint(test, "/user/1")
	require.Equal(test, http.StatusOK, status)

	var user store.User
	err := json.Unmarshal(body, &user)

	require.NoError(test, err)
	require.Equal(test, "User 1", user.Name)
	require.Equal(test, "user-1@example.test", user.Email)
}

func BenchmarkUserById(bench *testing.B) {
	status, _ := callEndpoint(bench, "/user/1")
	require.Equal(bench, http.StatusOK, status)

	bench.ResetTimer()
	for bench.Loop() {
		status, _ := callEndpoint(bench, "/user/1")
		require.Equal(bench, http.StatusOK, status)
	}
}

func TestMissingUser(test *testing.T) {
	status, _ := callEndpoint(test, "/user/250001")
	require.Equal(test, http.StatusNotFound, status)
}

func TestUserCount(test *testing.T) {
	status, body := callEndpoint(test, "/user-count")
	require.Equal(test, http.StatusOK, status)
	require.Equal(test, "250000", string(body))
}

func BenchmarkUserCount(bench *testing.B) {
	status, _ := callEndpoint(bench, "/user-count")
	require.Equal(bench, http.StatusOK, status)

	bench.ResetTimer()
	for bench.Loop() {
		status, _ := callEndpoint(bench, "/user-count")
		require.Equal(bench, http.StatusOK, status)
	}
}
