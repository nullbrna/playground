package handler

import (
	"context"
	"os"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nullbrna/playground/websrv/handler/auth"
	zlog "github.com/rs/zerolog/log"
	"golang.org/x/oauth2"
)

func InitRoutes(app *fiber.App) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	dbPool := newConnectedDBPool(ctx)
	oauthConfig := newInitialisedOAuthConfig()

	auth.RegisterRoutes(app.Group("/auth"), auth.Handler{
		DBPool:      dbPool,
		OAuthConfig: &oauthConfig,
	})
}

func newConnectedDBPool(ctx context.Context) *pgxpool.Pool {
	databaseURIFromEnv, uriFound := os.LookupEnv("DATABASE_URI")
	if !uriFound || len(databaseURIFromEnv) == 0 {
		zlog.Fatal().Str("key", "DATABASE_URI").Msg("Variable must be set/non-zero in length")
	}

	poolConfig, err := pgxpool.ParseConfig(databaseURIFromEnv)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating/parsing pool configuration")
		return nil // NOTE: No-op as [zlog.Fatal] exits code 1.
	}

	poolConfig.MinConns = 1
	poolConfig.MaxConns = 20

	// NOTE: We don't pass the given context, instead use [context.Background].
	// The minimum connections are created asynchronously, the given context can
	// cancel before it finishes making said connections.
	dbPool, err := pgxpool.NewWithConfig(context.Background(), poolConfig)
	if err != nil {
		zlog.Fatal().Err(err).Msg("Creating database pool")
		return nil // NOTE: No-op as [zlog.Fatal] exits code 1.
	}

	if err := dbPool.Ping(ctx); err != nil {
		zlog.Fatal().Err(err).Msg("Connecting/pinging database on creation")
		return nil // NOTE: No-op as [zlog.Fatal] exits code 1.
	}

	return dbPool
}

func newInitialisedOAuthConfig() oauth2.Config {
	newConfig := oauth2.Config{
		ClientID: "client-identifier",
		Scopes:   []string{"openid", "profile", "email"},
		// NOTE: These URLs should come from the environment.
		RedirectURL: "http://localhost:8080/auth/callback",
		Endpoint: oauth2.Endpoint{
			AuthURL:  "http://localhost:7654/default/authorize",
			TokenURL: "http://mock-oauth:8080/default/token",
		},
	}

	return newConfig
}
