FROM golang:1.27.3-alpine

WORKDIR /app

RUN go install github.com/air-verse/air@latest

# Static analysis tools.
# ---
# NOTE: See the Air configuration for the running commands.
RUN go install go.uber.org/nilaway/cmd/nilaway@latest
RUN go install github.com/nishanths/exhaustive/cmd/exhaustive@latest

CMD ["air", "-c", "./docker/air.toml"]
