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
	var newStore Store

	databaseURIFromEnv, uriFound := os.LookupEnv("DATABASE_URI")
	if !uriFound || len(databaseURIFromEnv) == 0 {
		zlog.Fatal().Str("key", "DATABASE_URI").Msg("Variable must be set/non-zero in length")
	}

	poolConfig, err := pgxpool.ParseConfig(databaseURIFromEnv)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating/parsing pool configuration")
	}

	poolConfig.MinConns = 1
	poolConfig.MaxConns = 20

	// [NOTE] We don't pass the given context, instead use [context.Background].
	// The minimum connections are created asynchronously, the given context can
	// cancel before it finishes making said connections.
	dbPool, err := pgxpool.NewWithConfig(context.Background(), poolConfig)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating database pool")
	}

	if err := dbPool.Ping(ctx); err != nil {
		zlog.Fatal().Err(err).Msg("Connecting/pinging database on creation")
	}

	newStore.pool = dbPool
	return newStore
}

func (this *Store) HealthCheck(ctx context.Context) error {
	queryStatement := `
		SELECT 1;
	`

	var result int32
	return this.pool.QueryRow(ctx, queryStatement).Scan(&result)
}
