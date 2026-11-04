package main

import (
	"os"
	"strings"
	"time"

	zlogcont "github.com/gofiber/contrib/v3/zerolog"
	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/handler"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func initialiseLoggerWithinApp(fiberApp *fiber.App) {
	appEnvFromEnv, appEnvFound := os.LookupEnv("APP_ENV")
	if appEnvFound && strings.ToLower(appEnvFromEnv) == "live" {
		log.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger()
	} else {
		var localOnlyWriter zerolog.ConsoleWriter
		localOnlyWriter.Out = os.Stdout
		localOnlyWriter.TimeFormat = time.TimeOnly

		log.Logger = zerolog.New(localOnlyWriter).With().Timestamp().Logger()
	}

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

	var logIntegrationConfig zlogcont.Config
	logIntegrationConfig.Logger = &log.Logger
	logIntegrationConfig.Fields = []string{
		zlogcont.FieldLatency,
		zlogcont.FieldStatus,
		zlogcont.FieldMethod,
		zlogcont.FieldRoute,
		zlogcont.FieldError,
	}
	logIntegrationConfig.Levels = []zerolog.Level{
		zerolog.ErrorLevel, // 5XX status code(s).
		zerolog.WarnLevel,  // 4XX status code(s).
		zerolog.DebugLevel, // XXX status code(s).
	}

	fiberApp.Use(zlogcont.New(logIntegrationConfig))
}

func main() {
	fiberInstance := fiber.New()
	initialiseLoggerWithinApp(fiberInstance)

	handler.InitialiseRoutesAndHandlers(fiberInstance)
	if err := fiberInstance.Listen(":8080"); err != nil {
		log.Fatal().Err(err).Msg("Root error while running server")
	}
}
