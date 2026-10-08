package handler

import (
	"context"
	"net/http"
	"strconv"
	"time"

	"github.com/gofiber/fiber/v3"
	"github.com/nullbrna/playground/websrv/internal/store"
)

type Handler struct {
	store *store.Store
}

func (this *Handler) Init(app *fiber.App, store *store.Store) {
	this.store = store
	app.Get("/ten-users", this.tenUsers)
	app.Get("/user/:id", this.userById)
	app.Get("/user-count", this.userCount)
}

func (this *Handler) tenUsers(reqCtx fiber.Ctx) error {
	ctx, cancel := context.WithTimeout(reqCtx.RequestCtx(), 3*time.Second)
	defer cancel()

	users, code := this.store.GetFirstTenUsers(ctx)
	if code != http.StatusOK {
		return reqCtx.SendStatus(code)
	}

	return reqCtx.JSON(users)
}

func (this *Handler) userById(reqCtx fiber.Ctx) error {
	idFromParam, err := strconv.ParseInt(reqCtx.Params("id"), 10, 64)
	if err != nil || idFromParam < 1 {
		return fiber.ErrBadRequest
	}

	ctx, cancel := context.WithTimeout(reqCtx.RequestCtx(), 3*time.Second)
	defer cancel()

	user, code := this.store.GetUserById(ctx, idFromParam)
	if code != http.StatusOK {
		return reqCtx.SendStatus(code)
	}

	return reqCtx.JSON(user)
}

func (this *Handler) userCount(reqCtx fiber.Ctx) error {
	ctx, cancel := context.WithTimeout(reqCtx.RequestCtx(), 3*time.Second)
	defer cancel()

	count, code := this.store.GetUserCount(ctx)
	if code != http.StatusOK {
		return reqCtx.SendStatus(code)
	}

	countResponse := strconv.FormatInt(count, 10)
	return reqCtx.SendString(countResponse)
}
