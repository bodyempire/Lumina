# Lumina Go Wrapper

This is the Go package for Lumina v1.4, interacting with the `lumina_ffi` shared library via `cgo`.

## Prerequisites
Build the FFI library first:
```bash
cargo build --release -p lumina-ffi
```

## Running Tests
Ensure Go can find the shared library at test time:
```bash
CGO_LDFLAGS="-L../../../target/release -llumina_ffi" \
LD_LIBRARY_PATH=../../../target/release \
go test -v ./...
```
