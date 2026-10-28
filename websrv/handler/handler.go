package handler

import (
	"context"
	"os"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nullbrna/playground/websrv/handler/auth"
	"github.com/rs/zerolog/log"
	"golang.org/x/oauth2"
)

func InitialiseRoutesAndHandlers(fiberInstance *fiber.App) {
	dependencySetupContext, setupContextCancel := context.WithTimeout(
		context.Background(),
		5*time.Second,
	)
	defer setupContextCancel()

	var authRootHandler auth.InjectedHandler
	authRootHandler.DBPool = newAndConnectedDBPool(dependencySetupContext)
	authRootHandler.OAuthConfig = newInitialisedOAuthConfig()

	auth.RegisterRoutes(fiberInstance.Group("/auth"), authRootHandler)
}

func newAndConnectedDBPool(dependencySetupContext context.Context) *pgxpool.Pool {
	const databaseURIKey = "DATABASE_URI"

	databaseURIFromEnv, databaseURIFound := os.LookupEnv(databaseURIKey)
	if !databaseURIFound || len(databaseURIFromEnv) == 0 {
		log.Fatal().Str("key", databaseURIKey).Msg("Variable must be set/non-zero in length")
	}

	poolConfigFromURIString, err := pgxpool.ParseConfig(databaseURIFromEnv)
	if err != nil {
		log.Fatal().Err(err).Msg("Creating/parsing pool configuration")
		return nil // NOTE: No-op as [log.Fatal] exits code 1.
	}

	poolConfigFromURIString.MinConns = 1
	poolConfigFromURIString.MaxConns = 20

	// NOTE: Use [context.Background] instead of the given context. The minimum
	// connections are created asynchronously, so no need for a timeout.
	configuredDBPool, err := pgxpool.NewWithConfig(context.Background(), poolConfigFromURIString)
	if err != nil {
		log.Fatal().Err(err).Msg("Creating database pool")
		return nil // NOTE: No-op as [log.Fatal] exits code 1.
	}

	if err := configuredDBPool.Ping(dependencySetupContext); err != nil {
		log.Fatal().Err(err).Msg("Connecting/pinging database on creation")
		return nil // NOTE: No-op as [log.Fatal] exits code 1.
	}

	return configuredDBPool
}

func newInitialisedOAuthConfig() *oauth2.Config {
	var initialisedOAuthConfig oauth2.Config
	initialisedOAuthConfig.ClientID = "client-identifier"
	initialisedOAuthConfig.Scopes = []string{"openid", "profile", "email"}
	initialisedOAuthConfig.RedirectURL = "http://localhost:8080/auth/callback"

	var providerSpecificEndpoints oauth2.Endpoint
	providerSpecificEndpoints.AuthURL = "http://localhost:7654/default/authorize"
	providerSpecificEndpoints.TokenURL = "http://mock-oauth:8080/default/token"

	initialisedOAuthConfig.Endpoint = providerSpecificEndpoints
	return &initialisedOAuthConfig
}
