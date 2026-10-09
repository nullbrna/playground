Experiments, learnings, and everything else unfinished.

> **NOTE:** Any changes to `main.go` won't hot-reload the server.

## Adding A New Module

1. Create the new folder.
2. In the new folder, run `go mod init github.com/nullbrna/playground/...` to
   initialise.
3. Back at root, run `go work use ...` against the new folder.

## Running Tests

| Module   | Test                           | Benchmark                      |
| -------- | ------------------------------ | ------------------------------ |
| `websrv` | `go test -count=1 ./tests/...` | `go test -bench=. ./tests/...` |
