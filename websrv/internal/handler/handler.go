package handler

import (
	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/internal/store"
)

type Handler struct {
	store *store.Store
}

func (this *Handler) Init(app *fiber.App, store *store.Store) {
	this.store = store
	app.Get("/", this.index)
}

func (this *Handler) index(reqCtx fiber.Ctx) error {
	return reqCtx.SendString("Hello, world!")
}
