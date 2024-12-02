package main

import (
	"net/http"
)

func main() {
	mux := http.NewServeMux()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		println("healthcheck")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("healthy"))
	})

	mux.HandleFunc("GET /comment/", SomeGetMethod)
	mux.HandleFunc("POST /comment/", SomePostMethod)
	mux.HandleFunc("/", SomeGetMethod)

	println("Listening on port 8090")
	if err := http.ListenAndServe(":8090", mux); err != nil {
		panic(err)
	}
}

func SomeGetMethod(w http.ResponseWriter, r *http.Request) {
	_, err := w.Write([]byte("GET method response"))
	println("hello from", r.URL.Path)
	if err != nil {
		panic(err)
	}
}

func SomePostMethod(w http.ResponseWriter, r *http.Request) {
	_, err := w.Write([]byte("POST method response"))
	println("hello from", r.URL.Path)
	if err != nil {
		panic(err)
	}
}
