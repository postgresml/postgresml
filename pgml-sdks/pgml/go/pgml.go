package main

/*
#cgo LDFLAGS: -l pgml -L ./../target/debug
#include "pgml.h"
*/
import "C"

import (
	"unsafe"
)

type Collection struct {
	collection *C.CollectionC
}

func main() {
	c_string_p := C.CString("Test CString")
	defer C.free(unsafe.Pointer(c_string_p))
	collection := C.new_collection(c_string_p)
	C.test_collection(collection)
	defer C.free_collection(collection)
}
