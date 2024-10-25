
# (cd examples/plugin_go && capnp compile -I `go list -m -f '{{.Dir}}' capnproto.org/go/capnp/v3`/std -ogo:src urchin.capnp)
(cd examples/plugin_go && GOOS=wasip1 GOARCH=wasm go build -o output/main.wasm ./src )
(cd examples/plugin_rs && cargo build --target wasm32-wasip1)