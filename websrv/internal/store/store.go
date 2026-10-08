package store

import (
	"context"
	"errors"
	"net/http"
	"os"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	zlog "github.com/rs/zerolog/log"
)

const dbEnvKey = "DATABASE_URI"

type Store struct {
	pool *pgxpool.Pool
}

func New(ctx context.Context) Store {
	dbUri, found := os.LookupEnv(dbEnvKey)
	if !found || len(dbUri) == 0 {
		zlog.Fatal().Str("key", dbEnvKey).Msg("Missing/empty variable")
	}

	cfg, err := pgxpool.ParseConfig(dbUri)
	if err != nil {
		zlog.Fatal().Err(err).Str("uri", dbEnvKey).Msg("Parsing URI")
	}

	cfg.MinConns = 5
	cfg.MaxConns = 25

	// NOTE: Ignore the passed context deadline so all minimum connections are
	// loaded. Connections warm up asynchronously.
	pool, err := pgxpool.NewWithConfig(context.Background(), cfg)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating database pool")
	}

	if err := pool.Ping(ctx); err != nil {
		pool.Close()
		zlog.Fatal().Err(err).Msg("Pinging database upon creation")
	}

	return Store{pool}
}

func (this *Store) Close() {
	this.pool.Close()
}

type User struct {
	Name  string `db:"name" json:"name"`
	Email string `db:"email" json:"email"`
}

func (this *Store) GetFirstTenUsers(ctx context.Context) ([]User, int) {
	statement := `
	SELECT name, email
	FROM users
	LIMIT 10;
	`

	rows, err := this.pool.Query(ctx, statement)
	if err != nil {
		zlog.Error().Err(err).Msg("Fetching 10 users")
		return nil, http.StatusInternalServerError
	}

	users, err := pgx.CollectRows(rows, pgx.RowToStructByName[User])
	if err != nil {
		zlog.Error().Err(err).Msg("Parsing 10 user rows")
		return nil, http.StatusInternalServerError
	}

	return users, http.StatusOK
}

func (this *Store) GetUserById(ctx context.Context, id int64) (User, int) {
	statement := `
	SELECT name, email
	FROM users
	WHERE id = $1;
	`

	rows, err := this.pool.Query(ctx, statement, id)
	if err != nil {
		zlog.Error().Err(err).Int64("id", id).Msg("Fetching user by ID")
		return User{}, http.StatusInternalServerError
	}

	user, err := pgx.CollectExactlyOneRow(rows, pgx.RowToStructByName[User])
	if errors.Is(err, pgx.ErrNoRows) {
		zlog.Warn().Int64("id", id).Msg("Missing user by ID")
		return User{}, http.StatusNotFound
	} else if err != nil {
		zlog.Error().Err(err).Int64("id", id).Msg("Parsing user row by ID")
		return User{}, http.StatusInternalServerError
	}

	return user, http.StatusOK
}

func (this *Store) GetUserCount(ctx context.Context) (int64, int) {
	statement := `
	SELECT COUNT(*) FROM users;
	`

	var count int64
	err := this.pool.QueryRow(ctx, statement).Scan(&count)
	if err != nil {
		zlog.Error().Err(err).Msg("Counting total users")
		return -1, http.StatusInternalServerError
	}

	return count, http.StatusOK
}
