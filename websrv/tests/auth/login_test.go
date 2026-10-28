package auth_test

import (
	"net/http"
	"net/url"
	"testing"

	"github.com/nullbrna/playground/websrv/tests"
	"github.com/stretchr/testify/require"
)

func TestAuthLoginRedirectsToTheProvider(test *testing.T) {
	loginResponse, err := tests.NoFollowHTTPClient.Get(tests.AuthLoginURL)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	// Extract the URL of where the redirect from our login endpoint was
	// targeting. This response is NOT the provider response.
	redirectHeader := loginResponse.Header.Get("Location")
	redirectTo, err := url.Parse(redirectHeader)
	require.NoError(test, err)

	require.Equal(test, http.StatusFound, loginResponse.StatusCode)
	require.Equal(test, "localhost:7654", redirectTo.Host)
}

func TestAuthLoginStoresOAuthCookies(test *testing.T) {
	loginResponse, err := tests.NoFollowHTTPClient.Get(tests.AuthLoginURL)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	callbackCookies := loginResponse.Cookies()
	require.Len(test, callbackCookies, 2)

	stateCookie := tests.GetACookieFromHeaderList(callbackCookies, "oauth.state")
	verifierCookie := tests.GetACookieFromHeaderList(callbackCookies, "oauth.verifier")

	require.NotNil(test, stateCookie)
	require.NotNil(test, verifierCookie)
}

func TestAuthLoginPermitsProtectedRouteRequests(test *testing.T) {
	unauthenticatedResponse, err := tests.BasicHTTPClient.Get(tests.AuthProtectedURL)
	require.NoError(test, err)
	defer unauthenticatedResponse.Body.Close()

	require.Equal(test, http.StatusUnauthorized, unauthenticatedResponse.StatusCode)
	authenticatedClient := tests.NewAuthenticatedClient(test)

	authenticatedResponse, err := authenticatedClient.Get(tests.AuthProtectedURL)
	require.NoError(test, err)
	defer authenticatedResponse.Body.Close()

	require.Equal(test, http.StatusOK, authenticatedResponse.StatusCode)
}

func TestAuthLogoutClearsSession(test *testing.T) {
	authenticatedClient := tests.NewAuthenticatedClient(test)

	logoutResponse, err := authenticatedClient.Post(tests.AuthLogoutURL, "", nil)
	require.NoError(test, err)
	defer logoutResponse.Body.Close()

	require.Equal(test, http.StatusNoContent, logoutResponse.StatusCode)
	accessCookie := tests.GetACookieFromHeaderList(logoutResponse.Cookies(), "token.access")
	refreshCookie := tests.GetACookieFromHeaderList(logoutResponse.Cookies(), "token.refresh")
	require.NotNil(test, accessCookie)
	require.NotNil(test, refreshCookie)

	// Logging out sets the cookies to be deleted by clearing the value and
	// negating the cookie age. The path must equal the cookie path also.
	// ---
	// NOTE: Technically the cookie still exists until the client (e.g. browser)
	// deletes said cookie.
	require.Empty(test, accessCookie.Value)
	require.Empty(test, refreshCookie.Value)
	require.Negative(test, accessCookie.MaxAge)
	require.Negative(test, refreshCookie.MaxAge)

	unauthenticatedResponse, err := authenticatedClient.Get(tests.AuthProtectedURL)
	require.NoError(test, err)
	defer unauthenticatedResponse.Body.Close()

	require.Equal(test, http.StatusUnauthorized, unauthenticatedResponse.StatusCode)
}
