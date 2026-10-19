package main

import ray "github.com/gen2brain/raylib-go/raylib"

const (
	windowWidth   int32 = 800
	windowHeight  int32 = 450
	windowPadding int32 = 10
	fontSize      int32 = 40

	windowTitle = "Hello World"
)

func main() {
	ray.SetTraceLogLevel(ray.LogWarning)
	ray.InitWindow(windowWidth, windowHeight, windowTitle)
	defer ray.CloseWindow()

	startupMessageWidth := ray.MeasureText(windowTitle, fontSize)
	startupMessageHor := (windowWidth - startupMessageWidth) / 2
	startupMessageVer := (windowHeight - fontSize) / 2

	ray.SetTargetFPS(60)
	for !ray.WindowShouldClose() {
		ray.BeginDrawing()
		ray.ClearBackground(ray.Black)

		ray.DrawText(
			windowTitle,
			startupMessageHor,
			startupMessageVer,
			fontSize,
			ray.White,
		)

		ray.DrawFPS(windowPadding, windowPadding)
		ray.EndDrawing()
	}
}
