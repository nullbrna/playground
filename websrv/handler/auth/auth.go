package auth

import (
	"context"
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
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

	zlog.Trace().Str("to", redirectTo).Msg("Redirecting to OAuth provider")
	return fib.Redirect().Status(http.StatusFound).To(redirectTo)
}

func (this *Handler) callback(fib fiber.Ctx) error {
	stateCookie := fib.Cookies(oauthStateCookie)
	verifierCookie := fib.Cookies(oauthVerifierCookie)

	// Remove the temporary cookies made at login.
	// NOTE: Same path and empty value ensures browser deletes it.
	defer setOAuthSessionCookie(fib, oauthStateCookie, "", -1)
	defer setOAuthSessionCookie(fib, oauthVerifierCookie, "", -1)

	providerState, authorisationCode := fib.Query("state"), fib.Query("code")
	if err := validateRedirectFromProvider(
		fib,
		providerState,
		authorisationCode,
		stateCookie,
		verifierCookie,
	); err != nil {
		zlog.Error().Err(err).Msg("Invalid redirect from the provider")
		return fib.SendStatus(http.StatusBadRequest)
	}

	ctx, cancel := context.WithTimeout(fib.Context(), 1*time.Second)
	defer cancel()

	// 1. Exchange the verifier for a one-time authorisation code.
	// 2. Provider validates the verifier against the challenge sent earlier.
	// 3. The returned token acts as a credential to get the user profile.
	// 4. User-unique subject is returned along with profile fields.
	profile, err := getUserProviderProfile(
		ctx,
		this.OAuthConfig,
		authorisationCode,
		verifierCookie,
	)
	if err != nil {
		zlog.Error().Err(err).Msg("Fetching the user profile from the provider")
		return fib.SendStatus(http.StatusBadGateway)
	}

	// 1. Set/update the user by its unique provider subject.
	// 2. Create a refresh token. Save it, keyed by the user.
	// 3. Create an access token. Save both tokens as cookies.
	if err := saveUserToDBAndSession(
		ctx,
		fib,
		this,
		&profile,
	); err != nil {
		zlog.Error().Err(err).Msg("Saving user and setting their session")
		return fib.SendStatus(http.StatusBadGateway)
	}

	return fib.SendStatus(http.StatusNoContent)
}

func setOAuthSessionCookie(fib fiber.Ctx, name, value string, maxAge int) {
	newCookie := fiber.Cookie{
		Name:     name,
		Path:     "/auth/callback",
		Value:    value,
		MaxAge:   maxAge,
		Secure:   false, // NOTE: Should be changed for HTTPS traffic.
		HTTPOnly: true,
		SameSite: "Lax",
	}

	fib.Cookie(&newCookie)
}

func validateRedirectFromProvider(
	fib fiber.Ctx,
	providerState, authorisationCode, stateCookie, verifierCookie string,
) error {
	if len(providerState) == 0 || len(authorisationCode) == 0 {
		return errors.New("missing state/authorisation code from provider")
	}

	if len(stateCookie) == 0 || len(verifierCookie) == 0 {
		return errors.New("missing state/verifier set at login")
	}

	if subtle.ConstantTimeCompare([]byte(providerState), []byte(stateCookie)) != 1 {
		return errors.New("mismatch between stored/provided state")
	}

	if err := fib.Query("error"); len(err) > 0 {
		return fmt.Errorf("error found in provider redirect: %v", err)
	}

	return nil
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
) (providerProfile, error) {
	providerToken, err := oauthConfig.Exchange(
		ctx,
		authorisationCode,
		oauth2.VerifierOption(verifierCookie),
	)
	if err != nil {
		return providerProfile{}, err
	}

	profileRequest, err := http.NewRequestWithContext(
		ctx,
		http.MethodGet,
		"http://mock-oauth:8080/default/userinfo",
		nil,
	)
	if err != nil {
		return providerProfile{}, err
	}

	profileResponse, err := oauthConfig.Client(ctx, providerToken).Do(profileRequest)
	if err != nil {
		return providerProfile{}, err
	}

	defer profileResponse.Body.Close()
	if profileResponse.StatusCode != http.StatusOK {
		return providerProfile{}, errors.New("unexpected status code from provider")
	}

	var profile providerProfile
	if err := json.NewDecoder(profileResponse.Body).Decode(&profile); err != nil {
		return providerProfile{}, err
	}

	if len(profile.Name) == 0 || len(profile.Email) == 0 || len(profile.Sub) == 0 {
		return providerProfile{}, errors.New("empty data in user from provider")
	}

	return profile, nil
}

func saveUserToDBAndSession(
	ctx context.Context,
	fib fiber.Ctx,
	handler *Handler,
	profile *providerProfile,
) error {
	savedUser, err := handler.UpsertUser(
		ctx,
		profile.Name,
		profile.Email,
		profile.Sub,
	)
	if err != nil {
		return fmt.Errorf("trying to upsert user: %v", err)
	}

	accessToken, err := newTokenFromKind(savedUser.id, accessKind)
	if err != nil {
		return fmt.Errorf("creating a new access token: %v", err)
	}

	refreshToken, err := newTokenFromKind(savedUser.id, refreshKind)
	if err != nil {
		return fmt.Errorf("creating a new refresh token: %v", err)
	}

	if err := handler.CreateRefreshToken(
		ctx,
		refreshToken.hashFromValue(),
		savedUser.id,
		refreshToken.expiresAt,
	); err != nil {
		return fmt.Errorf("saving a new refresh token: %v", err)
	}

	accessToken.putIntoCookie(fib)
	refreshToken.putIntoCookie(fib)
	return nil
}
