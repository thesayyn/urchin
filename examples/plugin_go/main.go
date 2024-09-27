// build with: GOOS=wasip1 GOARCH=wasm go build -o main.wasm main.go
package main

import "fmt"

//go:wasmimport urchin say_hello
//go:noescape
func say_hello()

func main() {
	say_hello()
	fmt.Println("Hello from Go!")
}
