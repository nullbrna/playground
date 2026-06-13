FROM rust:alpine
WORKDIR /app

RUN apk add --no-cache musl-dev

COPY Cargo* .
COPY src    ./src

RUN cargo install cargo-watch

ENV MODE=local
# Add the feature for test-only logic if needed. Run with hot reloading.
CMD ["sh", "-c", "if [ \"$MODE\" = \"testing\" ]; then cargo watch -x 'run --features testing'; else cargo watch -x run; fi"]
