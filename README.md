Experiments, learnings, and everything else unfinished.

## Adding A New Module

1. Create the new folder.
2. In the new folder, run `go mod init github.com/nullbrna/playground/...` to
   initialise.

## Running Tests

| Module   | Test                           | Benchmark                      | Notes                                            |
| -------- | ------------------------------ | ------------------------------ | ------------------------------------------------ |
| `websrv` | `go test -count=1 ./tests/...` | `go test -bench=. ./tests/...` | Changes to `main.go` won't hot-reload the server |
