package main

import rl "github.com/gen2brain/raylib-go/raylib"

const (
	windowTitle      string = "Hello World"
	baseWindowWidth  int32  = 800
	baseWindowHeight int32  = 600

	fontSize     int32   = 20
	textPadding  int32   = 10
	circleRadius float32 = 30.0

	greeting string = "Hello, world!"
)

type WindowDimGeneric interface {
	~int32 | ~float32
}

func getWindowDimensions[T WindowDimGeneric]() (T, T) {
	width, height := rl.GetScreenWidth(), rl.GetScreenHeight()
	return T(width), T(height)
}

func main() {
	rl.SetConfigFlags(rl.FlagWindowResizable)
	rl.SetTraceLogLevel(rl.LogWarning)

	rl.InitWindow(baseWindowWidth, baseWindowHeight, windowTitle)
	defer rl.CloseWindow()

	rl.SetTargetFPS(60)
	for !rl.WindowShouldClose() {
		windowWidth, windowHeight := getWindowDimensions[int32]()

		rl.BeginDrawing()
		rl.ClearBackground(rl.Black)

		rl.DrawFPS(10, 10)
		textWidth := rl.MeasureText(greeting, fontSize)

		rl.DrawText(greeting, windowWidth-textWidth-textPadding, textPadding, fontSize, rl.Gray)
		rl.DrawCircle(windowWidth/2, windowHeight/2, circleRadius, rl.Blue)

		rl.EndDrawing()
	}
}
