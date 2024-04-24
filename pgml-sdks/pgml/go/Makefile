BINARY_NAME=pgml

build:
	cargo build --features c
	cargo expand --features c > expanded.rs
	cbindgen --lang C -o pgml.h expanded.rs
	# GOARCH=amd64 GOOS=darwin go build -o ${BINARY_NAME}-darwin main.go
	GOARCH=amd64 GOOS=linux go build -o ${BINARY_NAME}-linux pgml.go
	# GOARCH=amd64 GOOS=windows go build -o ${BINARY_NAME}-windows main.go

build_test:
	cargo build --features c
	cargo expand --features c > expanded.rs
	cbindgen --lang C -o pgml.h expanded.rs
	gcc test.c -o test -l pgml -L ./../target/debug

test: build_test
	LD_LIBRARY_PATH=./../target/debug ./test

test_c:
	gcc test.c -o test -l pgml -L ./../target/debug
	LD_LIBRARY_PATH=./../target/debug ./test

run: build
	LD_LIBRARY_PATH=./../target/debug ./${BINARY_NAME}-linux

clean:
	go clean
	# rm ${BINARY_NAME}-darwin
	rm ${BINARY_NAME}-linux
	# rm ${BINARY_NAME}-windows
