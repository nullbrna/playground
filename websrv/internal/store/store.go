package store

import (
	"context"
	"os"

	"github.com/jackc/pgx/v5/pgxpool"
	zlog "github.com/rs/zerolog/log"
)

type Store struct {
	dbPool *pgxpool.Pool
}

func New(ctx context.Context) Store {
	const dbUriKey = "DATABASE_URI"

	dbUri, found := os.LookupEnv(dbUriKey)
	if !found || len(dbUri) == 0 {
		zlog.Fatal().Str("key", dbUriKey).Msg("Missing/empty variable")
	}

	cfg, err := pgxpool.ParseConfig(dbUri)
	if err != nil {
		zlog.Fatal().Err(err).Str("uri", dbUri).Msg("Parsing URI")
	}

	cfg.MinConns = 5
	cfg.MaxConns = 25

	// NOTE: Ignore the passed context deadline with [context.Background] so all
	// minimum connections are loaded. Connections warm up asynchronously.
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
	this.dbPool.Close()
}
