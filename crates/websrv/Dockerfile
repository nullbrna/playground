FROM rust:alpine

WORKDIR /app

# Install dependency for hot reloading support.
# ---
# <locked> Honour the crate's dependency versions.
RUN cargo install cargo-watch --locked

CMD ["cargo", "watch", "-x", "run"]
