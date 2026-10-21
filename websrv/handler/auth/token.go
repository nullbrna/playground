package auth

import (
	"crypto/rand"
	"crypto/sha256"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/golang-jwt/jwt/v5"
)

const accessTokenExpiresAt = 15 * time.Minute
const refreshTokenExpiresAt = 30 * 24 * time.Hour

type tokenKind uint8

const (
	accessKind tokenKind = iota
	refreshKind
)

type token struct {
	kind      tokenKind
	value     string
	expiresAt time.Time
}

type claims struct {
	UserID    int64     `json:"userId"`
	TokenKind tokenKind `json:"tokenKind"`
	jwt.RegisteredClaims
}

func newTokenFromKind(userID int64, kind tokenKind) (token, error) {
	issuedAt := time.Now()

	var expiresAt time.Time
	switch kind {
	case accessKind:
		expiresAt = issuedAt.Add(accessTokenExpiresAt)
	case refreshKind:
		expiresAt = issuedAt.Add(refreshTokenExpiresAt)
	default:
		expiresAt = issuedAt // For an unknown case, immediately expire.
	}

	claims := claims{
		UserID:    userID,
		TokenKind: kind,
		ID:        rand.Text(),
		IssuedAt:  jwt.NewNumericDate(issuedAt),
		ExpiresAt: jwt.NewNumericDate(expiresAt),
	}

	tokenSecret := []byte("super-secret")
	value, err := jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(tokenSecret)

	return token{kind, value, expiresAt}, err
}

func (this *token) hashFromValue() []byte {
	checksum := sha256.Sum256([]byte(this.value))
	return checksum[:]
}

func (this *token) putIntoCookie(fib fiber.Ctx) {
	var cookieName string
	var cookieAge float64

	switch this.kind {
	case accessKind:
		cookieName = "token.access"
		cookieAge = accessTokenExpiresAt.Seconds()
	case refreshKind:
		cookieName = "token.refresh"
		cookieAge = refreshTokenExpiresAt.Seconds()
	default:
		cookieName = ""
		cookieAge = 0.0 // For an unknown case, immediately expire.
	}

	newCookie := fiber.Cookie{
		Name:     cookieName,
		Path:     "/",
		Value:    this.value,
		MaxAge:   int(cookieAge),
		Secure:   false, // NOTE: Should be changed for HTTPS traffic.
		HTTPOnly: true,
		SameSite: "Lax",
	}

	fib.Cookie(&newCookie)
}
