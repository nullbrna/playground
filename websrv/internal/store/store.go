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
		zlog.Fatal().Str("envKey", dbUriKey).Msg("Missing/empty variable")
	}

	poolCfg, err := pgxpool.ParseConfig(dbUri)
	if err != nil {
		zlog.Fatal().Err(err).Str("uri", dbUri).Msg("Parsing URI")
	}

	poolCfg.MinConns = 5
	poolCfg.MaxConns = 25

	// NOTE: Replace the passed context deadline with [context.Background] so
	// all minimum connections are loaded. Connections warm up asynchronously.
	dbPool, err := pgxpool.NewWithConfig(context.Background(), poolCfg)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating database pool")
	}

	if err := dbPool.Ping(ctx); err != nil {
		dbPool.Close()
		zlog.Fatal().Err(err).Msg("Pinging database upon creation")
	}

	return Store{dbPool}
}

func (this *Store) Close() {
	this.dbPool.Close()
}
