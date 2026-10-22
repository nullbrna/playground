package auth

import (
	"context"
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/jackc/pgx/v5/pgxpool"
	zlog "github.com/rs/zerolog/log"
	"golang.org/x/oauth2"
)

type Handler struct {
	DBPool      *pgxpool.Pool
	OAuthConfig *oauth2.Config
}

func RegisterRoutes(router fiber.Router, root Handler) {
	router.Get("/login", root.login)
	router.Get("/callback", root.callback)
}

const oauthStateCookie = "oauth.state"
const oauthVerifierCookie = "oauth.verifier"

func (this *Handler) login(fib fiber.Ctx) error {
	randomStateBytes := make([]byte, 32)
	// NOTE: As per [rand.Read]: "It never returns an error... crashes the
	// program irrecoverably if an error is returned".
	rand.Read(randomStateBytes)

	// Encode the arbitrary bytes from [rand] to be URL safe.
	state := base64.RawURLEncoding.EncodeToString(randomStateBytes)
	verifier := oauth2.GenerateVerifier()

	const lifetimeInSecs = 300
	// 1. Save this short-lived, callback-scoped cookie in the browser.
	// 2. The provider gets our state in the URL and echoes it to our callback.
	// 3. The callback will compare and assert that value with this cookie.
	setOAuthSessionCookie(fib, oauthStateCookie, state, lifetimeInSecs)
	// 1. Save the raw PKCE verifier, callback-scoped cookie in the browser.
	// 2. The provider gets the challenge, issues an authorisation code from it.
	// 3. Our callback exchanges this code and our verifier for a token.
	// 4. The provider issues said token by hashing the verifier.
	// 5. The hashed verifier must match the saved challenge.
	setOAuthSessionCookie(fib, oauthVerifierCookie, verifier, lifetimeInSecs)

	redirectTo := this.OAuthConfig.AuthCodeURL(
		state,
		oauth2.S256ChallengeOption(verifier),
	)

	zlog.Trace().Msg("Redirecting to OAuth provider")
	return fib.Redirect().Status(http.StatusFound).To(redirectTo)
}

func (this *Handler) callback(fib fiber.Ctx) error {
	stateCookie := fib.Cookies(oauthStateCookie)
	verifierCookie := fib.Cookies(oauthVerifierCookie)

	const immediateExpiry = -1
	// Remove the temporary cookies made at login.
	// NOTE: Same path and empty value ensures browser deletes it.
	defer setOAuthSessionCookie(fib, oauthStateCookie, "", immediateExpiry)
	defer setOAuthSessionCookie(fib, oauthVerifierCookie, "", immediateExpiry)

	providerState, authorisationCode := fib.Query("state"), fib.Query("code")
	if statusCode := validateRedirectFromProvider(
		fib,
		providerState,
		authorisationCode,
		stateCookie,
		verifierCookie,
	); statusCode != http.StatusOK {
		return fib.SendStatus(statusCode)
	}

	ctx, cancel := context.WithTimeout(fib.Context(), 1*time.Second)
	defer cancel()

	// 1. Exchange the verifier for a one-time authorisation code.
	// 2. Provider validates the verifier against the challenge sent earlier.
	// 3. The returned token acts as a credential to get the user profile.
	// 4. User-unique subject is returned along with profile fields.
	profile, statusCode := getUserProviderProfile(
		ctx,
		this.OAuthConfig,
		authorisationCode,
		verifierCookie,
	)
	if statusCode != http.StatusOK {
		return fib.SendStatus(statusCode)
	}

	// 1. Set/update the user by its unique provider subject.
	// 2. Create a refresh token. Save it, keyed by the user.
	// 3. Create an access token. Save both tokens as cookies.
	if statusCode := saveUserToDBAndSession(
		ctx,
		fib,
		this,
		&profile,
	); statusCode != http.StatusOK {
		return fib.SendStatus(statusCode)
	}

	return fib.SendStatus(http.StatusNoContent)
}

func setOAuthSessionCookie(fib fiber.Ctx, name, value string, ageInSecs int) {
	fib.Cookie(&fiber.Cookie{
		Name:     name,
		Path:     "/auth/callback",
		Value:    value,
		MaxAge:   ageInSecs,
		Secure:   false, // NOTE: Should be changed for HTTPS traffic.
		HTTPOnly: true,
		SameSite: "Lax",
	})
}

func validateRedirectFromProvider(
	fib fiber.Ctx,
	providerState, authorisationCode, stateCookie, verifierCookie string,
) int {
	if err := fib.Query("error"); len(err) > 0 {
		zlog.Error().Str("err", err).Msg("Error found in provider redirect")
		return http.StatusBadRequest
	}

	if len(providerState) == 0 || len(authorisationCode) == 0 {
		zlog.Error().Msg("Missing state/authorisation code from provider")
		return http.StatusBadRequest
	}

	if len(stateCookie) == 0 || len(verifierCookie) == 0 {
		zlog.Error().Msg("Missing state/verifier set at login")
		return http.StatusBadRequest
	}

	// NOTE: Mitigate CSRF. Always compare every byte, not stopping at the first
	// mismatch, with [subtle.ConstantTimeCompare].
	if subtle.ConstantTimeCompare([]byte(providerState), []byte(stateCookie)) != 1 {
		zlog.Error().Msg("Mismatch between saved/provided state")
		return http.StatusBadRequest
	}

	return http.StatusOK
}

type providerProfile struct {
	Name  string `json:"name"`
	Email string `json:"email"`
	Sub   string `json:"sub"`
}

func getUserProviderProfile(
	ctx context.Context,
	oauthConfig *oauth2.Config,
	authorisationCode, verifierCookie string,
) (providerProfile, int) {
	providerToken, err := oauthConfig.Exchange(
		ctx,
		authorisationCode,
		oauth2.VerifierOption(verifierCookie),
	)
	if err != nil {
		zlog.Error().Err(err).Msg("Exchanging authorisation code for token")
		return providerProfile{}, http.StatusBadGateway
	}

	profileRequest, err := http.NewRequestWithContext(
		ctx,
		http.MethodGet,
		"http://mock-oauth:8080/default/userinfo",
		nil,
	)
	if err != nil {
		zlog.Error().Err(err).Msg("Building provider profile request")
		return providerProfile{}, http.StatusInternalServerError
	}

	profileResponse, err := oauthConfig.Client(ctx, providerToken).Do(profileRequest)
	if err != nil {
		zlog.Error().Err(err).Msg("Sending provider profile request")
		return providerProfile{}, http.StatusBadGateway
	}

	defer profileResponse.Body.Close()
	if profileResponse.StatusCode != http.StatusOK {
		zlog.Error().Msg("Bad status code returned from provider profile request")
		return providerProfile{}, http.StatusBadGateway
	}

	// Through OIDC, we've requested a certain shape of user profile. This isn't
	// a guarantee however i.e. a provider may not abide by OIDC.
	// NOTE: https://openid.net/developers/how-connect-works
	var profile providerProfile
	if err := json.NewDecoder(profileResponse.Body).Decode(&profile); err != nil {
		zlog.Error().Err(err).Msg("Decoding provider profile response")
		return providerProfile{}, http.StatusInternalServerError
	}

	if len(profile.Name) == 0 || len(profile.Email) == 0 || len(profile.Sub) == 0 {
		zlog.Error().Msg("Missing/empty data returned from provider profile request")
		return providerProfile{}, http.StatusBadGateway
	}

	return profile, http.StatusOK
}

func saveUserToDBAndSession(
	ctx context.Context,
	fib fiber.Ctx,
	handler *Handler,
	profile *providerProfile,
) int {
	savedUser, err := handler.UpsertUser(
		ctx,
		profile.Name,
		profile.Email,
		profile.Sub,
	)
	if err != nil {
		zlog.Error().Err(err).Msg("Upserting returned profile from provider")
		return http.StatusInternalServerError
	}

	accessToken, err := newTokenFromKind(savedUser.id, accessKind)
	if err != nil {
		zlog.Error().Err(err).Msg("Creating a new access token")
		return http.StatusInternalServerError
	}

	refreshToken, err := newTokenFromKind(savedUser.id, refreshKind)
	if err != nil {
		zlog.Error().Err(err).Msg("Creating a new refresh token")
		return http.StatusInternalServerError
	}

	if err := handler.CreateRefreshToken(
		ctx,
		refreshToken.hashFromValue(),
		savedUser.id,
		refreshToken.expiresAt,
	); err != nil {
		zlog.Error().Err(err).Msg("Saving the newly made refresh token")
		return http.StatusInternalServerError
	}

	zlog.Debug().Int64("userId", savedUser.id).Msg("Saved user from provider")
	accessToken.putIntoCookie(fib)
	refreshToken.putIntoCookie(fib)

	return http.StatusOK
}
