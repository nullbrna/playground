FROM golang:1.27.3-alpine

WORKDIR /app

RUN go install github.com/air-verse/air@latest

# Static analysis/formatting tools.
# ---
# NOTE: See the Air configuration for the running commands.
RUN go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@latest
RUN go install go.uber.org/nilaway/cmd/nilaway@latest

CMD ["air", "-c", "./docker/air.toml"]
