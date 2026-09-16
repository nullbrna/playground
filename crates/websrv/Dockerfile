FROM rust:alpine

WORKDIR /app

# [--no-cache] Don't wastefully retain package index.
# [  musl-dev] Add development headers for building in Alpine.
RUN apk add --no-cache musl-dev
RUN cargo install cargo-watch

CMD ["cargo", "watch", "-x", "run"]
