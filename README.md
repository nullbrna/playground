Experiments, learnings, and everything else unfinished.

## Local Usage

| Where    | What    | Command                        | Notes                                             |
| -------- | ------- | ------------------------------ | ------------------------------------------------- |
| `websrv` | Startup | `docker compose up`            |                                                   |
| `websrv` | Tests   | `go test -count=1 ./tests/...` | Server **must** be up. Cold cache with `<count>`. |
