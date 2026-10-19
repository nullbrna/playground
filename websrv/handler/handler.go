package handler

import (
	"context"
	"net/http"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/store"
	zlog "github.com/rs/zerolog/log"
)

type handler struct {
	store *store.Store
}

func InitRoutes(app *fiber.App) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	handlerStore := store.New(ctx)
	var appHandlers handler
	appHandlers.store = &handlerStore

	app.Get("/", appHandlers.indexRoute)
}

func (this *handler) indexRoute(fib fiber.Ctx) error {
	ctx, cancel := context.WithTimeout(fib.Context(), 1*time.Second)
	defer cancel()

	if err := this.store.HealthCheck(ctx); err != nil {
		zlog.Error().Err(err).Msg("Health check on the database")
		return fib.SendStatus(http.StatusServiceUnavailable)
	}

	return fib.SendStatus(http.StatusOK)
}
