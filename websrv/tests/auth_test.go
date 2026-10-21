package tests

import (
	"net/http"
	"net/http/cookiejar"
	"net/url"
	"slices"
	"testing"

	"github.com/stretchr/testify/require"
)

const authLoginRoute = "http://localhost:8080/auth/login"

func TestAuthLoginRedirectsToTheProvider(test *testing.T) {
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
	require.Len(test, redirectCookies, 2)

	stateCookieIndex := slices.IndexFunc(redirectCookies, func(cookie *http.Cookie) bool {
		return cookie.Name == "oauth.state"
	})
	verifierCookieIndex := slices.IndexFunc(redirectCookies, func(cookie *http.Cookie) bool {
		return cookie.Name == "oauth.verifier"
	})

	require.NotEqual(test, -1, stateCookieIndex)
	require.NotEqual(test, -1, verifierCookieIndex)
}

func TestAuthCallbackRedirectsFromTheProvider(test *testing.T) {
	jar, err := cookiejar.New(nil)
	require.NoError(test, err)

	resetCookieJar := setClientCookieJar(jar)
	defer resetCookieJar()

	loginResponse, err := httpClient.Get(authLoginRoute)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	require.Equal(test, http.StatusNoContent, loginResponse.StatusCode)
	// 1. Request to the login endpoint redirects to the mock provider.
	// 2. The HTTP client follows the redirect.
	// 3. Provider also returns a redirect, it doesn't request directly.
	// 4. Although we called the login endpoint, end URL is the callback.
	require.Equal(test, "/auth/callback", loginResponse.Request.URL.Path)
}

func TestAuthCallbackSetsTokenCookies(test *testing.T) {
	jar, err := cookiejar.New(nil)
	require.NoError(test, err)

	resetCookieJar := setClientCookieJar(jar)
	defer resetCookieJar()

	loginResponse, err := httpClient.Get(authLoginRoute)
	require.NoError(test, err)
	defer loginResponse.Body.Close()

	callbackCookies := loginResponse.Cookies()
	// The callback has deleted the 2 temporary cookies used for verification
	// and authentication. Yet they exist as "deleted" cookies in the response.
	require.Len(test, callbackCookies, 4)

	accessCookieIndex := slices.IndexFunc(callbackCookies, func(cookie *http.Cookie) bool {
		return cookie.Name == "token.access"
	})
	refreshCookieIndex := slices.IndexFunc(callbackCookies, func(cookie *http.Cookie) bool {
		return cookie.Name == "token.refresh"
	})

	require.NotEqual(test, -1, accessCookieIndex)
	require.NotEqual(test, -1, refreshCookieIndex)
}

func setClientToNotFollowRedirects() func() {
	httpClient.CheckRedirect = func(*http.Request, []*http.Request) error {
		// Return the initial redirect status code. Don't follow it so we can
		// inspect the initial request and not the follow-up.
		return http.ErrUseLastResponse
	}

	return func() {
		httpClient.CheckRedirect = nil
	}
}

func setClientCookieJar(newJar *cookiejar.Jar) func() {
	// The default [http.Client] doesn't save/resend cookies with requests. The
	// [cookiejar.Jar] instead saves and sends them when needed.
	httpClient.Jar = newJar

	return func() {
		httpClient.Jar = nil
	}
}
