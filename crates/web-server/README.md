## Usage

### Starting the server locally

```sh
docker compose up

# The tests require environment variables so it may be easier to logon.
docker exec -it web-server-server-1 sh
```

### Creating a new named migration

> Migration script will be placed in `migrations/` which will be the default for
  any runtime migrations.

```sh
cargo install sqlx
sqlx migrate add <name>
```
