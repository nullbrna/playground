Experiments, learnings, and everything else unfinished.

## Local Running

| Where    | What    | Command                    | Notes                                         |
| -------- | ------- | -------------------------- | --------------------------------------------- |
| `websrv` | Startup | `docker compose up`        |                                               |
| `websrv` | Tests   | `go test -count=1 ./tests` | Cold cache with `-count=1`, server must be up |
