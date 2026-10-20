package handler

import (
	"crypto/rand"
	"encoding/base64"
	"net/http"

	"github.com/gofiber/fiber/v3"
	zlog "github.com/rs/zerolog/log"
	"golang.org/x/oauth2"
)

func (this *handler) loginRoute(fib fiber.Ctx) error {
	state := generateRandomOAuthState()
	verifier := oauth2.GenerateVerifier()

	const temporaryLifetimeSecs = 300

	// 1. Store this short-lived, callback-scoped cookie in the browser.
	// 2. The provider gets our state in the URL and echoes it to our callback.
	// 3. The callback will compare and assert that value with this cookie.
	newTemporaryOAuthSessionCookie(
		fib,
		"oauth.state",
		state,
		"/auth/callback",
		temporaryLifetimeSecs,
	)

	// 1. Store the raw PKCE verifier, callback-scoped cookie in the browser.
	// 2. The provider gets the challenge, issues an authorisation code from it.
	// 3. Our callback exchanges this code and our verifier for a token.
	// 4. The provider issues said token by hashing the verifier.
	// 5. The hashed verifier must match the stored challenge.
	newTemporaryOAuthSessionCookie(
		fib,
		"oauth.verifier",
		verifier,
		"/auth/callback",
		temporaryLifetimeSecs,
	)

	providerChallenge := oauth2.S256ChallengeOption(verifier)
	redirectTo := this.oauthConfig.AuthCodeURL(state, providerChallenge)

	zlog.Trace().Str("to", redirectTo).Msg("Redirecting to OAuth provider")
	return fib.Redirect().Status(http.StatusFound).To(redirectTo)
}

func generateRandomOAuthState() string {
	const stateByteCount = 32

	randomStateBytes := make([]byte, stateByteCount)
	// [NOTE] As per [rand.Read], "It never returns an error... crashes the
	// program irrecoverably if an error is returned".
	rand.Read(randomStateBytes)

	return base64.RawURLEncoding.EncodeToString(randomStateBytes)
}

func newTemporaryOAuthSessionCookie(fib fiber.Ctx, name, value, path string, maxAge int) {
	var newCookie fiber.Cookie
	newCookie.Name = name
	newCookie.Path = path
	newCookie.Value = value
	newCookie.MaxAge = maxAge
	newCookie.Secure = false // [NOTE] Should be changed for HTTPS traffic.
	newCookie.HTTPOnly = true
	newCookie.SameSite = "Lax"

	fib.Cookie(&newCookie)
}
