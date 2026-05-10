###############
# Development #
###############

FROM rust:alpine
WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo* .
COPY src    ./src

RUN cargo install cargo-watch

# Restart file watcher through non-zero exit codes.
CMD ["sh", "-c", "while true; do cargo watch -x run; sleep 1; done"]

###############
# Production  #
###############

# FROM rust:alpine AS builder
# WORKDIR /app

# RUN apk add --no-cache musl-dev
# RUN cargo init --bin --vcs none

# COPY Cargo* .
# COPY src    ./src

# RUN cargo build -r

# FROM scratch

# COPY --from=builder /app/target/release/<binary> .

# CMD ["/<binary>]
