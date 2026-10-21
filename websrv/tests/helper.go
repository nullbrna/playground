package tests

import (
	"net/http"
	"time"
)

var httpClient = http.Client{Timeout: 1 * time.Second}
