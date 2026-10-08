Experiments, learnings, and everything else unfinished.

## Adding A New Module

1. Create the new folder.
2. In the new folder, run `go mod init github.com/nullbrna/playground/...` to
   initialise.
3. Back at root, run `go work use ...` against the new folder.

## Running Tests

1. Run `go test -count=1 ./tests/...` for all tests.
