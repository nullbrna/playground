package auth

import (
	"context"
	"net/http"
	"time"

	"github.com/rs/zerolog/log"
)

type user struct {
	id    int64
	name  string
	email string
}

func (this *InjectedHandler) upsertUserFromProviderProfile(
	requestContext context.Context,
	userProfile *providerProfile,
) (user, int) {
	const queryStatement = `
		INSERT INTO users (name, email, provider_sub)
		VALUES      ($1, $2, $3)
		ON CONFLICT (provider_sub) DO UPDATE
		SET         name = EXCLUDED.name, email = EXCLUDED.email
		RETURNING   id, name, email;
	`

	var upsertedUser user
	if err := this.DBPool.QueryRow(
		requestContext,
		queryStatement,
		userProfile.Name,
		userProfile.Email,
		userProfile.Sub,
	).Scan(&upsertedUser.id, &upsertedUser.name, &upsertedUser.email); err != nil {
		log.Error().Err(err).Msg("Upserting user profile from provider")
		return user{}, http.StatusInternalServerError
	}

	return upsertedUser, http.StatusOK
}

func (this *InjectedHandler) saveRefreshToken(
	requestContext context.Context,
	hash []byte,
	userId int64,
	expiresAt time.Time,
) int {
	const queryStatement = `
		INSERT INTO tokens (hash, user_id, expires_at)
		VALUES      ($1, $2, $3);
	`

	if _, err := this.DBPool.Exec(
		requestContext,
		queryStatement,
		hash,
		userId,
		expiresAt,
	); err != nil {
		log.Error().Err(err).Msg("Saving a new refresh token")
		return http.StatusInternalServerError
	}

	return http.StatusOK
}

func (this *InjectedHandler) deleteRefreshToken(
	requestContext context.Context,
	userId int64,
	hash []byte,
) int {
	const queryStatement = `
		DELETE FROM tokens
		WHERE       user_id = $1
		AND         hash    = $2;
	`

	tag, err := this.DBPool.Exec(requestContext, queryStatement, userId, hash)
	if err != nil {
		log.Error().Err(err).Msg("Deleting a refresh token")
		return http.StatusInternalServerError
	}

	if tag.RowsAffected() != 1 {
		log.Error().Msg("No saved refresh token found")
		return http.StatusUnauthorized
	}

	return http.StatusOK
}

func (this *InjectedHandler) rotateRefreshToken(
	requestContext context.Context,
	userId int64,
	oldHash, newHash []byte,
	expiresAt time.Time,
) int {
	const queryStatement = `
		UPDATE tokens
		SET    hash    = $1, expires_at = $2
		WHERE  user_id = $3
		AND    hash    = $4
		AND    expires_at > NOW();
	`

	tag, err := this.DBPool.Exec(requestContext, queryStatement, newHash, expiresAt, userId, oldHash)
	if err != nil {
		log.Error().Err(err).Msg("Rotating a refresh token")
		return http.StatusInternalServerError
	}

	if tag.RowsAffected() != 1 {
		log.Error().Msg("No saved refresh token to rotate")
		return http.StatusUnauthorized
	}

	return http.StatusOK
}
