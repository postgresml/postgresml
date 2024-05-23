BINARY_NAME=pgml
HEADER=include/${BINARY_NAME}.h
PGML_LIB=../target/debug/

bindings:
	cargo b --features c
	RUST_TOOLCHAIN=$(dirname $(rustup +nightly which rustc)) cbindgen --config cbindgen.toml --output ${HEADER} ../

build: bindings
	gcc -Wall -o build/example -Iinclude/ -L${PGML_LIB} -l ${BINARY_NAME} example/main.c

run: build
	LD_LIBRARY_PATH=${PGML_LIB} ./build/example

clean:
	rm ${HEADER}

