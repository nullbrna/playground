package auth

import (
	"context"
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"strconv"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/rs/zerolog/log"
	"golang.org/x/oauth2"
)

type InjectedHandler struct {
	DBPool      *pgxpool.Pool
	OAuthConfig *oauth2.Config
}

func RegisterRoutes(groupedRouter fiber.Router, rootHandler InjectedHandler) {
	groupedRouter.Get("/login", rootHandler.loginWithOAuth)
	groupedRouter.Get("/callback", rootHandler.callbackFromOAuthProvider)
	groupedRouter.Get("/myself", rootHandler.protectedMiddleware, rootHandler.myself)

	groupedRouter.Post("/logout", rootHandler.logoutFromOauth)
	groupedRouter.Post("/refresh", rootHandler.refreshOAuthTokens)
}

const (
	oauthStateCookieKey    = "oauth.state"
	oauthVerifierCookieKey = "oauth.verifier"
)

func (this *InjectedHandler) loginWithOAuth(requestInstance fiber.Ctx) error {
	randomStateBytes := make([]byte, 32)
	// NOTE: As per [rand.Read]: "It never returns an error... crashes the
	// program irrecoverably if an error is returned".
	rand.Read(randomStateBytes)

	// Encode the arbitrary bytes from [rand] to be URL safe.
	encodedRandomState := base64.RawURLEncoding.EncodeToString(randomStateBytes)
	randomPKCEVerifier := oauth2.GenerateVerifier()

	const lifetimeInSecs = 300
	// 1. The provider gets our state in the URL and echoes it to our callback.
	// 2. The callback will compare and assert that value with this cookie.
	setOAuthSessionCookie(requestInstance, oauthStateCookieKey, encodedRandomState, lifetimeInSecs)
	// 1. The provider gets the challenge and issues an authorisation code (made
	// by hashing the verifier) from it.
	// 2. Our callback exchanges this code and our verifier for a token once the
	// provider asserts the verifier matches the saved challenge.
	setOAuthSessionCookie(requestInstance, oauthVerifierCookieKey, randomPKCEVerifier, lifetimeInSecs)

	providerAuthDestination := this.OAuthConfig.AuthCodeURL(
		encodedRandomState,
		oauth2.S256ChallengeOption(randomPKCEVerifier),
	)

	log.Trace().Msg("Redirecting to OAuth provider")
	return requestInstance.Redirect().Status(http.StatusFound).To(providerAuthDestination)
}

func (this *InjectedHandler) callbackFromOAuthProvider(requestInstance fiber.Ctx) error {
	stateCookie := requestInstance.Cookies(oauthStateCookieKey)
	verifierCookie := requestInstance.Cookies(oauthVerifierCookieKey)

	const immediatelyExpiredAge = -1
	// Remove the temporary cookies made at login as soon as possible. Even if
	// the error is on our side, restart the login process.
	// ---
	// NOTE: Same path and empty value ensures the client/browser deletes it.
	setOAuthSessionCookie(requestInstance, oauthStateCookieKey, "", immediatelyExpiredAge)
	setOAuthSessionCookie(requestInstance, oauthVerifierCookieKey, "", immediatelyExpiredAge)

	stateFromProvider := requestInstance.Query("state")
	authorisationCode := requestInstance.Query("code")
	if statusCode := validateRedirectedDataFromProvider(
		requestInstance,
		stateFromProvider,
		authorisationCode,
		stateCookie,
		verifierCookie,
	); statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	providerQueryContext, queryContextCancel := context.WithTimeout(
		requestInstance.Context(),
		1*time.Second,
	)
	defer queryContextCancel()

	// 1. Exchange the verifier for a one-time authorisation code.
	// 2. Provider validates the verifier against the challenge sent at login.
	// 3. The returned token acts as a credential to get the user profile.
	// 4. User-unique subject is returned along with profile fields.
	providerProfile, statusCode := fetchUserProviderProfile(
		providerQueryContext,
		this.OAuthConfig,
		authorisationCode,
		verifierCookie,
	)
	if statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	if statusCode := saveUserToDBAndSession(
		requestInstance,
		this,
		&providerProfile,
	); statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	return requestInstance.SendStatus(http.StatusNoContent)
}

const userIdScopedKey = "local.user"

func (this *InjectedHandler) protectedMiddleware(requestInstance fiber.Ctx) error {
	accessCookieName := accessTokenKind.cookieName()
	accessCookie := requestInstance.Cookies(accessCookieName)

	userIdFromAccessCookie, statusCode := decodeTokenFromKind(accessCookie)
	if statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	// NOTE: As per [SetLocal]: "... slice is heap-allocated on every call.
	// SetLocal takes the value directly, so that slice is never built."
	requestInstance.SetLocal(userIdScopedKey, userIdFromAccessCookie)
	return requestInstance.Next()
}

func (this *InjectedHandler) myself(requestInstance fiber.Ctx) error {
	userIdFromLocal, didTypeCast := requestInstance.Locals(userIdScopedKey).(int64)
	if !didTypeCast {
		return requestInstance.SendStatus(http.StatusUnauthorized)
	}

	stringifiedId := strconv.FormatInt(userIdFromLocal, 10)
	return requestInstance.SendString(stringifiedId)
}

func (this *InjectedHandler) logoutFromOauth(requestInstance fiber.Ctx) error {
	accessCookieName, refreshCookieName := accessTokenKind.cookieName(), refreshTokenKind.cookieName()
	accessCookiePath, refreshCookiePath := accessTokenKind.cookiePath(), refreshTokenKind.cookiePath()

	refreshCookie := requestInstance.Cookies(refreshCookieName)
	unsetAuthTokenCookie(requestInstance, accessCookieName, accessCookiePath)
	unsetAuthTokenCookie(requestInstance, refreshCookieName, refreshCookiePath)

	userIdFromRefreshCookie, statusCode := decodeTokenFromKind(refreshCookie)
	if statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	// NOTE: Although not an issue, worth knowing that if this transaction
	// fails, the cookies are still cleared. Recreated/set at login anyhow.
	if statusCode := this.deleteRefreshToken(
		requestInstance.Context(),
		userIdFromRefreshCookie,
		hashTokenValue(refreshCookie),
	); statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	return requestInstance.SendStatus(http.StatusNoContent)
}

func (this *InjectedHandler) refreshOAuthTokens(requestInstance fiber.Ctx) error {
	refreshName := refreshTokenKind.cookieName()
	refreshCookie := requestInstance.Cookies(refreshName)

	shouldClearCookies := false
	// For unauthorised error cases, we ALWAYS want to clear the cookies.
	// ---
	// NOTE: We can't simply defer the deletion as the refreshed cookies would
	// also be deleted at the end of the handler.
	defer func() {
		if shouldClearCookies {
			accessPath, refreshPath := accessTokenKind.cookiePath(), refreshTokenKind.cookiePath()
			unsetAuthTokenCookie(requestInstance, accessTokenKind.cookieName(), accessPath)
			unsetAuthTokenCookie(requestInstance, refreshName, refreshPath)
		}
	}()

	userIdFromRefreshCookie, statusCode := decodeTokenFromKind(refreshCookie)
	if statusCode != http.StatusOK {
		shouldClearCookies = true
		return requestInstance.SendStatus(statusCode)
	}

	newAccessToken, newRefreshToken, statusCode := newTokenPair(userIdFromRefreshCookie)
	if statusCode != http.StatusOK {
		return requestInstance.SendStatus(statusCode)
	}

	if statusCode := this.rotateRefreshToken(
		requestInstance.Context(),
		userIdFromRefreshCookie,
		hashTokenValue(refreshCookie),
		hashTokenValue(newRefreshToken.value),
		newRefreshToken.expiresAt,
	); statusCode != http.StatusOK {
		// NOTE: An internal error querying the database shouldn't clear the
		// authentication cookies. Only when there's no token to rotate.
		if statusCode == http.StatusUnauthorized {
			shouldClearCookies = true
		}

		return requestInstance.SendStatus(statusCode)
	}

	newAccessToken.intoCookieForResponse(requestInstance)
	newRefreshToken.intoCookieForResponse(requestInstance)
	return requestInstance.SendStatus(http.StatusNoContent)
}

func setOAuthSessionCookie(
	requestInstance fiber.Ctx,
	cookieName, cookieValue string,
	ageInSecs int,
) {
	var cookieToSet fiber.Cookie
	cookieToSet.Name = cookieName
	cookieToSet.Value = cookieValue
	cookieToSet.MaxAge = ageInSecs
	cookieToSet.Secure = false
	cookieToSet.HTTPOnly = true
	cookieToSet.SameSite = "Lax"
	// This cookie will only live through login -> provider -> callback so we
	// can hard-code the path to limit usage.
	// ---
	// NOTE: This must be the same when removing the cookies also.
	cookieToSet.Path = "/auth/callback"

	requestInstance.Cookie(&cookieToSet)
}

func unsetAuthTokenCookie(requestInstance fiber.Ctx, cookieName, cookiePath string) {
	var cookieToSet fiber.Cookie
	cookieToSet.Name = cookieName
	cookieToSet.Path = cookiePath
	cookieToSet.Value = ""
	cookieToSet.MaxAge = -1
	cookieToSet.Secure = false
	cookieToSet.HTTPOnly = true
	cookieToSet.SameSite = "Lax"

	requestInstance.Cookie(&cookieToSet)
}

func validateRedirectedDataFromProvider(
	requestInstance fiber.Ctx,
	stateFromProvider, authorisationCode, stateCookie, verifierCookie string,
) int {
	if err := requestInstance.Query("error"); len(err) > 0 {
		log.Error().Str("err", err).Msg("Error found in provider redirect URL")
		return http.StatusBadRequest
	}

	if len(stateFromProvider) == 0 || len(authorisationCode) == 0 {
		log.Error().Msg("Missing state/authorisation code from provider")
		return http.StatusBadRequest
	}

	if len(stateCookie) == 0 || len(verifierCookie) == 0 {
		log.Error().Msg("Missing cookies set at login in provider redirect")
		return http.StatusBadRequest
	}

	// NOTE: Mitigate CSRF. Always compare every byte, not stopping at the first
	// mismatch, with [subtle.ConstantTimeCompare].
	if subtle.ConstantTimeCompare([]byte(stateFromProvider), []byte(stateCookie)) != 1 {
		log.Error().Msg("Mismatch between saved/provided state")
		return http.StatusBadRequest
	}

	return http.StatusOK
}

type providerProfile struct {
	Name  string `json:"name"`
	Email string `json:"email"`
	Sub   string `json:"sub"`
}

func fetchUserProviderProfile(
	providerQueryContext context.Context,
	oauthConfig *oauth2.Config,
	authorisationCode, verifierCookie string,
) (providerProfile, int) {
	exchangedProviderToken, err := oauthConfig.Exchange(
		providerQueryContext,
		authorisationCode,
		oauth2.VerifierOption(verifierCookie),
	)
	if err != nil {
		log.Error().Err(err).Msg("Exchanging authorisation code for token")
		return providerProfile{}, http.StatusBadGateway
	}

	userProfileRequest, err := http.NewRequestWithContext(
		providerQueryContext,
		http.MethodGet,
		"http://mock-oauth:8080/default/userinfo",
		nil,
	)
	if err != nil {
		log.Error().Err(err).Msg("Building user profile request")
		return providerProfile{}, http.StatusInternalServerError
	}

	userProfileResponse, err := oauthConfig.Client(
		providerQueryContext,
		exchangedProviderToken,
	).Do(userProfileRequest)
	if err != nil {
		log.Error().Err(err).Msg("Sending request for user profile from provider")
		return providerProfile{}, http.StatusBadGateway
	}

	defer userProfileResponse.Body.Close()
	if userProfileResponse.StatusCode != http.StatusOK {
		log.Error().Msg("Bad status code returned from request for user profile")
		return providerProfile{}, http.StatusBadGateway
	}

	// Through OIDC, we've requested a certain shape of user profile. This isn't
	// a guarantee however i.e. a provider may not abide by OIDC.
	// ---
	// NOTE: See more at https://openid.net/developers/how-connect-works.
	var userProfile providerProfile
	if err := json.NewDecoder(userProfileResponse.Body).Decode(&userProfile); err != nil {
		log.Error().Err(err).Msg("Decoding user profile response")
		return providerProfile{}, http.StatusBadGateway
	}

	if len(userProfile.Name) == 0 || len(userProfile.Email) == 0 || len(userProfile.Sub) == 0 {
		log.Error().Msg("Missing/empty data returned from user profile request")
		return providerProfile{}, http.StatusBadGateway
	}

	return userProfile, http.StatusOK
}

func saveUserToDBAndSession(
	requestInstance fiber.Ctx,
	handler *InjectedHandler,
	userProfile *providerProfile,
) int {
	savedUser, statusCode := handler.upsertUserFromProviderProfile(
		requestInstance.Context(),
		userProfile,
	)
	if statusCode != http.StatusOK {
		return statusCode
	}

	newAccessToken, newRefreshToken, statusCode := newTokenPair(savedUser.id)
	if statusCode != http.StatusOK {
		return statusCode
	}

	if statusCode := handler.saveRefreshToken(
		requestInstance.Context(),
		hashTokenValue(newRefreshToken.value),
		savedUser.id,
		newRefreshToken.expiresAt,
	); statusCode != http.StatusOK {
		return statusCode
	}

	log.Debug().Int64("userId", savedUser.id).Msg("Saved user from provider")
	newAccessToken.intoCookieForResponse(requestInstance)
	newRefreshToken.intoCookieForResponse(requestInstance)

	return http.StatusOK
}
