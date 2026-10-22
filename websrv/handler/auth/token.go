package auth

import (
	"crypto/rand"
	"crypto/sha256"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/golang-jwt/jwt/v5"
	zlog "github.com/rs/zerolog/log"
)

const accessTokenLifetime = 15 * time.Minute
const refreshTokenLifetime = 30 * 24 * time.Hour

type tokenKind uint8

const (
	accessKind tokenKind = iota
	refreshKind
)

func (this tokenKind) lifetimeOffset(now time.Time) time.Time {
	switch this {
	case accessKind:
		return now.Add(accessTokenLifetime)
	case refreshKind:
		return now.Add(refreshTokenLifetime)
	default:
		return now
	}
}

func (this tokenKind) lifetimeAsSecs() float64 {
	switch this {
	case accessKind:
		return accessTokenLifetime.Seconds()
	case refreshKind:
		return refreshTokenLifetime.Seconds()
	default:
		return 0.0
	}
}

func (this tokenKind) cookieName() string {
	switch this {
	case accessKind:
		return "token.access"
	case refreshKind:
		return "token.refresh"
	default:
		zlog.Fatal().Msg("Unknown case for token cookie name")
		return "" // NOTE: No-op as [zlog.Fatal] exits code 1.
	}
}

var tokenSecret = []byte("super-secret")

type token struct {
	kind      tokenKind
	value     string
	expiresAt time.Time
}

type claims struct {
	UserID int64     `json:"userId"`
	Kind   tokenKind `json:"kind"`
	jwt.RegisteredClaims
}

func newTokenFromKind(userID int64, kind tokenKind) (token, error) {
	issuedAt := time.Now()
	expiresAt := kind.lifetimeOffset(issuedAt)

	value, err := jwt.NewWithClaims(jwt.SigningMethodHS256, claims{
		UserID:    userID,
		Kind:      kind,
		ID:        rand.Text(),
		IssuedAt:  jwt.NewNumericDate(issuedAt),
		ExpiresAt: jwt.NewNumericDate(expiresAt),
	}).SignedString(tokenSecret)

	return token{kind, value, expiresAt}, err
}

func (this *token) hashFromValue() []byte {
	valueChecksum := sha256.Sum256([]byte(this.value))
	return valueChecksum[:]
}

func (this *token) putIntoCookie(fib fiber.Ctx) {
	ageInSecs := this.kind.lifetimeAsSecs()

	fib.Cookie(&fiber.Cookie{
		Name:     this.kind.cookieName(),
		Path:     "/",
		Value:    this.value,
		MaxAge:   int(ageInSecs),
		Secure:   false, // NOTE: Should be changed for HTTPS traffic.
		HTTPOnly: true,
		SameSite: "Lax",
	})
}
