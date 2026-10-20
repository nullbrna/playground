package store

import (
	"context"
	"os"

	"github.com/jackc/pgx/v5/pgxpool"
	zlog "github.com/rs/zerolog/log"
)

type Store struct {
	pool *pgxpool.Pool
}

func New(ctx context.Context) Store {
	databaseURIFromEnv, uriFound := os.LookupEnv("DATABASE_URI")
	if !uriFound || len(databaseURIFromEnv) == 0 {
		zlog.Fatal().Str("key", "DATABASE_URI").Msg("Variable must be set/non-zero in length")
	}

	var newStore Store

	poolConfig, err := pgxpool.ParseConfig(databaseURIFromEnv)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating/parsing pool configuration")
		// [NOTE] No-op as the above log exits with status code 1. Here to keep
		// static analysis tools happy.
		return newStore
	}

	poolConfig.MinConns = 1
	poolConfig.MaxConns = 20

	// [NOTE] We don't pass the given context, instead use [context.Background].
	// The minimum connections are created asynchronously, the given context can
	// cancel before it finishes making said connections.
	newStore.pool, err = pgxpool.NewWithConfig(context.Background(), poolConfig)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating database pool")
		// [NOTE] No-op as the above log exits with status code 1. Here to keep
		// static analysis tools happy.
		return newStore
	}

	if err := newStore.pool.Ping(ctx); err != nil {
		zlog.Fatal().Err(err).Msg("Connecting/pinging database on creation")
		// [NOTE] No-op as the above log exits with status code 1. Here to keep
		// static analysis tools happy.
		return newStore
	}

	return newStore
}
