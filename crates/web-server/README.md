## Usage

- To start the server locally:

```sh
docker compose up
# The tests require environment variables so it may be easier to logon.
docker exec -it web-server-server-1 sh
```

- Create a new named migration:

```sh
# Install helper program.
cargo install sqlx

sqlx migrate add <name>
```

> Migration script will be placed in `migrations/` which will be the default for
  any runtime migrations.
