package index

import (
	"context"
	"net/http"

	"github.com/gofiber/fiber/v3"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/rs/zerolog/log"
)

type Handler struct {
	DBPool *pgxpool.Pool
}

func RegisterRoutes(parentRouter fiber.Router, rootHandler Handler) {
	parentRouter.Get("/", rootHandler.index)
}

func (this *Handler) index(fiberCtx fiber.Ctx) error {
	if err := databaseHealthCheck(fiberCtx.Context(), this.DBPool); err != nil {
		log.Error().Err(err).Msg("Performing DB health check")
		return fiberCtx.SendStatus(http.StatusInternalServerError)
	}

	return fiberCtx.SendStatus(http.StatusOK)
}

func databaseHealthCheck(requestContext context.Context, pool *pgxpool.Pool) error {
	const queryStatement = `
		SELECT 1;
	`

	var checkResult int
	if err := pool.QueryRow(requestContext, queryStatement).Scan(&checkResult); err != nil {
		return err
	}

	return nil
}
