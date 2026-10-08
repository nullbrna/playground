FROM golang:1.27.1-alpine

WORKDIR /app

# [--no-cache] Don't wastefully retain package index.
RUN apk add --no-cache git
RUN go install github.com/air-verse/air@latest

CMD ["air"]
