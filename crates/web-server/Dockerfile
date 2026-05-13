###############
# Development #
###############

FROM rust:alpine
WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo* .
COPY src    ./src

RUN cargo install cargo-watch

ENV MODE=local

# Add the feature for test-only logic if needed; run with hot-reloading.
CMD ["sh", "-c", "if [ \"$MODE\" = \"testing\" ]; then cargo watch -x 'run --features testing'; else cargo watch -x run; fi"]

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
