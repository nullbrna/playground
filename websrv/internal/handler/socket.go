package handler

import (
	"strings"

	ws "github.com/gofiber/contrib/v3/websocket"
	zlog "github.com/rs/zerolog/log"
)

func (this *Handler) handleSocketMessage(conn *ws.Conn) {
	defer conn.Close()
	conn.SetReadLimit(1024) // 1KB.

	acceptedClosureErrors := []int{
		ws.CloseNormalClosure,
		ws.CloseGoingAway,
		ws.CloseAbnormalClosure,
	}

	for {
		messageType, message, err := conn.ReadMessage()
		if err != nil {
			if ws.IsCloseError(err, acceptedClosureErrors...) {
				zlog.Trace().Msg("Connection closed")
				return
			}

			zlog.Error().Err(err).Msg("Reading message")
			return
		}

		if messageType != ws.TextMessage {
			zlog.Debug().Int("type", messageType).Msg("Unexpected message type")
			continue
		}

		messageText := string(message)
		if strings.EqualFold(messageText, "ping") {
			conn.WriteMessage(ws.TextMessage, []byte("pong"))
			continue
		}

		zlog.Warn().Str("text", messageText).Msg("Unexpected text message")
	}
}
