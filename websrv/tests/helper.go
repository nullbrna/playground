package tests

import (
	"net/http"
	"time"
)

var httpClient = http.Client{Timeout: 1 * time.Second}

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
