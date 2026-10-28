package auth_test

import (
	"net/http"
	"net/url"
	"testing"

	"github.com/nullbrna/playground/websrv/tests"
	"github.com/stretchr/testify/require"
)

func TestAuthRotatesRefreshToken(test *testing.T) {
	authenticatedClient := tests.NewAuthenticatedClient(test)

	// Authentication cookies are set under the authenticated path. In order to
	// find the related cookies, construct a URL to match said path.
	urlForMatchingAuthCookies, err := url.Parse(tests.AuthRefreshURL)
	require.NoError(test, err)

	oldRefreshCookie := tests.GetACookieFromHeaderList(
		authenticatedClient.Jar.Cookies(urlForMatchingAuthCookies),
		"token.refresh",
	)
	require.NotNil(test, oldRefreshCookie)

	tokenRefreshResponse, err := authenticatedClient.Post(tests.AuthRefreshURL, "", nil)
	require.NoError(test, err)
	defer tokenRefreshResponse.Body.Close()

	require.Equal(test, http.StatusNoContent, tokenRefreshResponse.StatusCode)
	newRefreshCookie := tests.GetACookieFromHeaderList(
		tokenRefreshResponse.Cookies(),
		"token.refresh",
	)
	require.NotNil(test, newRefreshCookie)

	require.NotEmpty(test, newRefreshCookie.Value)
	require.NotEqual(test, oldRefreshCookie.Value, newRefreshCookie.Value)
}

func TestAuthCannotRotateMissingRefreshToken(test *testing.T) {
	authenticatedClient := tests.NewAuthenticatedClient(test)
	authenticatedClient.Jar = nil

	tokenRefreshResponse, err := authenticatedClient.Post(tests.AuthRefreshURL, "", nil)
	require.NoError(test, err)
	defer tokenRefreshResponse.Body.Close()

	require.Equal(test, http.StatusUnauthorized, tokenRefreshResponse.StatusCode)
}

func TestAuthCannotRotateInvalidRefreshToken(test *testing.T) {
	authenticatedClient := tests.NewAuthenticatedClient(test)

	urlForMatchingAuthCookies, err := url.Parse(tests.AuthRefreshURL)
	require.NoError(test, err)

	var overwrittenRefreshToken http.Cookie
	overwrittenRefreshToken.Name = "token.refresh"
	overwrittenRefreshToken.Path = "/auth"
	overwrittenRefreshToken.Value = "invalid-refresh-token"

	// Overwrite the refresh token set when logging in. An incorrect token value
	// will fail the decoding step and return an unauthorised status code.
	authenticatedClient.Jar.SetCookies(
		urlForMatchingAuthCookies,
		[]*http.Cookie{&overwrittenRefreshToken},
	)

	tokenRefreshResponse, err := authenticatedClient.Post(tests.AuthRefreshURL, "", nil)
	require.NoError(test, err)
	defer tokenRefreshResponse.Body.Close()

	require.Equal(test, http.StatusUnauthorized, tokenRefreshResponse.StatusCode)
}
