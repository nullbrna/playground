FROM rust:alpine AS development

WORKDIR /app

# Install dependencies for building and hot reloading.
RUN apk add --no-cache musl-dev
RUN cargo install cargo-watch

CMD ["cargo", "watch", "-x", "run"]

FROM rust:alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev

# Build a release binary. Creates a fresh ".lock" file.
COPY Cargo.toml .
COPY src        ./src

RUN cargo build --release

FROM scratch AS release

COPY --from=builder /app/target/release/web-server /web-server

CMD ["/web-server"]
