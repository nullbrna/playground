package auth

import (
	"context"
	"time"
)

type user struct {
	id    int64
	name  string
	email string
}

func (this *Handler) UpsertUser(
	ctx context.Context,
	name, email, providerSub string,
) (user, error) {
	const queryStatement = `
		INSERT INTO users (name, email, provider_sub)
		VALUES ($1, $2, $3)
		ON CONFLICT (provider_sub) DO UPDATE
		SET name = EXCLUDED.name, email = EXCLUDED.email
		RETURNING id, name, email;
	`

	var upsertedUser user
	err := this.DBPool.QueryRow(ctx, queryStatement, name, email, providerSub).Scan(
		&upsertedUser.id,
		&upsertedUser.name,
		&upsertedUser.email,
	)

	return upsertedUser, err
}

func (this *Handler) CreateRefreshToken(
	ctx context.Context,
	hash []byte,
	userID int64,
	expiresAt time.Time,
) error {
	const queryStatement = `
		INSERT INTO tokens (hash, user_id, expires_at)
		VALUES ($1, $2, $3);
	`

	_, err := this.DBPool.Exec(ctx, queryStatement, hash, userID, expiresAt)
	return err
}
