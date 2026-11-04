package tests

import (
	"net/http"
	"time"
)

const IndexRouteURL = "http://localhost:8080/"

var BasicHTTPClient http.Client

func init() {
	BasicHTTPClient.Timeout = 1 * time.Second
}
