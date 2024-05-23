BINARY_NAME=pgml
HEADER=${BINARY_NAME}.h
PGML_LIB=../target/debug/

bindings:
	cargo b --features c
	RUST_TOOLCHAIN=$(dirname $(rustup +nightly which rustc)) cbindgen --config cbindgen.toml --output ${HEADER} ../

build: bindings
	gcc -Wall -o ./example -Iinclude/ -L${PGML_LIB} -l ${BINARY_NAME} example.c

run: build
	LD_LIBRARY_PATH=${PGML_LIB} ./example

clean:
	rm ${HEADER}
	rm -rf ./example

