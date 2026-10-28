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

func localOnlyLogger() zerolog.Logger {
	var localOnlyWriter zerolog.ConsoleWriter
	localOnlyWriter.Out = os.Stdout
	localOnlyWriter.TimeFormat = time.TimeOnly

	return zerolog.New(localOnlyWriter).With().Timestamp().Logger()
}

func initialiseLoggerWithinApp(fiberInstance *fiber.App) {
	appEnvType, appEnvFound := os.LookupEnv("APP_ENV")
	if appEnvFound && len(appEnvType) > 0 {
		switch strings.ToLower(appEnvType) {
		case "live":
			log.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger()
		default:
			log.Logger = localOnlyLogger()
		}
	} else {
		log.Logger = localOnlyLogger()
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
		// Replace [zlogcont.FieldURL] to avoid a potential leak.
		zlogcont.FieldRoute,
		zlogcont.FieldError,
	}
	logIntegrationConfig.Levels = []zerolog.Level{
		zerolog.ErrorLevel, // 5XX status codes.
		zerolog.WarnLevel,  // 4XX status codes.
		zerolog.DebugLevel, // XXX status codes.
	}

	fiberInstance.Use(zlogcont.New(logIntegrationConfig))
}

func main() {
	fiberInstance := fiber.New()
	initialiseLoggerWithinApp(fiberInstance)

	handler.InitialiseRoutesAndHandlers(fiberInstance)
	if err := fiberInstance.Listen(":8080"); err != nil {
		log.Fatal().Err(err).Msg("Root error while running server")
	}
}
