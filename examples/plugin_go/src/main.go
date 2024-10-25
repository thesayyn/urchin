// GOOS=wasip1 GOARCH=wasm go build -o main.wasm main.go
package main

import (
	"fmt"

	"capnproto.org/go/capnp/v3"
)

//go:wasmimport urchin exchange
//go:noescape
func exchange()

func main() {
	arena := capnp.SingleSegment(nil)

	_, seg, err := capnp.NewMessage(arena)
	if err != nil {
		panic(err)
	}
	person, err := NewRootPerson(seg)
	if err != nil {
		panic(err)
	}
	person.SetName("Alice")
	person.SetEmail("e@example.com")

	_ = person.ToPtr().EncodeAsPtr(seg)
	exchange()

	fmt.Println("Hello from go!")
}
