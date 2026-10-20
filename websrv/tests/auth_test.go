package tests

import (
	"net/http"
	"net/url"
	"slices"
	"testing"

	"github.com/stretchr/testify/require"
)

const authLoginRoute = "http://localhost:8080/auth/login"

func TestAuthLoginRedirect(test *testing.T) {
	resetClientRedirect := setClientToNotFollowRedirects()
	defer resetClientRedirect()

	loginResponse, err := httpClient.Get(authLoginRoute)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	// Extract the URL of where the redirect from our login endpoint was
	// targeting. This response is NOT the provider's response.
	redirectHeader := loginResponse.Header.Get("Location")
	redirectTo, err := url.Parse(redirectHeader)
	require.NoError(test, err)

	require.Equal(test, http.StatusFound, loginResponse.StatusCode)
	require.Equal(test, "localhost:7654", redirectTo.Host)
}

func TestAuthLoginSendsOAuthCookies(test *testing.T) {
	resetClientRedirect := setClientToNotFollowRedirects()
	defer resetClientRedirect()

	loginResponse, err := httpClient.Get(authLoginRoute)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	redirectCookies := loginResponse.Cookies()
	require.Equal(test, 2, len(redirectCookies))

	stateCookieIndex := slices.IndexFunc(redirectCookies, func(cookie *http.Cookie) bool {
		return cookie.Name == "oauth.state"
	})
	verifierCookieIndex := slices.IndexFunc(redirectCookies, func(cookie *http.Cookie) bool {
		return cookie.Name == "oauth.verifier"
	})

	require.NotEqual(test, -1, stateCookieIndex)
	require.NotEqual(test, -1, verifierCookieIndex)
}
