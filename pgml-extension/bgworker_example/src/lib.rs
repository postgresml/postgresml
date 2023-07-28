use std::{
    os::unix::net::{UnixListener, UnixStream},
    thread,
};

use bgworker_message::{Reply, Request};
use pgrx::prelude::*;
use serde_json::json;

pgrx::pg_module_magic!();

#[pg_guard]
#[no_mangle]
pub extern "C" fn bg_main(datum: pg_sys::Datum) {
    let path = unsafe { String::from_datum(datum, false) }.unwrap();

    let listener = UnixListener::bind(&path).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                thread::spawn(|| handle_client(stream));
            }
            Err(_) => {
                /* connection failed */
                break;
            }
        }
    }
}

fn handle_client(mut stream: UnixStream) {
    /* Read request */
    let _request = Request::recv(&mut stream).expect("failed to recv request");

    /* Process request */

    /* Send reply */
    let reply = Reply::Transform(Ok(json!({ "success": true })));
    reply.send(&mut stream).unwrap();
}
