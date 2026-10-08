package main

import (
	"os"
	"time"

	rl "github.com/gen2brain/raylib-go/raylib"
	"github.com/rs/zerolog"
	zlog "github.com/rs/zerolog/log"
)

const (
	windowWidth  int32 = 640
	windowHeight int32 = 480

	fontSize     int32   = 24
	textMargin   int32   = 12
	circleRadius float32 = 48
)

func init() {
	logOutput := zerolog.ConsoleWriter{
		Out:        os.Stdout,
		TimeFormat: time.TimeOnly,
	}

	zlog.Logger = zerolog.New(logOutput).With().Timestamp().Logger().Level(zerolog.DebugLevel)
}

func main() {
	rl.SetTraceLogLevel(rl.LogWarning)
	rl.SetConfigFlags(rl.FlagWindowResizable)
	rl.SetConfigFlags(rl.FlagMsaa4xHint)

	rl.InitWindow(windowWidth, windowHeight, "Hello World")
	defer rl.CloseWindow()

	rl.SetTargetFPS(60)
	greeting := "Hello, world!"

	zlog.Info().Int32("width", windowWidth).Int32("height", windowHeight).Msg("Starting game...")
	for !rl.WindowShouldClose() {
		width, height := int32(rl.GetScreenWidth()), int32(rl.GetScreenHeight())

		rl.BeginDrawing()
		rl.ClearBackground(rl.Black)

		rl.DrawCircle(width/2, height/2, circleRadius, rl.Green)

		textWidth := rl.MeasureText(greeting, fontSize)
		rl.DrawText(greeting, width-textWidth-textMargin, textMargin, fontSize, rl.DarkGray)

		rl.DrawFPS(10, 10)
		rl.EndDrawing()
	}
}
