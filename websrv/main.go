package main

import (
	"context"
	"os"
	"strings"
	"time"

	flog "github.com/gofiber/contrib/v3/zerolog"
	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/internal/handler"
	"github.com/nullbrna/playground/websrv/internal/store"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
)

func init() {
	var logLevel zerolog.Level

	fromEnv, found := os.LookupEnv("LOG_LEVEL")
	if !found || len(fromEnv) == 0 {
		logLevel = zerolog.TraceLevel
	}

	switch strings.ToLower(fromEnv) {
	case "trace":
		logLevel = zerolog.TraceLevel
	case "debug":
		logLevel = zerolog.DebugLevel
	case "warn":
		logLevel = zerolog.WarnLevel
	case "info":
		logLevel = zerolog.InfoLevel
	case "error":
		logLevel = zerolog.ErrorLevel
	default:
		logLevel = zerolog.TraceLevel
	}

	zlog.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger().Level(logLevel)
}

func newAndInitStore() store.Store {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	return store.New(ctx)
}

func parsePortFromEnv() string {
	port, found := os.LookupEnv("PORT")
	if !found || len(port) == 0 {
		return ":8080"
	}

	return ":" + port
}

func main() {
	logMiddleware := flog.New(flog.Config{
		Logger: &zlog.Logger,
		Levels: []zerolog.Level{
			zerolog.ErrorLevel, // All 5XX codes.
			zerolog.WarnLevel,  // All 4XX codes.
			zerolog.DebugLevel, // Everything else.
		},
		Fields: []string{
			flog.FieldIP,
			flog.FieldLatency,
			flog.FieldStatus,
			flog.FieldMethod,
			flog.FieldPath,
			flog.FieldError,
		},
	})

	app := fiber.New()
	app.Use(logMiddleware)

	store := newAndInitStore()
	defer store.Close()

	var handlers handler.Handler
	handlers.Init(app, &store)

	port := parsePortFromEnv()
	zlog.Info().Msg("Starting server...")

	if err := app.Listen(port); err != nil {
		zlog.Error().Err(err).Msg("Server stopped abruptly")
	}
}
