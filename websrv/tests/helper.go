package tests

import (
	"net/http"
	"net/http/cookiejar"
	"slices"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

const (
	AuthLoginURL     = "http://localhost:8080/auth/login"
	AuthLogoutURL    = "http://localhost:8080/auth/logout"
	AuthRefreshURL   = "http://localhost:8080/auth/refresh"
	AuthProtectedURL = "http://localhost:8080/auth/myself"

	defaultClientTimeout = 1 * time.Second
)

var (
	BasicHTTPClient    http.Client
	NoFollowHTTPClient http.Client
)

func init() {
	BasicHTTPClient.Timeout = defaultClientTimeout

	NoFollowHTTPClient.Timeout = defaultClientTimeout
	NoFollowHTTPClient.CheckRedirect = func(*http.Request, []*http.Request) error {
		return http.ErrUseLastResponse
	}
}

func NewAuthenticatedClient(test *testing.T) http.Client {
	test.Helper()

	authenticatedCookieJar, err := cookiejar.New(nil)
	require.NoError(test, err)

	// Persist the cookies beyond the login flow. Allows the authentication
	// cookies set at login to be read e.g. by a protected route.
	client := BasicHTTPClient
	client.Jar = authenticatedCookieJar

	loginResponse, err := client.Get(AuthLoginURL)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	require.Equal(test, http.StatusNoContent, loginResponse.StatusCode)
	require.Equal(test, "/auth/callback", loginResponse.Request.URL.Path)
	return client
}

func GetACookieFromHeaderList(cookiesFromHeader []*http.Cookie, nameToFind string) *http.Cookie {
	cookieIndex := slices.IndexFunc(cookiesFromHeader, func(eachCookie *http.Cookie) bool {
		return eachCookie.Name == nameToFind
	})

	if cookieIndex == -1 {
		return nil
	}

	return cookiesFromHeader[cookieIndex]
}
