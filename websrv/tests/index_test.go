package tests

import (
	"encoding/json"
	"io"
	"net/http"
	"strconv"
	"testing"
	"time"

	"github.com/nullbrna/playground/websrv/internal/store"
	"github.com/stretchr/testify/require"
)

const url = "http://localhost:8080"

var httpClient = http.Client{Timeout: 5 * time.Second}

func callEndpoint(test *testing.T, path string) (int, []byte) {
	test.Helper()

	response, err := httpClient.Get(url + path)
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

func TestUserByID(test *testing.T) {
	status, body := callEndpoint(test, "/user/1")
	require.Equal(test, http.StatusOK, status)

	var user store.User
	err := json.Unmarshal(body, &user)

	require.NoError(test, err)
	require.Equal(test, "User 1", user.Name)
	require.Equal(test, "user-1@example.test", user.Email)
}

func TestMissingUser(test *testing.T) {
	status, _ := callEndpoint(test, "/user/250001")
	require.Equal(test, http.StatusNotFound, status)
}

func TestUserCount(test *testing.T) {
	status, body := callEndpoint(test, "/user-count")
	require.Equal(test, http.StatusOK, status)

	countFromString, err := strconv.ParseInt(string(body), 10, 64)
	require.NoError(test, err)
	require.Equal(test, int64(250_000), countFromString)
}
