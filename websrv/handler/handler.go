package handler

import (
	"context"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/store"
	"golang.org/x/oauth2"
)

type handler struct {
	store       *store.Store
	oauthConfig oauth2.Config
}

func InitRoutes(app *fiber.App) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	handlerStore := store.New(ctx)
	oauthConfig := newInitialisedOAuthConfig()

	var appHandlers handler
	appHandlers.store = &handlerStore
	appHandlers.oauthConfig = oauthConfig

	app.Get("/auth/login", appHandlers.loginRoute)
}

func newInitialisedOAuthConfig() oauth2.Config {
	var newConfig oauth2.Config
	newConfig.ClientID = "client-identifier"
	// [NOTE] These URLs should come from the environment.
	newConfig.RedirectURL = "http://localhost:8080/auth/callback"

	var oauthProviderEndpoints oauth2.Endpoint
	oauthProviderEndpoints.AuthURL = "http://localhost:7654/default/authorize"
	oauthProviderEndpoints.TokenURL = "http://mock-oauth:8080/default/token"

	newConfig.Endpoint = oauthProviderEndpoints
	return newConfig
}
