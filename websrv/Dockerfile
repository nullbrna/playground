FROM golang:1.27.2-alpine

WORKDIR /app

RUN go install github.com/air-verse/air@latest

# Static analysis tools, see the Air configuration.
RUN go install go.uber.org/nilaway/cmd/nilaway@latest
RUN go install github.com/nishanths/exhaustive/cmd/exhaustive@latest

CMD ["air", "-c", "./docker/air.toml"]
