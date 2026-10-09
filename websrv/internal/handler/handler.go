package handler

import (
	"net/http"

	ws "github.com/gofiber/contrib/v3/websocket"
	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/internal/store"
)

type Handler struct {
	store *store.Store
}

func (this *Handler) Init(app *fiber.App, store *store.Store) {
	this.store = store

	app.Use("/ws", this.socketUpgrade)
	app.Get("/ws", ws.New(this.handleSocketMessage))
}

func (this *Handler) socketUpgrade(fib fiber.Ctx) error {
	if ws.IsWebSocketUpgrade(fib) {
		return fib.Next()
	}

	return fib.SendStatus(http.StatusUpgradeRequired)
}
