package auth

import (
	"crypto/rand"
	"crypto/sha256"
	"errors"
	"net/http"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/golang-jwt/jwt/v5"
	"github.com/rs/zerolog/log"
)

const (
	accessTokenLifetime  = 15 * time.Minute
	refreshTokenLifetime = 30 * 24 * time.Hour
)

type authTokenKind uint8

const (
	accessTokenKind authTokenKind = iota
	refreshTokenKind
)

func (this authTokenKind) lifetimeOffset(issuedAt time.Time) time.Time {
	switch this {
	case accessTokenKind:
		return issuedAt.Add(accessTokenLifetime)
	case refreshTokenKind:
		return issuedAt.Add(refreshTokenLifetime)
	default:
		return issuedAt
	}
}

func (this authTokenKind) lifetimeAsSecs() float64 {
	switch this {
	case accessTokenKind:
		return accessTokenLifetime.Seconds()
	case refreshTokenKind:
		return refreshTokenLifetime.Seconds()
	default:
		return 0.0
	}
}

func (this authTokenKind) cookieName() string {
	switch this {
	case accessTokenKind:
		return "token.access"
	case refreshTokenKind:
		return "token.refresh"
	default:
		log.Fatal().Msg("Unknown case for token cookie name")
		return "" // NOTE: No-op as [log.Fatal] exits code 1.
	}
}

func (this authTokenKind) cookiePath() string {
	switch this {
	case accessTokenKind:
		return "/"
	case refreshTokenKind:
		return "/auth"
	default:
		log.Fatal().Msg("Unknown case for token cookie path")
		return "" // NOTE: No-op as [log.Fatal] exits code 1.
	}
}

var authTokenSecret = []byte("super-secret")

type authToken struct {
	kind      authTokenKind
	value     string
	expiresAt time.Time
}

func (this *authToken) intoCookieForResponse(requestInstance fiber.Ctx) {
	var cookieToSet fiber.Cookie
	cookieToSet.Name = this.kind.cookieName()
	cookieToSet.Path = this.kind.cookiePath()
	cookieToSet.Value = this.value
	cookieToSet.Secure = false
	cookieToSet.HTTPOnly = true
	cookieToSet.SameSite = "Lax"

	ageInSeconds := this.kind.lifetimeAsSecs()
	// NOTE: Float to integer rounds toward zero.
	cookieToSet.MaxAge = int(ageInSeconds)

	requestInstance.Cookie(&cookieToSet)
}

func hashTokenValue(tokenValue string) []byte {
	tokenHashChecksum := sha256.Sum256([]byte(tokenValue))
	return tokenHashChecksum[:]
}

type authClaims struct {
	UserId    int64         `json:"userId"`
	TokenKind authTokenKind `json:"tokenKind"`
	jwt.RegisteredClaims
}

func newTokenPair(userId int64) (authToken, authToken, int) {
	pairIssuedAt := time.Now()
	tokenBuilder := func(tokenKind authTokenKind) (authToken, error) {
		tokenExpiryTime := tokenKind.lifetimeOffset(pairIssuedAt)

		var claimsToEncode authClaims
		claimsToEncode.UserId = userId
		claimsToEncode.ID = rand.Text()
		claimsToEncode.IssuedAt = jwt.NewNumericDate(pairIssuedAt)
		claimsToEncode.ExpiresAt = jwt.NewNumericDate(tokenExpiryTime)

		tokenValue, err := jwt.NewWithClaims(
			jwt.SigningMethodHS256,
			claimsToEncode,
		).SignedString(authTokenSecret)

		var newAuthToken authToken
		newAuthToken.kind = tokenKind
		newAuthToken.value = tokenValue
		newAuthToken.expiresAt = tokenExpiryTime

		return newAuthToken, err
	}

	newAccessToken, err := tokenBuilder(accessTokenKind)
	if err != nil {
		log.Error().Err(err).Msg("Creating a new access token")
		return authToken{}, authToken{}, http.StatusInternalServerError
	}

	newRefreshToken, err := tokenBuilder(refreshTokenKind)
	if err != nil {
		log.Error().Err(err).Msg("Creating a new refresh token")
		return authToken{}, authToken{}, http.StatusInternalServerError
	}

	return newAccessToken, newRefreshToken, http.StatusOK
}

func decodeTokenFromKind(tokenValue string, tokenKind authTokenKind) (int64, int) {
	tokenValidator := func(tokenToValidate *jwt.Token) (any, error) {
		if tokenToValidate.Method != jwt.SigningMethodHS256 {
			return nil, errors.New("incorrect token method when decoding")
		}

		return authTokenSecret, nil
	}

	decodedClaims, acceptedMethods := new(authClaims), []string{jwt.SigningMethodHS256.Alg()}
	parseResult, err := jwt.ParseWithClaims(
		tokenValue,
		decodedClaims,
		tokenValidator,
		jwt.WithExpirationRequired(),
		jwt.WithLeeway(60*time.Second),
		jwt.WithValidMethods(acceptedMethods),
	)
	if err != nil {
		log.Error().Err(err).Msg("Parsing token into claims")
		return -1, http.StatusUnauthorized
	}

	if parseResult == nil || !parseResult.Valid || decodedClaims.TokenKind != tokenKind {
		log.Error().Err(err).Msg("Invalid structure of decoded claims")
		return -1, http.StatusUnauthorized
	}

	return decodedClaims.UserId, http.StatusOK
}
