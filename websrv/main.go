package main

import (
	"os"
	"strings"

	zlogcontrib "github.com/gofiber/contrib/v3/zerolog"
	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/handler"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
)

func initialiseLoggerWithinApp(app *fiber.App) {
	zlog.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger()

	levelFromEnv, levelFound := os.LookupEnv("LOG_LEVEL")
	if levelFound && len(levelFromEnv) > 0 {
		switch strings.ToLower(levelFromEnv) {
		case "trace":
			zerolog.SetGlobalLevel(zerolog.TraceLevel)
		case "debug":
			zerolog.SetGlobalLevel(zerolog.DebugLevel)
		case "warn":
			zerolog.SetGlobalLevel(zerolog.WarnLevel)
		case "info":
			zerolog.SetGlobalLevel(zerolog.InfoLevel)
		case "error":
			zerolog.SetGlobalLevel(zerolog.ErrorLevel)
		default:
			zerolog.SetGlobalLevel(zerolog.DebugLevel)
		}
	} else {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
	}

	loggerIntegration := zlogcontrib.New(zlogcontrib.Config{
		Logger: &zlog.Logger,
		Fields: []string{
			zlogcontrib.FieldLatency,
			zlogcontrib.FieldStatus,
			zlogcontrib.FieldMethod,
			// Although the others are all default, we need to replace
			// [zlogcontrib.FieldURL] due to potential sensitive information.
			zlogcontrib.FieldRoute,
			zlogcontrib.FieldError,
		},
		Levels: []zerolog.Level{
			zerolog.ErrorLevel, // 5XX status codes.
			zerolog.WarnLevel,  // 4XX status codes.
			zerolog.DebugLevel, // XXX status codes.
		},
	})

	app.Use(loggerIntegration)
}

func main() {
	app := fiber.New()
	initialiseLoggerWithinApp(app)

	handler.InitRoutes(app)
	if err := app.Listen(":8080"); err != nil {
		zlog.Fatal().Err(err).Msg("Root error while running server")
	}
}
